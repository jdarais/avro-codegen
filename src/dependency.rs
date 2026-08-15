use std::path::PathBuf;

#[derive(Clone)]
pub enum DependencySpec {
    Version(String),
    Path(PathBuf),
}

fn is_path_dependency(value: &toml::map::Map<String, toml::Value>) -> bool {
    value.contains_key("path") && value.get("path").map_or(false, |v| v.is_str())
}

pub fn dependency_spec_from_toml(value: &toml::Value) -> Result<DependencySpec, anyhow::Error> {
    match value {
        toml::Value::String(s) => Ok(DependencySpec::Version(s.clone())),
        toml::Value::Table(t) => {
            if is_path_dependency(&t) {
                let path = PathBuf::from(t["path"].to_string());
                Ok(DependencySpec::Path(path))
            } else {
                Err(anyhow::anyhow!("Unknown dependency type"))
            }
        }
        _ => Err(anyhow::anyhow!("Invalid dependency type. Expected a string or table."))
    }
}

/*

pub enum DependencyLocator {
    Path(PathBuf),
}

pub struct DependencyNode {
    config: Arc<ProjectConfig>,
    dependencies: Vec<DependencyLocator>,
}

fn visit_dependency(config: Arc<ProjectConfig>, dep_map: &mut HashMap<String, Arc<ProjectConfig>>) {
    dep_map.insert(String::from(config.name.as_ref()), config.clone());

    for (dep_name, dep_spec) in config.dependencies.iter() {

    }
}

pub fn traverse_dependencies(config: Arc<ProjectConfig>) -> HashMap<String, Arc<ProjectConfig>> {
    let mut dep_map: HashMap<String, Arc<ProjectConfig>> = HashMap::new();
    dep_map.insert(String::from(config.name.as_ref()), config.clone());



    dep_map
}

*/
