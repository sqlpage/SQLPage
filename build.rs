use actix_rt::spawn;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

#[actix_rt::main]
async fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    for h in [
        spawn(download_deps(
            "sqlpage/sqlpage.js",
            out_dir.join("sqlpage.js"),
        )),
        spawn(download_deps(
            "sqlpage/sqlpage.css",
            out_dir.join("sqlpage.css"),
        )),
        spawn(download_deps(
            "sqlpage/tabler-icons.svg",
            out_dir.join("tabler-icons.svg"),
        )),
    ] {
        h.await.unwrap();
    }
}

/// Creates a file with inlined remote files included
async fn download_deps(path_in: &str, path_out: PathBuf) {
    println!("cargo:rerun-if-changed=build.rs");
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
    let client = awc::Client::default();
    let mut outfile = BufWriter::new(File::create(path_out).unwrap());
    for l in BufReader::new(original).lines() {
        let line = l.unwrap();
        if line.starts_with("/* !include https://") {
            let url = line
                .trim_start_matches("/* !include ")
                .trim_end_matches(" */");
            let mut resp = client.get(url).send().await.expect(
                "We need to download external frontend dependencies to build the static frontend.",
            );
            let body = resp
                .body()
                .await
                .expect("Failed to read external frontend dependency");
            outfile
                .write_all(&body)
                .expect("Failed to write external frontend dependency to local file");
            outfile.write_all(b"\n").unwrap();
        } else {
            writeln!(outfile, "{}", line).unwrap();
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
