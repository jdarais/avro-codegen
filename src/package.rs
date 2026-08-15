use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::{borrow::Cow, path::PathBuf};

use crate::{
    config::{self, ProjectConfig},
    datamodel::PackageInfo,
};

pub struct SchemaPackage {
    pub config: ProjectConfig,
    pub schemas: Vec<SchemaSource>,
}

pub struct SchemaSource {
    // Package that the schema belongs to
    pub package: Arc<PackageInfo>,
    // Path relative to package root
    pub path: Arc<str>,
    // Schema json string
    pub schema: Arc<str>,
}

pub fn get_package_from_project_path<'a, P: AsRef<Path>>(
    project_dir: P,
    is_external: bool,
) -> Result<SchemaPackage, anyhow::Error> {
    let canonical_project_dir = Path::new(project_dir.as_ref()).canonicalize()?;
    let config = config::read_from_toml(project_dir.as_ref())?;

    let package = Arc::new(PackageInfo {
        name: String::from(config.name.as_ref()),
        version: String::from(config.version.as_ref()),
        description: String::from(config.description.as_ref()),
        is_external,
    });

    let mut schemas: Vec<SchemaSource> = Vec::new();
    for include_path in config.include.iter() {
        let absolute_include_path = canonical_project_dir.join(include_path.as_ref());
        let absolute_include_path_str = absolute_include_path.to_str().ok_or_else(|| {
            anyhow::anyhow!("Failed to convert path {absolute_include_path:?} to utf-8")
        })?;
        let files = glob::glob(absolute_include_path_str)?;
        for f_path_res in files {
            let f_path = f_path_res?;
            let canonical_f_path = f_path.canonicalize()?;
            let relative_f_path = canonical_f_path
                .strip_prefix(&canonical_project_dir)?
                .to_owned();
            let mut f = File::open(&f_path)?;
            let file_size = f.metadata()?.len();

            let mut schema = String::with_capacity(file_size as usize);
            f.read_to_string(&mut schema)?;

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

    Ok(SchemaPackage {
        config,
        schemas,
    })
}
