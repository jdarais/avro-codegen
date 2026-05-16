use std::env;
use std::ffi::OsString;
use std::fs::{read_dir, File};
use std::io::{self, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use flate2::write::GzEncoder;
use flate2::Compression;

fn build_file_list(base_path: &Path, path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    let full_path = base_path.join(path);
    if full_path.is_dir() {
        let dir_entries = read_dir(full_path)?;
        for entry_res in dir_entries {
            let entry = entry_res?;
            let entry_path = path.join(entry.file_name());
            build_file_list(base_path, entry_path.as_path(), files)?;
        }
    } else if full_path.is_file() {
        files.push(PathBuf::from(path));
    }
    Ok(())
}

fn main() {
    let out_dir =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR env var is expected to be defined"));

    let generators_dir = read_dir(Path::new("src/generators"))
        .expect("Expected to be able to read src/generators dir in project directory");

    for generator_dir_res in generators_dir {
        let generator_dir = generator_dir_res
            .expect("Expected to be able to read directories under src/generators");

        let mut file_paths: Vec<PathBuf> = Vec::new();
        build_file_list(&generator_dir.path(), Path::new(""), &mut file_paths).unwrap();

        let mut archive_filename = OsString::from(generator_dir.file_name());
        archive_filename.push(".tgz");

        let archive_path = out_dir.join(&archive_filename);
        let out_file = File::create(&archive_path).unwrap();
        let compressed_writer = GzEncoder::new(out_file, Compression::default());
        let mut tar_writer = tar::Builder::new(compressed_writer);

        for path in file_paths {
            let full_path = generator_dir.path().join(&path);
            println!("cargo::rerun-if-changed={:?}", &full_path);

            let mut f = File::open(&full_path).unwrap();
            let f_size = f.seek(SeekFrom::End(0)).unwrap();
            f.rewind().unwrap();

            let mut header = tar::Header::new_gnu();
            header.set_size(f_size);
            header.set_cksum();

            tar_writer.append_data(&mut header, &path, f).unwrap();
        }

        tar_writer.finish().unwrap();

        println!(
            "cargo::rustc-env=GENERATOR_ARCHIVE_PATH_{}={}",
            &generator_dir.file_name().into_string().unwrap(),
            &archive_path.into_os_string().into_string().unwrap()
        );
    }
}
