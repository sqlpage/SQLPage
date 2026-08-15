use libflate::gzip;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Assets the server embeds and serves, built by `npm run build`.
const SERVED_ASSETS: [&str; 5] = [
    "sqlpage.js",
    "apexcharts.js",
    "tomselect.js",
    "sqlpage.css",
    "favicon.svg",
];

const ICON_SPRITE: &str = "tabler-sprite.svg";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let dist = Path::new("frontend").join("dist");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    for filename in SERVED_ASSETS {
        compress_asset(&read_built_asset(&dist, filename), &out_dir, filename);
    }
    write_icon_map(&read_built_asset(&dist, ICON_SPRITE), &out_dir);

    set_odbc_rpath();
}

fn read_built_asset(dist: &Path, filename: &str) -> Vec<u8> {
    let path = dist.join(filename);
    println!("cargo:rerun-if-changed={}", path.display());
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "Unable to read the frontend asset '{}': {e}\n\
            The frontend is built by its own toolchain. Run: \n\
            npm ci && npm run build",
            path.display()
        )
    })
}

/// Serves each asset pre-compressed, under a name that changes with its contents.
fn compress_asset(contents: &[u8], out_dir: &Path, filename: &str) {
    let path_out = out_dir.join(filename);
    let mut outfile = gzip::Encoder::new(File::create(&path_out).unwrap()).unwrap();
    outfile.write_all(contents).unwrap();
    outfile
        .finish()
        .as_result()
        .expect("Unable to write compressed frontend asset");
    std::fs::write(
        format!("{}.filename.txt", path_out.display()),
        hashed_filename(&path_out),
    )
    .unwrap();
}

// Given a filename, creates a new unique filename based on the file contents
fn hashed_filename(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut buf = [0u8; 4096];
    let mut hasher = DefaultHasher::new();
    loop {
        let bytes_read = file
            .read(&mut buf)
            .unwrap_or_else(|e| panic!("error reading '{}': {}", path.display(), e));
        if bytes_read == 0 {
            break;
        }
        hasher.write(&buf[..bytes_read]);
    }
    let hash = hasher.finish();
    format!(
        "{}.{:x}.{}",
        path.file_stem().unwrap().to_str().unwrap(),
        hash,
        path.extension().unwrap().to_str().unwrap()
    )
}

fn write_icon_map(sprite_content: &[u8], out_dir: &Path) {
    let mut file = File::create(out_dir.join("icons.rs")).unwrap();
    file.write_all(b"[").unwrap();
    extract_icons_from_sprite(sprite_content, |name, content| {
        writeln!(file, "({name:?}, r#\"{content}\"#),").unwrap();
    });
    file.write_all(b"]").unwrap();
}

fn extract_icons_from_sprite(sprite_content: &[u8], mut callback: impl FnMut(&str, &str)) {
    let mut sprite_str = std::str::from_utf8(sprite_content).unwrap();
    fn take_between<'a>(s: &mut &'a str, start: &str, end: &str) -> Option<&'a str> {
        let start_index = s.find(start)?;
        let end_index = s[start_index + start.len()..].find(end)?;
        let result = &s[start_index + start.len()..][..end_index];
        *s = &s[start_index + start.len() + end_index + end.len()..];
        Some(result)
    }
    while let Some(mut symbol_tag) = take_between(&mut sprite_str, "<symbol", "</symbol>") {
        let id = take_between(&mut symbol_tag, "id=\"tabler-", "\"").expect("id not found");
        let content_start = symbol_tag.find('>').unwrap() + 1;
        callback(id, &symbol_tag[content_start..]);
    }
}

/// On debian-based linux distributions, odbc drivers are installed in /usr/lib/<target>-linux-gnu/odbc
/// which is not in the default library search path.
fn set_odbc_rpath() {
    if cfg!(all(target_os = "linux", feature = "odbc-static")) {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/{}-linux-gnu/odbc",
            std::env::var("TARGET").unwrap().split('-').next().unwrap()
        );
    }
}
