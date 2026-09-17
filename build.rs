use actix_rt::spawn;
use actix_rt::time::sleep;
use libflate::gzip;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

const SERVED_ASSETS: &[(&str, &[&str])] = &[
    (
        "sqlpage.js",
        &["https://cdn.jsdelivr.net/npm/@tabler/core@1.5.0/dist/js/tabler.min.js"],
    ),
    (
        "sqlpage.css",
        &[
            "https://cdn.jsdelivr.net/npm/@tabler/core@1.5.0/dist/css/tabler.min.css",
            "https://cdn.jsdelivr.net/npm/tom-select@2.6.2/dist/css/tom-select.bootstrap5.css",
            "https://cdn.jsdelivr.net/npm/@tabler/core@1.5.0/dist/css/tabler-vendors.min.css",
        ],
    ),
    (
        "apexcharts.js",
        &["https://cdn.jsdelivr.net/npm/apexcharts@7.1.0/dist/apexcharts.min.js"],
    ),
    (
        "tomselect.js",
        &["https://cdn.jsdelivr.net/npm/tom-select@2.6.2/dist/js/tom-select.popular.min.js"],
    ),
    ("favicon.svg", &[]),
];

const ICON_SPRITE_URL: &str =
    "https://cdn.jsdelivr.net/npm/@tabler/icons-sprite@3.46.0/dist/tabler-sprite.svg";

#[actix_rt::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    let client = Rc::new(make_client());

    let mut assets = Vec::with_capacity(SERVED_ASSETS.len() + 1);
    for &(name, libraries) in SERVED_ASSETS {
        assets.push(spawn(build_served_asset(client.clone(), name, libraries)));
    }
    assets.push(spawn(download_tabler_icons(
        client.clone(),
        ICON_SPRITE_URL,
    )));
    for asset in assets {
        asset.await.unwrap();
    }
    set_odbc_rpath();
}

fn make_client() -> awc::Client {
    awc::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .no_default_headers()
        .finish()
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

async fn build_served_asset(
    client: Rc<awc::Client>,
    name: &'static str,
    libraries: &'static [&'static str],
) {
    let source = Path::new("sqlpage").join(name);
    println!("cargo:rerun-if-changed={}", source.display());

    let built = out_dir().join(name);
    let mut gzipped = gzip::Encoder::new(File::create(&built).unwrap()).unwrap();
    // docs.rs builds without network access, so the libraries are left out there.
    if std::env::var_os("DOCS_RS").is_none() {
        for url in libraries {
            copy_url_to_opened_file(&client, url, &mut gzipped).await;
            gzipped.write_all(b"\n").unwrap();
        }
    }
    std::io::copy(&mut File::open(&source).unwrap(), &mut gzipped).unwrap();
    gzipped
        .finish()
        .as_result()
        .expect("Unable to write compressed frontend asset");

    std::fs::write(
        format!("{}.filename.txt", built.display()),
        hashed_filename(&built),
    )
    .unwrap();
}

async fn copy_url_to_opened_file(client: &awc::Client, url: &str, outfile: &mut impl Write) {
    // If the file has been downloaded manually, use it
    let cached_file_path = make_url_path(url);
    if !cached_file_path.exists() {
        download_url_to_path(client, url, &cached_file_path).await;
        println!("cargo:rerun-if-changed={}", cached_file_path.display());
    }
    copy_cached_to_opened_file(&cached_file_path, outfile);
}

fn copy_cached_to_opened_file(source: &Path, outfile: &mut impl Write) {
    let reader = File::open(source).unwrap();
    let mut buf = BufReader::new(reader);
    // Not async, but performance should not really matter here
    std::io::copy(&mut buf, outfile).unwrap();
}

async fn download_url_to_path(client: &awc::Client, url: &str, path: &Path) {
    let mut attempt = 1;
    let max_attempts = 2;

    loop {
        match client.get(url).send().await {
            Ok(mut resp) => {
                assert!(
                    resp.status() == 200,
                    "Received {} status code from {}",
                    resp.status(),
                    url
                );
                let bytes = resp.body().limit(128 * 1024 * 1024).await.unwrap();
                std::fs::write(path, &bytes)
                    .expect("Failed to write external frontend dependency to local file");
                break;
            }
            Err(err) => {
                if attempt >= max_attempts {
                    let path = make_url_path(url).display().to_string();
                    panic!(
                        "We need to download external frontend dependencies to build the static frontend. \n\
                        Could not download static asset after {max_attempts} attempts. You can manually download the file with: \n\
                        curl {url:?} > {path:?} \n\
                        {err}"
                    );
                }
                sleep(Duration::from_secs(1)).await;
                eprintln!("Retrying download of {url} after {err}.");
                attempt += 1;
            }
        }
    }
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

fn make_url_path(url: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let sqlpage_artefacts = Path::new(&manifest_dir)
        .join("target")
        .join("sqlpage_artefacts");
    std::fs::create_dir_all(&sqlpage_artefacts).unwrap();
    let filename = url.replace(
        |c: char| !c.is_ascii_alphanumeric() && !['.', '-'].contains(&c),
        "_",
    );
    sqlpage_artefacts.join(filename)
}

async fn download_tabler_icons(client: Rc<awc::Client>, sprite_url: &str) {
    let icon_map_path = out_dir().join("icons.rs");
    let mut sprite_content = Vec::with_capacity(3 * 1024 * 1024);
    copy_url_to_opened_file(&client, sprite_url, &mut sprite_content).await;
    let mut file = File::create(icon_map_path).unwrap();
    file.write_all(b"[").unwrap();
    extract_icons_from_sprite(&sprite_content, |name, content| {
        writeln!(file, "({name:?}, r#\"{content}\"#),").unwrap();
    });
    file.write_all(b"]").unwrap();
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
