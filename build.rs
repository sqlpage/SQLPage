use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

const ICON_SPRITE: &str = "frontend/dist/tabler-sprite.svg";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_icon_map();
    set_odbc_rpath();
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

fn build_icon_map() {
    println!("cargo:rerun-if-changed={ICON_SPRITE}");
    let mut sprite = Vec::with_capacity(3 * 1024 * 1024);
    File::open(ICON_SPRITE)
        .unwrap_or_else(|err| {
            panic!(
                "Unable to read {ICON_SPRITE}: {err}\n\
                 The browser assets are built by npm: \
                 run `npm ci && npm run build` before `cargo build`."
            )
        })
        .read_to_end(&mut sprite)
        .unwrap();
    let mut icon_map = File::create(out_dir().join("icons.rs")).unwrap();
    icon_map.write_all(b"[").unwrap();
    extract_icons_from_sprite(&sprite, |name, content| {
        writeln!(icon_map, "({name:?}, r#\"{content}\"#),").unwrap();
    });
    icon_map.write_all(b"]").unwrap();
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
