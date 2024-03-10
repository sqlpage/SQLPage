# Download SQLPage for Mac OS

On Mac OS, Apple blocks the execution of downloaded files by default. The easiest way to run SQLPage is to use [Homebrew](https://brew.sh).
Open a terminal and run the following commands:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install sqlpage
sqlpage
```

> **Note**: Advanced users can alternatively install SQLPage using
> [the precompiled binaries](https://github.com/sqlpage/SQLPage/releases/latest),
> [docker](https://hub.docker.com/repository/docker/lovasoa/SQLPage/general),
> [nix](https://search.nixos.org/packages?channel=unstable&show=sqlpage),
> or [cargo](https://crates.io/crates/sqlpage).

> **Not on Mac OS?** See the instructions for [Windows](?os=windows#download), or for [Other Systems](?os=any#download).