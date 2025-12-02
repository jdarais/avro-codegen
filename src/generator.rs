use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::include_bytes;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use flate2::read::GzDecoder;
use glob::glob;
use tera::Tera;

use crate::tera_env::create_tera;

#[derive(Clone)]
pub struct ParamDescription {
    description: Arc<str>,
}

pub struct Generator {
    pub tera: Arc<Mutex<Tera>>,
    pub generate_script: Arc<str>,
    pub params: HashMap<Arc<str>, ParamDescription>
}

fn get_template_files_for_generator(generator_dir: &Path) -> Result<Vec<(PathBuf, Option<Arc<str>>)>, anyhow::Error> {
    let mut search_path = generator_dir.to_owned();
    search_path.push("templates");
    search_path.push("**");
    search_path.push("*");
    let search_pattern = search_path.to_str()
        .expect("File path is expected to always be valid UTF-8");
    let glob_paths = glob(search_pattern)?;

    let mut result: Vec<(PathBuf, Option<Arc<str>>)> = Vec::new();
    for glob_path in glob_paths {
        match glob_path {
            Ok(path) => {
                let template_dir = generator_dir.join("templates");
                let path_rel_to_template_dir = path.strip_prefix(template_dir)?;
                let path_rel_to_template_dir_str = path_rel_to_template_dir.to_str()
                    .expect("File path is expected to always be valid UTF-8");
                    
                result.push((path.clone(), Some(Arc::from(path_rel_to_template_dir_str))));
            }
            Err(e) => {
                println!("Unable to read directory {} ({}) Skipping...", e.path().display(), e.error());
            }
        }
    }

    Ok(result)
}

fn read_params_from_toml(params_toml: &str) -> Result<HashMap<Arc<str>, ParamDescription>, anyhow::Error> {
    let toml_value: toml::Table = params_toml.parse()?;
    let params_table = match toml_value.get("params") {
        Some(v) => match v {
            toml::Value::Table(t) => Cow::Borrowed(t),
            _ => { return Err(anyhow!("'params' attribute in params.toml is not a table")); }
        },
        None => Cow::Owned(toml::Table::new())
    };

    let mut params: HashMap<Arc<str>, ParamDescription> = HashMap::with_capacity(toml_value.len());
    for (k, v) in params_table.as_ref() {
        let param = match v {
            toml::Value::Table(t) => {
                let description = t.get("description")
                .and_then(|desc| desc.as_str())
                .map(|dstr| Arc::from(dstr))
                .unwrap_or_else(|| Arc::<str>::from(""));

                ParamDescription { description }
            }
            toml::Value::String(s) => {
                ParamDescription { description: Arc::from(s.as_str()) }
            }
            _ => { return Err(anyhow!("Invalid value for param definition: {}", v)); }
        };
        params.insert(Arc::from(k.as_str()), param);
    }

    Ok(params)
}

fn read_builtin_generator_archive(archive_data: &[u8]) -> Result<Generator, anyhow::Error> {
    let mut templates: HashMap<Arc<str>, Arc<str>> = HashMap::new();
    let mut generate_script: Option<Arc<str>> = None;
    let mut params: HashMap<Arc<str>, ParamDescription> = HashMap::new();

    let compressed_reader = GzDecoder::new(&archive_data[..]);
    let mut tar_reader = tar::Archive::new(compressed_reader);

    let mut buf = String::new();
    for entry_res in tar_reader.entries()? {
        let mut entry = entry_res?;
        let path = Arc::<str>::from(entry.path()?.to_str().unwrap());

        if path.starts_with("templates/") {
            buf.clear();
            entry.read_to_string(&mut buf)?;
            templates.insert(Arc::<str>::from(path.strip_prefix("templates/").unwrap()), Arc::from(buf.as_str()));
        } else if path.as_ref() == "params.toml" {
            buf.clear();
            entry.read_to_string(&mut buf)?;
            params = read_params_from_toml(&buf)?;
        } else if path.as_ref() == "generate.lua" {
            buf.clear();
            entry.read_to_string(&mut buf)?;
            generate_script = Some(Arc::from(buf.as_str()));
        }
    }

    println!("Adding generator with templates {:?}", templates);

    match generate_script {
        None => Err(anyhow!("No generate.lua file found")),
        Some(s) => {
            let mut tera = create_tera();
            tera.add_raw_templates(templates)?;

            Ok(Generator {
                tera: Arc::new(Mutex::new(tera)),
                generate_script: s,
                params
            })
        }
    }
}

fn create_generator_from_path(generator_dir_str: &str) -> Result<Generator, anyhow::Error> {
    let generator_dir = Path::new(generator_dir_str);
    let params_toml_path = generator_dir.join("params.toml");

    // One of these paths must exist
    let generate_script_path = generator_dir.join("generate.lua");

    if !generator_dir.is_dir() { return Err(anyhow!("Given generator directory path is not a directory: {}", generator_dir.display())); }
    if !generate_script_path.is_file() {
        return Err(anyhow!("Generator directory must contain a generate.lua file"));
    }

    let mut tera = create_tera();
    tera.add_template_files(get_template_files_for_generator(generator_dir)?)?;

    let mut generate_script_file = File::open(generate_script_path)?;
    let files_toml_content_len = generate_script_file.metadata()?.len();

    let mut generate_script_content = String::with_capacity(files_toml_content_len as usize);
    generate_script_file.read_to_string(&mut generate_script_content)?;

    let params_toml_file = File::open(params_toml_path);
    let params: HashMap<Arc<str>, ParamDescription> = match params_toml_file {
        Err(_) => HashMap::new(),
        Ok(mut f) => {
            let params_toml_content_len = f.metadata()?.len();
            let mut params_toml_content = String::with_capacity(params_toml_content_len as usize);
            f.read_to_string(&mut params_toml_content)?;
            read_params_from_toml(&params_toml_content)?                    
        }
    };

    Ok(Generator{
        tera: Arc::new(Mutex::new(tera)),
        generate_script: generate_script_content.into(),
        params
    })
}

pub fn get_generator(generator_name_or_dir: &str) -> Result<Generator, anyhow::Error> {
    match generator_name_or_dir {
        "rust" => {
            let archive_data = include_bytes!(env!("GENERATOR_ARCHIVE_PATH_rust"));
            read_builtin_generator_archive(archive_data)
        },
        _ => create_generator_from_path(generator_name_or_dir)
    }
}
