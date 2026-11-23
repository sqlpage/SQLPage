-- Positions are local to each operating system. The first method is its default.
create view install_option as
with downloads(os, position, archive, download_label, notes) as (
    values
        ('macos', 2, 'sqlpage-macos.tgz', 'Download for Apple silicon',
         'Extract the archive in your website folder, then run `./sqlpage.bin` in Terminal. On an Intel Mac, [use Homebrew](?os=macos&method=homebrew#download).'),
        ('windows', 1, 'sqlpage-windows.zip', 'Download for Windows x64',
         'Extract the ZIP into your website folder. Keep the `sqlpage` folder alongside `sqlpage.exe`, then double-click the executable to start your site.'),
        ('linux', 1, 'sqlpage-linux.tgz', 'Download for Linux x86-64',
         'Extract the archive in your website folder, then run `./sqlpage.bin` there. For an ARM device such as a Raspberry Pi, [use Docker](?os=linux&method=docker#download).')
),
packages(method, label, command, notes) as (
    values
        ('homebrew', 'Homebrew', 'brew install sqlpage',
         'Run `sqlpage` in your website folder to start it. If you don’t have `brew` yet, [install Homebrew](https://brew.sh/).'),
        ('scoop', 'Scoop', 'scoop install sqlpage',
         'Run this in PowerShell with [Scoop](https://scoop.sh/) installed. Then run `sqlpage` in your website folder.'),
        ('docker', 'Docker',
         'docker run --rm -p 127.0.0.1:8080:8080 -v .:/var/www -e SQLPAGE_CONFIGURATION_DIRECTORY=sqlpage lovasoa/sqlpage',
         'Run this from your website folder with [Docker](https://docs.docker.com/get-docker/) running. Open [localhost:8080](http://localhost:8080). Your files and database stay in that folder.'),
        ('nix', 'Nix', 'nix run nixpkgs#sqlpage',
         'Run this from your website folder to start SQLPage. Requires [Nix](https://nixos.org/download/) with `nix-command` and `flakes` enabled.'),
        ('cargo', 'Cargo', 'cargo install --locked sqlpage',
         'Builds from source with [Rust](https://rustup.rs/). After installation, run `sqlpage` in your website folder.')
),
package_order(os, method, position, caveat) as (
    values
        ('macos',   'homebrew', 1, 'Intel Macs may need [Command Line Tools](https://docs.brew.sh/Installation#macos-requirements) to build from source.'),
        ('macos',   'docker',   3, null),
        ('macos',   'nix',      4, null),
        ('macos',   'cargo',    5, 'Install unixODBC first with `brew install unixodbc`.'),
        ('windows', 'scoop',    2, null),
        ('windows', 'docker',   3, 'Use Linux containers in Docker Desktop.'),
        ('windows', 'cargo',    4, null),
        ('linux',   'docker',   2, 'The container runs as user 1000 and needs write access to the folder.'),
        ('linux',   'nix',      3, null),
        ('linux',   'homebrew', 4, null),
        ('linux',   'cargo',    5, 'Install the unixODBC development package before building.')
)
select os, 'download' as method, 'Direct download' as label, position, null as command,
    'https://github.com/sqlpage/SQLPage/releases/latest/download/' || archive as download_url,
    download_label, notes
from downloads
union all
select os, method, label, position, command, null, null,
    notes || coalesce(' ' || caveat, '')
from package_order join packages using (method);
