# Contributing

So glad you land on this page! Feel free to share any idea. Your contribution is more than welcome.

Feel free to open an issue or a discussion.

## Ideas

* Add unit tests running on different distributions,
* Add cryptographically safe checksums,
* Any idea?

## How to Create a Release?

Docker is required. It is used to compile the application and build packages for recent as well as older distributions. Run the Docker daemon and `make release`.

The [RPMs](https://rpm-packaging-guide.github.io/) are built with [cargo-rpm](https://crates.io/crates/cargo-rpm) and target Fedora. The DEB is built with [cargo-deb](https://crates.io/crates/cargo-deb) and targets Debian (Ubuntu is based on Debian).

## Man Page Guidelines

* [Utility Argument Syntax](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html) (SYNOPSIS section),
* [Headers descriptions](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap13.html).
