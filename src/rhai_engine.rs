use std::boxed::Box;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;

use rhai::{Engine, EvalAltResult};

use crate::datamodel::{PackageInfo, SchemaInfo};
use crate::generator::Generator;

pub struct GeneratorContext {
    output_dir: PathBuf,
    generator: Arc<Generator>,
    schemas: Vec<SchemaInfo>,
    package: PackageInfo,
    params: serde_json::Map<String, serde_json::Value>
}

impl GeneratorContext {
    pub fn new(output_dir: PathBuf, generator: Arc<Generator>, schemas: Vec<SchemaInfo>, package: PackageInfo, params: serde_json::Map<String, serde_json::Value>) -> GeneratorContext {
        GeneratorContext {
            output_dir: output_dir,
            generator: generator,
            schemas: schemas,
            package: package,
            params: params
        }
    }
}

pub fn render(generator_context: &GeneratorContext, template: &str, output: &str, params_opt: Option<rhai::Map>) -> Result<(), Box<EvalAltResult>> {
    let params = params_opt.unwrap_or(rhai::Map::new());
    
    let mut context_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    context_map.insert(String::from("schemas"), serde_json::to_value(&generator_context.schemas).map_err(|e| format!("{e}"))?);
    context_map.insert(String::from("package"), serde_json::to_value(&generator_context.package).map_err(|e| format!("{e}"))?);
    
    let mut params_json = generator_context.params.clone();
    for (k, v) in params {
        params_json.insert(String::from(k), serde_json::to_value(v).map_err(|e| format!("{e}"))?);
    }
    context_map.insert(String::from("params"), serde_json::Value::Object(params_json));

    let tera = generator_context.generator.tera.lock().unwrap();
    let context = tera::Context::from_value(serde_json::Value::Object(context_map)).map_err(|e|format!("{e}"))?;
    let rendered = (&*tera).render(template, &context).map_err(|e| format!("{e:?}"))?;

    let output_file = generator_context.output_dir.join(output);
    if let Some(d) = output_file.parent() {
        create_dir_all(d).map_err(|e| format!("{e}"))?;
    }

    std::fs::write(output_file, rendered).map_err(|e| format!("{e}"))?;

    Ok(())
}

pub fn create_rhai_env(context: GeneratorContext) -> Engine {
    let mut engine = Engine::new();

    engine.register_fn("render", move |template: &str, output: &str, params: rhai::Map| -> Result<(), Box<EvalAltResult>> {
        render(&context, template, output, Some(params))
    });


    engine
}
