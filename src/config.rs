// Avro-Codegen
// Copyright (C) 2026 Jeremiah Darais
//
// This program is licensed under the GPLv3.0 license (https://github.com/jdarais/cobble/blob/main/COPYING)

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use anyhow::anyhow;
use serde::Deserialize;

use crate::dependency::{DependencySpec, dependency_spec_from_toml};

#[derive(Deserialize, Debug, Clone)]
pub struct GeneratorConfig {
    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub params: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone)]
pub struct ProjectConfig {
    pub name: Arc<str>,
    pub version: Arc<str>,
    pub description: Arc<str>,
    pub include: Vec<Arc<str>>,
    pub default_generators: Vec<Arc<str>>,
    pub dependencies: HashMap<Arc<str>, DependencySpec>,
    pub generator_configs: HashMap<Arc<str>, GeneratorConfig>,
}

#[derive(Deserialize, Debug)]
struct ProjectConfigToml {
    pub name: String,
    pub version: String,
    pub description: String,
    pub include: Vec<String>,
    pub default_generators: Vec<String>,

    #[serde(default)]
    pub dependencies: HashMap<String, toml::Value>,

    #[serde(default)]
    pub generators: HashMap<String, GeneratorConfig>,
}

pub fn read_from_toml<P: AsRef<Path>>(project_dir: P) -> anyhow::Result<ProjectConfig> {
    let project_config_path = project_dir.as_ref().join("avro_codegen.toml");
    if !project_config_path.is_file() {
        return Err(anyhow!(
            "No project config file found at: {}",
            project_config_path.display()
        ));
    }

    let mut f = File::open(project_config_path)?;
    let f_size = f.metadata()?.len();

    let mut project_config_toml = String::with_capacity(f_size as usize);
    f.read_to_string(&mut project_config_toml)?;
    let project_config: ProjectConfigToml = toml::from_str(project_config_toml.as_str())?;

    let ProjectConfigToml {
        name: name_toml,
        version: version_toml,
        description: description_toml,
        include: include_toml,
        default_generators: default_generators_toml,
        dependencies: dependencies_toml,
        generators: generators_toml,
    } = project_config;

    let name: Arc<str> = name_toml.into();
    let version: Arc<str> = version_toml.into();
    let description: Arc<str> = description_toml.into();

    let incl: Vec<Arc<str>> = include_toml.into_iter().map(String::into).collect();

    let default_generators: Vec<Arc<str>> = default_generators_toml
        .into_iter()
        .map(String::into)
        .collect();

    let mut dependencies: HashMap<Arc<str>, DependencySpec> = HashMap::new();
    for (k, v) in dependencies_toml.iter() {
        let dep = dependency_spec_from_toml(v)?;
        dependencies.insert(k.clone().into(), dep);
    }

    let generator_configs: HashMap<Arc<str>, GeneratorConfig> = generators_toml
        .into_iter()
        .map(|(k, v)| (k.into(), v))
        .collect();

    Ok(ProjectConfig {
        name,
        version,
        description,
        include: incl,
        default_generators,
        dependencies,
        generator_configs,
    })
}
