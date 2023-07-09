use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    inline_dependencies("sqlpage/sqlpage.js", &out_dir.join("sqlpage.js"));
    inline_dependencies("sqlpage/sqlpage.css", &out_dir.join("sqlpage.css"));
}

fn inline_dependencies(path_in: &str, path_out: &Path) {
    println!("cargo:rerun-if-changed=build.rs");
    // Generate outfile by reading infile and interpreting all comments
    // like "/* !include https://... */" as a request to include the contents of
    // the URL in the generated file.
    println!("cargo:rerun-if-changed={}", path_in);
    let original = File::open(path_in).unwrap();
    process_input_file(path_out, original);
    std::fs::write(
        format!("{}.filename.txt", path_out.display()),
        hashed_filename(path_out),
    )
    .unwrap();
}

fn process_input_file(path_out: &Path, original: File) {
    let mut outfile = BufWriter::new(File::create(path_out).unwrap());
    for l in BufReader::new(original).lines() {
        let line = l.unwrap();
        if line.starts_with("/* !include https://") {
            let url = line
                .trim_start_matches("/* !include ")
                .trim_end_matches(" */");
            let resp = ureq::get(url).call().unwrap();
            let mut contents = BufReader::new(resp.into_reader());
            std::io::copy(&mut contents, &mut outfile).unwrap();
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
