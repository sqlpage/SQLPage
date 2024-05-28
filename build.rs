use actix_rt::spawn;
use futures_util::StreamExt;
use libflate::gzip;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[actix_rt::main]
async fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    for h in [
        spawn(download_deps("sqlpage.js")),
        spawn(download_deps("sqlpage.css")),
        spawn(download_deps("tabler-icons.svg")),
        spawn(download_deps("apexcharts.js")),
        spawn(download_deps("tomselect.js")),
    ] {
        h.await.unwrap();
    }
}

/// Creates a file with inlined remote files included
async fn download_deps(filename: &str) {
    let path_in = format!("sqlpage/{}", filename);
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let path_out: PathBuf = out_dir.join(filename);
    // Generate outfile by reading infile and interpreting all comments
    // like "/* !include https://... */" as a request to include the contents of
    // the URL in the generated file.
    println!("cargo:rerun-if-changed={}", path_in);
    let original = File::open(path_in).unwrap();
    process_input_file(&path_out, original).await;
    std::fs::write(
        format!("{}.filename.txt", path_out.display()),
        hashed_filename(&path_out),
    )
    .unwrap();
}

async fn process_input_file(path_out: &Path, original: File) {
    let client = awc::ClientBuilder::new()
        .timeout(core::time::Duration::from_secs(3))
        .max_http_version(awc::http::Version::HTTP_11)
        .no_default_headers()
        .finish();
    let mut outfile = gzip::Encoder::new(File::create(path_out).unwrap()).unwrap();
    for l in BufReader::new(original).lines() {
        let line = l.unwrap();
        if line.starts_with("/* !include https://") {
            let url = line
                .trim_start_matches("/* !include ")
                .trim_end_matches(" */");
            download_url_to_opened_file(&client, url, &mut outfile).await;
            outfile.write_all(b"\n").unwrap();
        } else {
            writeln!(outfile, "{}", line).unwrap();
        }
    }
    outfile
        .finish()
        .as_result()
        .expect("Unable to write compressed frontend asset");
}

async fn download_url_to_opened_file(
    client: &awc::Client,
    url: &str,
    outfile: &mut impl std::io::Write,
) {
    let mut resp = client.get(url).send().await.unwrap_or_else(|err| {
        panic!(
            "We need to download external frontend dependencies to build the static frontend. \
                Could not download {url} \
                {err}"
        )
    });
    if resp.status() != 200 {
        panic!("Received {} status code from {}", resp.status(), url);
    }
    while let Some(b) = resp.next().await {
        let chunk = b.unwrap_or_else(|err| panic!("Failed to read data from {url}: {err}"));
        outfile
            .write_all(&chunk)
            .expect("Failed to write external frontend dependency to local file");
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
