// Avro-Codegen
// Copyright (C) 2026 Jeremiah Darais
//
// This program is licensed under the GPLv3.0 license (https://github.com/jdarais/cobble/blob/main/COPYING)

mod config;
mod datamodel;
mod generator;
mod lua_env;
mod tera_env;

use std::collections::{BTreeMap, HashMap};
use std::env::{set_current_dir, current_dir};
use std::fs::{remove_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};
use std::sync::Arc;

use apache_avro::schema::{DecimalSchema, InnerDecimalSchema, Schema, UuidSchema};
use clap::{Parser, Subcommand};
use mlua::LuaSerdeExt;

use crate::config::GeneratorConfig;
use crate::datamodel::{schema_to_json, PackageInfo, SchemaInfo};
use crate::generator::{Generator, INTERNAL_GENERATOR_NAMES, get_generator};
use crate::lua_env::{create_lua_env, GeneratorContext};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run code generation
    Generate {
        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: Arc<str>,

        /// Schema project directory to process
        #[arg(short, long, default_value = ".")]
        project_dir: Arc<str>,

        /// Generator to use. Can be specified multiple times
        #[arg(short, long)]
        generator: Vec<Arc<str>>
    },

    /// Display information about a code generator
    Show {
        /// Generator name
        #[arg()]
        generator: Arc<str>,

        /// Schema project directory.  If the specified directory is not a schema project, internal generators can still be shown
        #[arg(short, long, default_value = ".")]
        project_dir: Arc<str>,
    },

    /// List available generators
    List {
        /// Schema project directory.  If the specified directory is not a schema project, internal generators will be listed
        #[arg(short, long, default_value = ".")]
        project_dir: Arc<str>,
    },
}

fn collect_schemas(
    schema_collection: &mut BTreeMap<String, SchemaInfo>,
    schema: &Schema,
    schema_info: &SchemaInfo,
) {
    match schema {
        Schema::Enum(sch) => {
            let fullname = sch.name.fullname(None);
            schema_collection.insert(
                fullname.clone(),
                SchemaInfo {
                    name: sch.name.name().to_string(),
                    namespace: sch
                        .name
                        .namespace()
                        .map(str::to_string)
                        .unwrap_or_else(String::new),
                    full_name: fullname,
                    file_path: schema_info.file_path.clone(),
                    schema: schema.clone(),
                },
            );
        }
        Schema::Fixed(sch) => {
            let fullname = sch.name.fullname(None);
            schema_collection.insert(
                fullname.clone(),
                SchemaInfo {
                    name: sch.name.name().to_string(),
                    namespace: sch
                        .name
                        .namespace()
                        .map(str::to_string)
                        .unwrap_or_else(String::new),
                    full_name: fullname,
                    file_path: schema_info.file_path.clone(),
                    schema: schema.clone(),
                },
            );
        }
        Schema::Decimal(DecimalSchema {
            inner: InnerDecimalSchema::Fixed(fixed),
            ..
        })
        | Schema::Duration(fixed)
        | Schema::Uuid(UuidSchema::Fixed(fixed)) => {
            let fullname = fixed.name.fullname(None);
            schema_collection.insert(
                fullname.clone(),
                SchemaInfo {
                    name: fixed.name.name().to_string(),
                    namespace: fixed
                        .name
                        .namespace()
                        .map(str::to_string)
                        .unwrap_or_else(String::new),
                    full_name: fullname,
                    file_path: schema_info.file_path.clone(),
                    schema: schema.clone(),
                },
            );
        }
        Schema::Union(sch) => {
            for variant in sch.variants() {
                collect_schemas(&mut *schema_collection, variant, schema_info);
            }
        }
        Schema::Record(sch) => {
            let fullname = sch.name.fullname(None);
            schema_collection.insert(
                fullname.clone(),
                SchemaInfo {
                    name: sch.name.name().to_string(),
                    namespace: sch
                        .name
                        .namespace()
                        .map(str::to_string)
                        .unwrap_or_else(String::new),
                    full_name: fullname,
                    file_path: schema_info.file_path.clone(),
                    schema: schema.clone(),
                },
            );
            for field in sch.fields.iter() {
                collect_schemas(&mut *schema_collection, &field.schema, schema_info);
            }
        }
        Schema::Decimal(sch) => match &sch.inner {
            InnerDecimalSchema::Fixed(fixed_sch) => {
                let fullname = fixed_sch.name.fullname(None);
                schema_collection.insert(
                    fullname.clone(),
                    SchemaInfo {
                        name: fixed_sch.name.name().to_string(),
                        namespace: fixed_sch.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                        full_name: fullname,
                        file_path: schema_info.file_path.clone(),
                        schema: schema.clone()
                    }
                );
            }
            _ => { /* Nothing to do */ }
        }
        _ => { /* Nothing to do */ }
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Command::Generate {
            output,
            project_dir,
            generator
        } => {
            let canonical_project_dir = Path::new(project_dir.as_ref()).canonicalize().unwrap();
            set_current_dir(&canonical_project_dir).unwrap();
            let cfg = config::read_from_toml(&current_dir().unwrap()).unwrap();
            let mut schema_paths: Vec<PathBuf> = Vec::new();
            let mut schema_strings: Vec<String> = Vec::new();
            for include_path in cfg.include.iter() {
                let files = glob::glob(include_path.as_ref()).unwrap();
                for f_path_res in files {
                    let f_path = f_path_res.unwrap();
                    let canonical_f_path = f_path.canonicalize().unwrap();
                    let relative_f_path = canonical_f_path
                        .strip_prefix(&canonical_project_dir)
                        .unwrap()
                        .to_owned();
                    let mut f = File::open(&f_path).unwrap();
                    let file_size = f.metadata().unwrap().len();

                    let mut schema = String::with_capacity(file_size as usize);
                    f.read_to_string(&mut schema).unwrap();

                    schema_paths.push(relative_f_path);
                    schema_strings.push(schema);
                }
            }

            let schema_strs: Vec<&str> = schema_strings.iter().map(|s| s.as_str()).collect();
            let schemas = Schema::parse_list(&schema_strs[..]).unwrap();
            let mut schema_infos: Vec<SchemaInfo> = Vec::new();
            for (path, schema) in schema_paths.iter().zip(schemas.iter()) {
                let name = schema.name().map(|n| n.name().to_string());
                let namespace = schema.namespace();
                let full_name = match &name {
                    Some(nm) => match &namespace {
                        Some(ns) => {
                            let mut name_accum = String::with_capacity(ns.len() + 1 + nm.len());
                            name_accum.push_str(ns);
                            name_accum.push('.');
                            name_accum.push_str(nm);
                            Some(name_accum)
                        }
                        None => Some(nm.clone()),
                    },
                    None => None,
                };

                schema_infos.push(SchemaInfo {
                    name: name.unwrap_or_else(|| String::new()),
                    namespace: namespace.map(str::to_string).unwrap_or_else(|| String::new()),
                    full_name: full_name.unwrap_or_else(|| String::new()),
                    // TODO: Always provide unix-style path regardless fo platform
                    file_path: String::from(path.as_os_str().to_str().unwrap())
                        .replace(MAIN_SEPARATOR_STR, "/"),
                    schema: schema.clone(),
                });
            }

            let mut all_schemas: BTreeMap<String, SchemaInfo> = BTreeMap::new();
            for schema_info in schema_infos.iter() {
                collect_schemas(&mut all_schemas, &schema_info.schema, &schema_info);
            }

            let mut all_schemas_json: Vec<serde_json::Value> = Vec::new();
            for (_, v) in &all_schemas {
                all_schemas_json.push(schema_to_json(&v.schema, v, schemas.as_slice()).unwrap());
            }
            let all_schemas_json = all_schemas_json;

            let generator_names = if generator.is_empty() { cfg.default_generators.clone() } else { generator };

            let mut generators: Vec<(Arc<str>, Arc<Generator>)> = Vec::new();
            for gen_name in generator_names.iter() {
                let generator = get_generator(gen_name, &cfg.generator_configs).unwrap();
                generators.push((gen_name.clone(), Arc::new(generator)));
            }

            let package_info = PackageInfo {
                name: String::from(cfg.name.as_ref()),
                version: String::from(cfg.version.as_ref()),
                description: String::from(cfg.description.as_ref()),
            };

            for (generator_name, generator) in generators.iter() {
                let generator_output_dir = Path::new(output.as_ref()).join(generator_name.as_ref());

                if generator_output_dir.exists() {
                    remove_dir_all(&generator_output_dir).unwrap();
                }

                let mut params: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for (name, param) in &generator.params {
                    let val = serde_json::to_value(param.default.clone()).unwrap();
                    params.insert(String::from(name.as_ref()), val);
                }

                if let Some(generator_config) = cfg.generator_configs.get(generator_name.as_ref()) {
                    for (name, param) in &generator_config.params {
                        params.insert(name.clone(), param.clone());
                    }
                }

                let lua = create_lua_env(GeneratorContext::new(
                    generator_output_dir,
                    generator.clone(),
                    all_schemas_json.clone(),
                    package_info.clone(),
                    params,
                ))
                .unwrap();

                lua.globals()
                    .set(
                        "schemas",
                        lua.to_value(&serde_json::to_value(&all_schemas_json).unwrap())
                            .unwrap(),
                    )
                    .unwrap();
                lua.globals()
                    .set(
                        "package",
                        lua.to_value(&serde_json::to_value(&package_info).unwrap())
                            .unwrap(),
                    )
                    .unwrap();

                lua.load(generator.generate_script.as_ref())
                    .set_name("@generate.lua")
                    .exec()
                    .unwrap();
            }
        }
        Command::Show { generator, project_dir } => {
            let canonical_project_dir = Path::new(project_dir.as_ref()).canonicalize().unwrap();
            set_current_dir(&canonical_project_dir).unwrap();
            let cfg_res = config::read_from_toml(&current_dir().unwrap());

            let no_generator_configs: HashMap<Arc<str>, GeneratorConfig> = HashMap::new();
            let generator_configs = match cfg_res {
                Ok(ref cfg) => &cfg.generator_configs,
                Err(e) => {
                    eprintln!("Warning: unable to find or read avro_codegen.toml project file.  Only internal generators can be shown. ({})", e);
                    &no_generator_configs
                }
            };
            
            let gen_res = get_generator(generator.as_ref(), generator_configs);
            let generator_info = match gen_res {
                Ok(g) => g,
                Err(e) => {
                    panic!("Error getting generator '{generator}': {e}");
                }
            };

            println!("ID: {generator}");
            println!("Name: {}", generator_info.name);
            println!("Description: {}", generator_info.description);
            println!("");
            println!("Params:");

            for (name, param) in &generator_info.params {
                println!(
                    "  {}: {} (default={})",
                    name, param.description, param.default
                );
            }
        }
        Command::List { project_dir} => {
            let canonical_project_dir = Path::new(project_dir.as_ref()).canonicalize().unwrap();
            set_current_dir(&canonical_project_dir).unwrap();
            let cfg_res = config::read_from_toml(&current_dir().unwrap());

            let mut generator_names: Vec<Arc<str>> = match cfg_res {
                Ok(cfg) => cfg.generator_configs.keys().cloned().collect(),
                Err(e) => {
                    eprintln!("Warning: unable to find or read avro_codegen.toml project file.  Only internal generators will be listed. ({})", e);
                    Vec::new()
                }
            };

            for gen in INTERNAL_GENERATOR_NAMES {
                if let None = generator_names.iter().find(|s| s.as_ref() == gen) {
                    generator_names.push(Arc::from(gen.to_owned()));
                }
            }
            
            for gen in generator_names {
                println!("{}", gen);
            }

        }
    };
}
