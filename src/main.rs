use chrono::prelude::*;
use clap;
use clap::Parser;
use lazy_static;
use log;
use pbr::{MultiBar, Pipe, ProgressBar};
use serde::Deserialize;
use simplelog::*;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Stdout, Write};
use std::path::Path;
use std::path::PathBuf;
use std::string::String;
use std::thread;
use std::time::Duration;
use toml;
use walkdir::{DirEntry, FilterEntry, IntoIter, WalkDir};

#[derive(Debug, Deserialize, PartialEq)]
struct MyConfig {
    ignore: Vec<String>,
}

// Make the app customization global, set default if not specified in the CLI argument.
lazy_static::lazy_static! {
    static ref CONFIG: MyConfig = {
        let args = Cli::parse();
        match args.config {
            None => MyConfig { ignore: Vec::new() },
            Some(config_path) => {
                match fs::read_to_string(config_path) {
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    },
                    Ok(config_str) => {
                        toml::from_str::<MyConfig>(config_str.as_str()).unwrap()
                    }
                }
            }
        }
    };
}

/// Recursively compute checksums on files.
///
/// This tool is intended for checking the integrity of a folder located in a hard drive. The tool
/// recursively walks through a path, skipping some folders (not considered valuable in a backup),
/// and computing the CRC32C checksum of each file. The output is a text file with one
/// path/checksum entry per line. The given root path is removed from the paths and entries are
/// alphabetically sorted, so that you can easily compare folders located on two different hard
/// drives with the tool of your choice such as `diff` or Meld. The log is saved into a file.
///
/// The CRC32C is not cryptographically safe but very fast, your CPU probably have
/// instruction-level support (SSE 4.2). It is not collision-resistant but still have good error
/// detection capabilities. The CRC32C is the default checksum used at the block level on Btrfs.
#[derive(Parser)]
#[clap(version, color = clap::ColorChoice::Never)]
struct Cli {
    /// Root path to start from, you can provide one or many paths
    #[clap(short, long, parse(from_os_str), name("INPUT PATH"))]
    input: Vec<PathBuf>,

    /// Path to the report file, none or as many as input paths
    #[clap(short, long, parse(from_os_str), name("OUTPUT FILENAME"))]
    output: Option<Vec<PathBuf>>,

    /// Path to the log file, none or one file
    #[clap(short, long, parse(from_os_str), name("LOG FILENAME"))]
    log_file: Option<PathBuf>,

    /// Path to the config file in TOML format. If missing, all folders will be processed
    ///
    /// Example: https://github.com/coffeacloudberry/integral-drive/blob/main/config.toml
    #[clap(short, long, parse(from_os_str), name("CONFIG FILENAME"))]
    config: Option<PathBuf>,
}

/// Return false if the element is blacklisted.
///
/// Exclude user-generated folders such as project- and language-specific stuff. Such folders
/// include a large amount of files that are irrelevant in a backup since they do not contain
/// valuable information and can be easily re-generated.
fn is_valuable(entry: &DirEntry) -> bool {
    match entry.file_name().to_str() {
        None => true,
        Some(s) => !CONFIG.ignore.contains(&String::from(s)),
    }
}

/// Compute the CRC-32-Castagnoli of a given file. Output example: 07CF90A4
fn file_crc32c(path: &Path, file_type: Option<fs::FileType>, n_pbs: usize) -> String {
    let printable_path = path.to_str().unwrap();
    if file_type.is_some() && file_type.unwrap().is_symlink() {
        log::info!("Symbolic link `{}'", printable_path);
        nnl(n_pbs);
        String::from("<SYMLINK>")
    } else {
        match fs::read(path) {
            Err(e) => {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    log::warn!("Permission denied to read `{}'", printable_path);
                    nnl(n_pbs);
                    String::from("<DENIED>")
                } else {
                    log::error!("Failed to read `{}'", printable_path);
                    nnl(n_pbs);
                    String::from("<UNKNOWN>")
                }
            }
            Ok(content) => {
                if content.len() == 0 {
                    String::from("<EMPTY>")
                } else {
                    format!("{:08X}", crc32fast::hash(&*content))
                }
            }
        }
    }
}

/// Process one file or directory. The root path is removed from the path.
fn process_entry(
    path: &Path,
    root_path_length: usize,
    v: Option<&String>,
    file_type: Option<fs::FileType>,
    n_pbs: usize,
) -> Option<String> {
    match path.to_str() {
        None => None,
        Some(path_str) => match String::from(path_str).get(root_path_length..) {
            None => None,
            Some(s) => match v {
                None => {
                    let printed_checksum = file_crc32c(path, file_type, n_pbs);
                    Some(format_entry(s, &printed_checksum))
                }
                Some(info) => Some(format_entry(s, info)),
            },
        },
    }
}

/// String for one entry in the output file.
fn format_entry(file_path: &str, checksum: &String) -> String {
    format!("{}; {}", file_path, checksum)
}

/// String on top of the output file.
fn format_header(input_path: &PathBuf) -> String {
    let dt = Utc::now();
    let dt_str = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    format!(
        "# Processed at: {}\n# Input path: {}\n",
        dt_str,
        input_path.display()
    )
}

/// Return an iterator of the entries to process.
fn get_walk_dir(root: &PathBuf) -> FilterEntry<IntoIter, fn(&DirEntry) -> bool> {
    WalkDir::new(root).into_iter().filter_entry(is_valuable)
}

/// Print a number of new lines to stay outside the multi progress bar.
fn nnl(n_pbs: usize) {
    print!("{}", "\n".repeat(n_pbs));
}

/// Handle the permission denied error.
fn denied_entry(path: &Path, path_len: usize, n_pbs: usize, entries: &mut Vec<String>) {
    let s = String::from("<DENIED>");
    let entry = process_entry(path, path_len, Some(&s), None, n_pbs).unwrap();
    entries.push(entry);
    log::warn!("Permission denied to read `{}'", path.display());
    nnl(n_pbs);
}

/// Walk through a given path recursively and return the entire content file to write.
///
/// The path are sorted alphabetically to ease comparison and the possibility to compare with any
/// diff/git interface. The output file is written in one go to reduce system calls.
fn walk_dir(path: PathBuf, pb: &mut ProgressBar<Pipe>, n_pbs: usize) -> String {
    let path_len = Path::new(&path).as_os_str().len() + 1;
    let walker = get_walk_dir(&path);
    let mut all_entries = Vec::new();
    for entry in walker {
        match entry {
            Ok(entry) => {
                let file_type = entry.file_type();
                if !file_type.is_dir() {
                    let path = entry.path();
                    match process_entry(path, path_len, None, Some(file_type), n_pbs) {
                        Some(entry) => all_entries.push(entry),
                        None => {
                            log::error!("Fail to process `{}'", path.display());
                            nnl(n_pbs);
                        }
                    }
                }
            }
            Err(e) => {
                let path = e.path().unwrap();
                if let Some(inner) = e.io_error() {
                    match inner.kind() {
                        io::ErrorKind::PermissionDenied => {
                            denied_entry(&path, path_len, n_pbs, &mut all_entries);
                        }
                        _ => {
                            log::error!("{:?}", e);
                            nnl(n_pbs);
                        }
                    }
                }
            }
        }
        pb.inc();
    }
    all_entries.sort();
    let mut output_str = format_header(&path);
    output_str.push_str(all_entries.join("\n").as_str());
    output_str.push('\n');
    output_str
}

/// Set log according to CLI argument, configure terminal output as well as log file.
fn setup_log(output_path: Option<PathBuf>) -> Result<(), log::SetLoggerError> {
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Never, // bad contrast
    )];
    if output_path.is_some() {
        loggers.push(WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(output_path.unwrap()).unwrap(),
        ));
    }
    CombinedLogger::init(loggers)
}

/// Check arguments and return a list of file names as reports. Do not write anything.
fn process_args(arg_input: &Vec<PathBuf>, arg_output: &Option<Vec<PathBuf>>) -> Vec<PathBuf> {
    for input in arg_input {
        if !input.is_dir() {
            eprintln!("Input path must be a directory");
            std::process::exit(1);
        }
    }
    let output_path = match arg_output {
        Some(p) => p.to_vec(),
        None => arg_input
            .iter()
            .map(|p| {
                let mut op = PathBuf::from(&p);
                op.push("integral_drive.txt");
                op
            })
            .collect::<Vec<PathBuf>>(),
    };
    if arg_input.len() != output_path.len() {
        eprintln!("Number of output files must be equal to number of input paths");
        std::process::exit(1);
    }
    for output in &output_path {
        if output.is_dir() {
            eprintln!("Output path cannot be directory, please add a file name");
            std::process::exit(1);
        }
        if output.exists() {
            let output_path_str = PathBuf::from(&output)
                .into_os_string()
                .into_string()
                .unwrap();
            eprintln!("Output file already exists `{}'", output_path_str);
            std::process::exit(1);
        }
    }

    output_path
}

/// Create and customize a progress bar and add it to the multi progress bar.
fn create_pb(multi_pb: &MultiBar<Stdout>, total_entries: u64) -> ProgressBar<Pipe> {
    let mut pb = multi_pb.create_bar(total_entries);
    pb.show_speed = false; // meaningless
    pb.show_time_left = false; // inaccurate
    pb.format("[#>~]");
    pb.set_max_refresh_rate(Some(Duration::from_millis(500))); // reduce CPU usage
    pb
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    if args.input.len() == 0 {
        eprintln!("Missing input, try --help");
        std::process::exit(1);
    }
    let output_path = process_args(&args.input, &args.output);

    // setup log and optionally create a common log file
    setup_log(args.log_file)?;

    // process in parallel
    let mut handles = Vec::with_capacity(output_path.len());
    let multi_pb = MultiBar::new();
    let n_pbs = args.input.len();
    for (pos, output) in output_path.iter().enumerate() {
        let input = PathBuf::from(&args.input[pos]);
        let mut output_buf = File::create(&output)?;
        let input_path = fs::canonicalize(&input)?;
        let total_entries = get_walk_dir(&input_path).count();
        let mut pb = create_pb(&multi_pb, total_entries as u64);
        let op = PathBuf::from(&output);
        handles.push(thread::spawn(move || {
            let content_file = walk_dir(input_path, &mut pb, n_pbs);
            let output_path_str = op.into_os_string().into_string().unwrap();
            let mut message = format!("Saving checksums into `{}'...", output_path_str);
            pb.message(message.as_str());
            output_buf.write_all(content_file.as_ref()).unwrap();
            message = format!("Checksums saved in `{}'", output_path_str);
            pb.finish_print(message.as_str());
        }));
    }

    // wait for all threads to finish
    multi_pb.listen();
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}
