# Install SQLPage on macOS

For a quick start on an Apple silicon Mac (M-series), download SQLPage below.
If you already use [Homebrew](https://brew.sh/) or want easier updates, use the Homebrew instructions instead.
On an Intel Mac, use Homebrew.

## Download SQLPage for Apple silicon

[Download SQLPage for Apple silicon](https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage-macos.tgz).
Open the downloaded archive to extract it. Keep `sqlpage.bin` and the `sqlpage` folder together in your website folder.
Open Terminal in that folder and run:

```sh
./sqlpage.bin
```

macOS may ask you to confirm opening SQLPage the first time.
To update later, download the latest version and replace `sqlpage.bin`. Keep your SQL files, database, and `sqlpage` configuration folder.

Starting with SQLPage v0.47.0, this download is for Apple silicon only. On an Intel Mac, use Homebrew below.

## Install with Homebrew

If you do not already have Homebrew, open Terminal and install it:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the installer's **Next steps** to add Homebrew to your PATH. Then install SQLPage:

```sh
brew install sqlpage
```

Open Terminal in your website folder and run `sqlpage` to start your website.
To update SQLPage later, run `brew update` followed by `brew upgrade sqlpage`.

If you previously used the downloaded executable, keep your SQL files, database, and `sqlpage` configuration folder in place and run `sqlpage` from the same website folder instead of `./sqlpage.bin`.

Homebrew may compile SQLPage and its dependencies from source on Intel Macs, so installation can take longer.
Source builds require Apple's Command Line Tools, which you can install with `xcode-select --install`.
Homebrew classifies Intel Macs as Tier 3 (limited support); older macOS versions also have restrictions. Check [Homebrew's macOS requirements](https://docs.brew.sh/Installation#macos-requirements) if installation fails.

> **Note**: Advanced users can alternatively install SQLPage using
> [docker](https://hub.docker.com/repository/docker/lovasoa/SQLPage/general),
> [nix](https://search.nixos.org/packages?channel=unstable&show=sqlpage),
> or [cargo](https://crates.io/crates/sqlpage).

> **Not on Mac OS?** See the instructions for [Windows](?os=windows#download), or for [Other Systems](?os=any#download).
