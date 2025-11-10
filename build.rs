use actix_rt::spawn;
use actix_rt::time::sleep;
use libflate::gzip;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

#[actix_rt::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    let c = Rc::new(make_client());

    for h in [
        spawn(download_deps(c.clone(), "sqlpage.js")),
        spawn(download_deps(c.clone(), "sqlpage.css")),
        spawn(download_tabler_icons(
            c.clone(),
            "https://cdn.jsdelivr.net/npm/@tabler/icons-sprite@3.35.0/dist/tabler-sprite.svg",
        )),
        spawn(download_deps(c.clone(), "apexcharts.js")),
        spawn(download_deps(c.clone(), "tomselect.js")),
        spawn(download_deps(c.clone(), "favicon.svg")),
    ] {
        h.await.unwrap();
    }
    set_odbc_rpath();
}

fn make_client() -> awc::Client {
    awc::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .no_default_headers()
        .finish()
}

/// Creates a file with inlined remote files included
async fn download_deps(client: Rc<awc::Client>, filename: &str) {
    let path_in = format!("sqlpage/{filename}");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let path_out: PathBuf = out_dir.join(filename);
    // Generate outfile by reading infile and interpreting all comments
    // like "/* !include https://... */" as a request to include the contents of
    // the URL in the generated file.
    println!("cargo:rerun-if-changed={path_in}");
    let original = File::open(path_in).unwrap();
    process_input_file(&client, &path_out, original).await;
    std::fs::write(
        format!("{}.filename.txt", path_out.display()),
        hashed_filename(&path_out),
    )
    .unwrap();
}

async fn process_input_file(client: &awc::Client, path_out: &Path, original: File) {
    let mut outfile = gzip::Encoder::new(File::create(path_out).unwrap()).unwrap();
    for l in BufReader::new(original).lines() {
        let line = l.unwrap();
        if line.starts_with("/* !include https://") {
            let url = line
                .trim_start_matches("/* !include ")
                .trim_end_matches(" */");
            if std::env::var("DOCS_RS").is_err() {
                copy_url_to_opened_file(client, url, &mut outfile).await;
            } else {
                println!("cargo:warning=Skipping download of {url} because we're building docs.");
                return;
            }
            outfile.write_all(b"\n").unwrap();
        } else {
            writeln!(outfile, "{line}").unwrap();
        }
    }
    outfile
        .finish()
        .as_result()
        .expect("Unable to write compressed frontend asset");
}

async fn copy_url_to_opened_file(
    client: &awc::Client,
    url: &str,
    outfile: &mut impl std::io::Write,
) {
    // If the file has been downloaded manually, use it
    let cached_file_path = make_url_path(url);
    if !cached_file_path.exists() {
        println!("cargo:warning=Downloading {url} to cache file {cached_file_path:?}.");
        download_url_to_path(client, url, &cached_file_path).await;
        println!("cargo:rerun-if-changed={}", cached_file_path.display());
    }
    copy_cached_to_opened_file(&cached_file_path, outfile);
}

fn copy_cached_to_opened_file(source: &Path, outfile: &mut impl std::io::Write) {
    let reader = std::fs::File::open(source).unwrap();
    let mut buf = std::io::BufReader::new(reader);
    // Not async, but performance should not really matter here
    std::io::copy(&mut buf, outfile).unwrap();
}

async fn download_url_to_path(client: &awc::Client, url: &str, path: &Path) {
    let mut attempt = 1;
    let max_attempts = 2;

    loop {
        match client.get(url).send().await {
            Ok(mut resp) => {
                if resp.status() != 200 {
                    panic!("Received {} status code from {}", resp.status(), url);
                }
                let bytes = resp.body().limit(128 * 1024 * 1024).await.unwrap();
                std::fs::write(path, &bytes)
                    .expect("Failed to write external frontend dependency to local file");
                break;
            }
            Err(err) => {
                if attempt >= max_attempts {
                    let path = make_url_path(url);
                    panic!(
                        "We need to download external frontend dependencies to build the static frontend. \n\
                        Could not download static asset after {max_attempts} attempts. You can manually download the file with: \n\
                        curl {url:?} > {path:?} \n\
                        {err}"
                    );
                }
                sleep(Duration::from_secs(1)).await;
                println!("cargo:warning=Retrying download of {url} after {err}.");
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
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let icon_map_path = out_dir.join("icons.rs");
    let mut sprite_content = Vec::with_capacity(3 * 1024 * 1024);
    copy_url_to_opened_file(&client, sprite_url, &mut sprite_content).await;
    let mut file = File::create(icon_map_path).unwrap();
    file.write_all(b"[").unwrap();
    extract_icons_from_sprite(&sprite_content, |name, content| {
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
