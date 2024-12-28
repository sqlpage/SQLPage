# Download SQLPage

SQLPage is a small single-file program that will
execute the SQL files you write,
and render the database responses as nice web pages.

If you have already downloaded SQLPage,
you can skip this step and [start writing your website](#tutorial).

[Download the latest SQLPage](https://github.com/sqlpage/SQLPage/releases) for your operating system.
In the _release assets_ section, you will find files named `sqlpage-windows.zip`, `sqlpage-linux.tgz`, and `sqlpage-macos.tgz`.
Download the one that corresponds to your operating system, and extract the executable file from the archive.

> **Note**: On Mac OS, Apple blocks the execution of downloaded files by default. The easiest way to run SQLPage is to use [Homebrew](https://brew.sh).

> **Note**: Advanced users can alternatively install SQLPage using:
>  - [docker](https://hub.docker.com/repository/docker/lovasoa/SQLPage/general) (docker images are also available for ARM, making it easy to run SQLPage on a Raspberry Pi, for example),
> - [brew](https://formulae.brew.sh/formula/sqlpage) (the easiest way to install SQLPage on Mac OS),
> - [nix](https://search.nixos.org/packages?channel=unstable&show=sqlpage) (declarative package management for reproducible deployments),
> - [scoop](https://scoop.sh/#/apps?q=sqlpage&id=305b3437817cd197058954a2f76ac1cf0e444116) (a command-line installer for Windows),
> - or [cargo](https://crates.io/crates/sqlpage) (the Rust package manager).

You can also find the source code of SQLPage on [GitHub](https://github.com/sqlpage/SQLPage), [install rust](https://www.rust-lang.org/tools/install) on your computer, and compile it yourself with `cargo install sqlpage`.

See the instructions for [MacOS](?os=macos#download), or for [Windows](?os=windows#download).
