% INTEGRAL-DRIVE(1) Version 0.3.0 | Integral Drive Manual

NAME
====

Integral Drive - Recursively compute checksums on files

SYNOPSIS
========

integral-drive `[-h|--help] [-f|--force] -i|--input` *DIRECTORY* [`-o|--output` *DIRECTORY*] [`-i|--input` *DIRECTORY* [`-o|--output` *DIRECTORY*]]... [`-l|--log-file` *FILE*] [`-c|--config` *FILE*]

DESCRIPTION
===========

This tool is intended for checking the integrity of a directory located in a hard drive. The tool recursively walks through a directory, skipping some directories not considered valuable in a backup, and computing the CRC32C checksum of each file. The output is a text file with one path/checksum entry per line. The given root path is removed from the paths and entries are alphabetically sorted, so that you can easily compare folders located on two different hard drives with the tool of your choice such as diff or Meld. The log is saved into a file.

The CRC32C is not cryptographically safe but very fast, your CPU probably has instruction-level support -- SSE 4.2. It is not collision-resistant but still have good error detection capabilities. The CRC32C is the default checksum used at the block level on Btrfs.

APPLICATION USAGE
=================

For example **integral-drive `--input` */path/to/your/stuff* `--output` *integral_drive.txt* `--log-file` *backup.log***. If the **`--output`** option is missing, **integral_drive.txt** will be saved at the root of the input path. If the **`--log-file`** option is missing, the log will only be printed in the standard output.

The total number of entries is the total number of files and directories excluding symbolic links and directories not considered valuable in a backup. The list of non-valuable directories is configurable by adding **`--config` *path/to/your/config.toml*** where ***config.toml*** could contain for example:

```
ignore = [
    ".idea", # JetBrains stuff
    ".vscode", # Visual Studio Code stuff
    "node_modules", # JavaScript packages
    "venv", # Python virtual environment
    "__pycache__", # Python cache
    ".mypy_cache", # Python cache
    ".pytest_cache", # Python cache
    "htmlcov", # Python cache
    ".scannerwork", # Sonar reports
    ".vercel", # Vercel stuff
]
```

The generated list of checksums is saved into a file containing for example:

```
# Processed at: 2022-03-15 17:05:28 UTC
# Input path: /path/to/your/stuff
image.jpg; 8BDC85FF
nothing.txt; <EMPTY>
sub_folder/image.jpg; <DENIED>
sub_folder/symlink_blabla.txt; <SYMLINK>
symlink_sub_folder; <SYMLINK>
```

The value **`<EMPTY>`** is printed if the file is empty. An empty directory is skipped. The value **`<DENIED>`** is printed if you don't have read access to the file (or directory), in that case you may retry after either by changing the file/directory permission or by elevating yourself to sudo/admin user. The value **`<SYMLINK>`** is printed if the file (or directory) is a symbolic link, the file/directory is not read/open because the link could potentially break without notice -- f.i. when the drive is mounted to another point. Finally, the value **`<UNKNOWN>`** is printed if something unexpected occurred.

RATIONALE
=========

This tool is useful for checking what has been actually transferred. Cloud service providers or filesystems may not warn you in case of data integrity errors. They might rename your files or metadata, corrupt files or loose symbolic links without notice. It is therefore recommended checking the integrity of your offline/air-gap backup. This tool does not copy anything, `cp` and `rsync` already exist.

OPTIONS
=======

`-i, --input` *DIRECTORY*

:   Root path to start from, you can provide one or many paths by repeating the option

`-o, --output` *DIRECTORY*

:   Path to the report file, none or as many as input paths

`-l, --log-file` *FILE*

:   Path to the optional log file

`-c, --config`

:   Path to the config file in TOML format -- if missing, all directories will be processed

`-f, --force`

:   Overwrite output file if already existing

AUTHOR
======

What matters is not who I am, but what I do.

I am open to discussion at https://github.com/coffeacloudberry/integral-drive/discussions

Private talk with PGP, my key is https://keybase.io/happydude/pgp_keys.asc fingerprint DFFF34860D361C52

REPORTING BUGS
==============

Bugs can be reported and filed at https://github.com/coffeacloudberry/integral-drive/issues

COPYRIGHT
=========

Copyright © 2023 coffeacloudberry. Hippocratic License 3.0

https://firstdonoharm.dev
https://raw.githubusercontent.com/coffeacloudberry/integral-drive/main/LICENSE.pdf

SEE ALSO
========

Github repository at https://github.com/coffeacloudberry/integral-drive
