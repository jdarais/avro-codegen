// Avro-Codegen
// Copyright (C) 2026 Jeremiah Darais
//
// This program is licensed under the GPLv3.0 license (https://github.com/jdarais/cobble/blob/main/COPYING)

use assert_cmd::cargo::cargo_bin_cmd; // Import cargo_bin_cmd! macro and methods
use std::path::Path;
use std::fs;
use std::convert::AsRef;
use std::process;

fn copy_files<BP, DP>(base_path: BP, pattern: &str, dest: DP) -> Result<(), anyhow::Error>
where
    BP: AsRef<Path>,
    DP: AsRef<Path>,
{
    let base_path_abs = base_path.as_ref().canonicalize()?;
    let fileglob = glob::glob(base_path.as_ref().join(pattern).to_str().unwrap())?;

    for file_res in fileglob {
        let file = file_res?;

        if !file.is_file() {
            println!("Skipping copy of non-file: {:?}", file);
            continue;
        }

        let file_abs = file.canonicalize()?;
        let rel_path = file_abs.strip_prefix(&base_path_abs)?;

        let dest_path = dest.as_ref().join(rel_path);
        let dest_dir_opt = dest_path.parent();

        if let Some(dest_dir) = dest_dir_opt {
            fs::create_dir_all(dest_dir)?;
        }

        fs::copy(&file_abs, dest.as_ref().join(rel_path))?;
    }
    
    Ok(())
}

#[test]
fn test_rust_generator_output() -> Result<(), anyhow::Error> {
    let temp_dir = tempfile::tempdir()?;
    let temp = temp_dir.keep();

    copy_files("examples/sample_schemas", "src/**/*", &temp)?;
    copy_files("examples/sample_schemas", "test/rust/src/**/*", &temp)?;
    copy_files("examples/sample_schemas", "test/rust/Cargo.toml", &temp)?;
    copy_files("examples/sample_schemas", "test/rust/src/**/*", &temp)?;
    copy_files("examples/sample_schemas", "avro_codegen.toml", &temp)?;

    let mut cmd = cargo_bin_cmd!("avro_codegen");

    cmd.arg("generate")
        .arg("-g").arg("rust")
        .arg("-p").arg(temp.to_str().unwrap())
        .assert().success();

    assert_cmd::cmd::Command::from_std(process::Command::new(env!("CARGO")))
        .current_dir(temp.join("test/rust"))
        .arg("test")
        .assert()
        .success();

    Ok(())
}
