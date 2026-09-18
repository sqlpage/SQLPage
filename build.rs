use libflate::gzip;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

const DIST: &str = "frontend/dist";

const SERVED_ASSETS: &[&str] = &[
    "sqlpage.js",
    "sqlpage.css",
    "apexcharts.js",
    "tomselect.js",
    "favicon.svg",
];

const ICON_SPRITE: &str = "tabler-sprite.svg";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for name in SERVED_ASSETS {
        build_served_asset(name);
    }
    build_icon_map();
    set_odbc_rpath();
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

fn built_asset(name: &str) -> PathBuf {
    Path::new(DIST).join(name)
}

fn open_input(name: &str) -> File {
    let path = built_asset(name);
    println!("cargo:rerun-if-changed={}", path.display());
    File::open(&path).unwrap_or_else(|err| {
        panic!(
            "Unable to read {}: {err}\n\
             The browser assets are built by npm: \
             run `npm ci && npm run build` before `cargo build`.",
            path.display()
        )
    })
}

fn build_served_asset(name: &str) {
    let built = out_dir().join(name);
    let mut gzipped = gzip::Encoder::new(File::create(&built).unwrap()).unwrap();
    std::io::copy(&mut open_input(name), &mut gzipped).unwrap();
    gzipped
        .finish()
        .as_result()
        .expect("Unable to write compressed frontend asset");

    std::fs::write(
        format!("{}.filename.txt", built.display()),
        hashed_file_content(name),
    )
    .unwrap();
}

fn build_icon_map() {
    let mut sprite = Vec::with_capacity(3 * 1024 * 1024);
    open_input(ICON_SPRITE).read_to_end(&mut sprite).unwrap();
    let mut icon_map = File::create(out_dir().join("icons.rs")).unwrap();
    icon_map.write_all(b"[").unwrap();
    extract_icons_from_sprite(&sprite, |name, content| {
        writeln!(icon_map, "({name:?}, r#\"{content}\"#),").unwrap();
    });
    icon_map.write_all(b"]").unwrap();
}

fn hashed_file_content(name: &str) -> String {
    let path = built_asset(name);
    let mut file = File::open(&path).unwrap();
    let mut buf = [0u8; 4096];
    let mut hasher = DefaultHasher::new();
    loop {
        let bytes_read = file
            .read(&mut buf)
            .unwrap_or_else(|e| panic!("error reading '{}': {e}", path.display()));
        if bytes_read == 0 {
            break;
        }
        hasher.write(&buf[..bytes_read]);
    }
    let name = Path::new(name);
    format!(
        "{}.{:x}.{}",
        name.file_stem().unwrap().to_str().unwrap(),
        hasher.finish(),
        name.extension().unwrap().to_str().unwrap()
    )
}

fn take_between<'a>(s: &mut &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_index = s.find(start)?;
    let end_index = s[start_index + start.len()..].find(end)?;
    let result = &s[start_index + start.len()..][..end_index];
    *s = &s[start_index + start.len() + end_index + end.len()..];
    Some(result)
}

fn extract_icons_from_sprite(sprite_content: &[u8], mut callback: impl FnMut(&str, &str)) {
    let mut sprite_str = std::str::from_utf8(sprite_content).unwrap();
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
