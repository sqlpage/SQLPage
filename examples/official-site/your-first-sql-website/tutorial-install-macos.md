# Install SQLPage on macOS

The recommended and easiest way to install SQLPage on macOS is [Homebrew](https://brew.sh/), including on Intel Macs.
If you do not already have Homebrew, open Terminal and run:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the installer's **Next steps** to add Homebrew to your PATH. Then install SQLPage:

```sh
brew install sqlpage
```

Open Terminal in your website folder and run `sqlpage` to start your website.
To update SQLPage later, run `brew update` followed by `brew upgrade sqlpage`.

**Using an Intel Mac?** Starting with SQLPage v0.47.0, the `sqlpage-macos.tgz` download is for Apple silicon (M-series Macs) only.
Use Homebrew on Intel Macs. If you previously used the downloaded executable, keep your SQL files, database, and `sqlpage` configuration folder in place and run `sqlpage` from the same website folder instead of `./sqlpage.bin`.
If SQLPage is already installed through Homebrew, use the upgrade commands above.

Homebrew may compile SQLPage and its dependencies from source on Intel Macs, so installation can take longer.
Source builds require Apple's Command Line Tools, which you can install with `xcode-select --install`.
Homebrew classifies Intel Macs as Tier 3 (limited support); older macOS versions also have restrictions. Check [Homebrew's macOS requirements](https://docs.brew.sh/Installation#macos-requirements) if installation fails.

> **Note**: Advanced users can alternatively install SQLPage using
> [the precompiled binaries for Apple silicon](https://github.com/sqlpage/SQLPage/releases/latest),
> [docker](https://hub.docker.com/repository/docker/lovasoa/SQLPage/general),
> [nix](https://search.nixos.org/packages?channel=unstable&show=sqlpage),
> or [cargo](https://crates.io/crates/sqlpage).

> **Not on Mac OS?** See the instructions for [Windows](?os=windows#download), or for [Other Systems](?os=any#download).
