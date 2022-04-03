# Integral Drive

This project provides a command line interface (CLI) for checking the integrity of a folder located in a hard drive. The tool recursively walks through a path, skipping some folders (not considered valuable in a backup), and computing the CRC32C checksum of each file. The output is a text file with one path/checksum entry per line. The given root path is removed from the paths and entries are alphabetically sorted, so that you can easily compare folders located on two different hard drives with the tool of your choice such as `diff`, `git`, [Meld](https://meldmerge.org/). The log can be saved into a file in addition to the standard output and contains info, warnings, and errors.

## Use Cases

* Cloud file storage providers may offer top security features, and comply with many standards, but may lack data integrity under specific conditions such as poor network connection. Lost files can be detected by comparing the original local folder to the fetched folder,
* Cloud file storage providers may rename or duplicate files containing special characters in the file name. A copy from one drive to another may remove file metadata, such as the creation/modification date. In those cases, the checksum shall not change,
* Some filesystems do not have integrity check, or do not handle special characters, or may corrupt files without notice. A copied file may be lost if the hard drive has been removed improperly and not actually flushed. Consequently, integrity check is useful, especially on your backup since you realise you have lost files when it is too late,
* You or your applications/system may use symbolic links. If you make the mistake to back up only those links, and the actual files are lost, then you will only find broken links. This tool explicitly tells you which files/folders are symbolic links so that you can act accordingly.

**Notices:**

* This tool only checks unintentional changes. CRC32C is fast, but not recommended for checking malicious change,
* For detecting intrusion on your system, [AIDE](https://aide.github.io/) is probably a good choice,
* This tool has only been tested on a GNU/Linux operating system.

## Usage

For example `integral-drive --input /path/to/your/stuff --output integral_drive.txt --log-file backup.log`. If the `--output` option is missing, `integral_drive.txt` will be saved at the root of the input path. If the `--log-file` option is missing, the log will only be printed in the standard output.

The total number of entries is the total number of files and directories excluding symbolic links and directories not considered valuable in a backup. The list of non-valuable directories is configurable in [config.toml](config.toml). You can download this file and append the argument `--config path/to/your/config.toml`.

The generated list of checksums is saved into `integral_drive.txt` and contains for example:

```
# Processed at: 2022-03-15 17:05:28 UTC
# Input path: /path/to/your/stuff
blabla.txt; D2D45B01
copy_blabla.txt; D2D45B01
copy_image.jpg; 8BDC85FF
image.jpg; 8BDC85FF
modified_blabla.txt; C5B9D87F
nothing.txt; <EMPTY>
recompressed_image.jpg; CFC7A135
sub_folder/blabla.txt; D2D45B01
sub_folder/image.jpg; <DENIED>
sub_folder/symlink_blabla.txt; <SYMLINK>
symlink_sub_folder; <SYMLINK>
```

The value `<EMPTY>` is printed if the file is empty. An empty directory is skipped. The value `<DENIED>` is printed if you don't have read access to the file (or directory), in that case you may retry after either by changing the file/directory permission or by elevating yourself to sudo/admin user. The value `<SYMLINK>` is printed if the file (or directory) is a symbolic link, the file/directory is not read/open because the link could potentially break without notice (f.i. when the drive is mounted to another point). Finally, the value `<UNKNOWN>` is printed if something unexpected occurred.

To sum up:

| Checksum Error | Detail            | Apply to File | Apply to Directory | Log Level |
|----------------|-------------------|---------------|--------------------|-----------|
| `<EMPTY>`      | Empty file        | Yes           | No                 | Nil       |
| `<DENIED>`     | Permission denied | Yes           | Yes                | Warn      |
| `<SYMLINK>`    | Symbolic link     | Yes           | Yes                | Info      |
| `<UNKNOWN>`    | Unexpected error  | Yes           | Yes                | Error     |

**Notices:**

* The output file is wiped out at the beginning of the process, and filled in at the very end,
* Run [`detox`](https://linux.die.net/man/1/detox) if your backup has missing files containing exotic characters.

### Parallel Check

It is possible to provide multiple input paths that would be processed in parallel and use the full power of your multicore CPU! Let's say you just backed up your valuable files and want to compute the checksums of both the original files and your backup. You can run: `integral-drive --input /path/to/your/stuff --input /path/to/your/backup --output integral_drive_stuff.txt --output integral_drive_backup.txt --log-file backup.log`. The log file is shared. If no output file is given, then an `integral_drive.txt` would be found in every given input. The number of input paths is not limited.

## Install

This tool has no dependencies except GLIBC 2.31+ that should already be in your system.

### Fedora 64-bit (x86_64)

1. [Download the RPM](https://github.com/coffeacloudberry/integral-drive/releases)
2. (Optional) The packages are digitally signed to make sure no third party can alter the content. [Download my PGP public key](https://keybase.io/happydude/pgp_keys.asc) (fingerprint `DFFF34860D361C52`) and import it with `sudo rpm --import pgp_keys.asc` and verify the package with `rpm -K integral-drive*.rpm`, the output must be `digests signatures OK`
3. Install by running `sudo rpm -U integral-drive*.rpm`

### Debian-based 64-bit (x86_64)

1. [Download the DEB](https://github.com/coffeacloudberry/integral-drive/releases) (and associated .sig for GPG signature verification)
2. (Optional) The package is digitally signed to make sure no third party can alter the content. [Download my PGP public key](https://keybase.io/happydude/pgp_keys.asc) (fingerprint `DFFF34860D361C52`) and import it with `gpg --import pgp_keys.asc` and verify the package with `gpg --verify integral-drive*.deb.asc`, the output must be `Good signature` with the email address given in the PGP public key.
3. Install by running `sudo dpkg -i integral-drive*.deb`

## Build From Source

First, you need to [install Rust](https://www.rust-lang.org/tools/install), clone this repo, and build with `cargo build --release`

## License

This project is licensed under the [Hippocratic License 3.0](https://firstdonoharm.dev/). Please read the license ([md](LICENSE.md)/[pdf](LICENSE.pdf)) carefully since this is not *open* according to the Open Source Initiative, and not *free* according to the Free Software Foundation.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

Please read [SECURITY.md](SECURITY.md).
