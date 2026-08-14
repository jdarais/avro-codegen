use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::{self, ProjectConfig};

#[derive(Serialize, Deserialize, Clone)]
pub struct SchemaPackage {
    pub name: Arc<str>,
    pub version: Arc<str>,
    pub description: Arc<str>,
}

pub struct SchemaSource {
    // Package that the schema belongs to
    pub package: Arc<SchemaPackage>,
    // Path relative to package root
    pub path: Arc<str>,
    // Schema json string
    pub schema: Arc<str>,
}

pub fn get_schema_sources_from_project_path<'a, P: AsRef<Path>>(
    project_dir: P,
    project_config: Option<&'a ProjectConfig>
) -> Result<Vec<SchemaSource>, anyhow::Error> {
    let canonical_project_dir = Path::new(project_dir.as_ref()).canonicalize().unwrap();
    let cfg: Cow<'a, ProjectConfig> = match project_config {
        Some(c) => Cow::Borrowed(c),
        None => Cow::Owned(config::read_from_toml(project_dir.as_ref())?)
    };

    let package = Arc::new(SchemaPackage {
        name: cfg.name.clone(),
        version: cfg.version.clone(),
        description: cfg.description.clone(),
    });

    let mut schemas: Vec<SchemaSource> = Vec::new();
    for include_path in cfg.include.iter() {
        let absolute_include_path = canonical_project_dir.join(include_path.as_ref());
        let absolute_include_path_str = absolute_include_path.to_str().ok_or_else(|| {
            anyhow::anyhow!("Failed to convert path {absolute_include_path:?} to utf-8")
        })?;
        let files = glob::glob(absolute_include_path_str).unwrap();
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

            let relative_f_path_str = relative_f_path.to_str().ok_or_else(|| {
                anyhow::anyhow!("Failed to convert path {relative_f_path:?} to utf-8")
            })?;

            schemas.push(SchemaSource {
                package: package.clone(),
                path: Arc::<str>::from(relative_f_path_str),
                schema: schema.into(),
            });
        }
    }

    Ok(schemas)
}
