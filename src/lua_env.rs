use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;

use mlua::{Lua, LuaOptions, StdLib};

use crate::datamodel::{PackageInfo, SchemaInfo};
use crate::generator::Generator;

pub struct GeneratorContext {
    output_dir: PathBuf,
    generator: Arc<Generator>,
    schemas: Vec<SchemaInfo>,
    package: PackageInfo,
    params: serde_json::Map<String, serde_json::Value>,
}

impl GeneratorContext {
    pub fn new(
        output_dir: PathBuf,
        generator: Arc<Generator>,
        schemas: Vec<SchemaInfo>,
        package: PackageInfo,
        params: serde_json::Map<String, serde_json::Value>,
    ) -> GeneratorContext {
        GeneratorContext {
            output_dir: output_dir,
            generator: generator,
            schemas: schemas,
            package: package,
            params: params,
        }
    }
}

pub fn json_to_lua(lua: &Lua, value: &serde_json::Value) -> mlua::Result<mlua::Value> {
    match value {
        serde_json::Value::Null => Ok(mlua::Value::Nil),
        serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            Ok(mlua::Value::Number(n.as_f64().ok_or_else(|| {
                mlua::Error::runtime("Unable to convert json number to lua number")
            })?))
        }
        serde_json::Value::String(s) => Ok(mlua::Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let items = lua.create_table()?;
            for val in arr {
                items.push(json_to_lua(lua, val)?)?;
            }

            Ok(mlua::Value::Table(create_array(lua, Some(items))?))
        }
        serde_json::Value::Object(obj) => {
            let items = lua.create_table()?;
            for (k, v) in obj {
                items.set(k.clone(), json_to_lua(lua, v)?)?;
            }

            Ok(mlua::Value::Table(create_map(lua, Some(items))?))
        }
    }
}

fn create_array(lua: &Lua, init: Option<mlua::Table>) -> mlua::Result<mlua::Table> {
    let array_ctor: mlua::Function = lua.globals().get("array")?;
    array_ctor.call(init)
}

fn lua_table_looks_like_array(value: &mlua::Table) -> mlua::Result<bool> {
    for pair in value.pairs() {
        let (k, _v): (mlua::Value, mlua::Value) = pair?;
        if !k.is_number() {
            return Ok(false);
        }
    }

    return Ok(true);
}

fn lua_table_to_json_array(lua: &Lua, value: &mlua::Table) -> mlua::Result<Vec<serde_json::Value>> {
    let array_len = value.len()?;
    let mut result: Vec<serde_json::Value> = vec![serde_json::Value::Null; array_len as usize];
    for pair in value.pairs() {
        let (k, v): (i64, mlua::Value) = pair?;
        if k < 1 || k > array_len {
            return Err(mlua::Error::runtime(format!(
                "Found non-contiguous index while converting lua table to array: {k}"
            )));
        }

        result[(k as usize) - 1] = lua_to_json(lua, &v)?;
    }

    Ok(result)
}

fn create_map(lua: &Lua, init: Option<mlua::Table>) -> mlua::Result<mlua::Table> {
    let map_ctor: mlua::Function = lua.globals().get("map")?;
    map_ctor.call(init)
}

fn lua_table_looks_like_object(value: &mlua::Table) -> mlua::Result<bool> {
    for pair in value.pairs() {
        let (k, _v): (mlua::Value, mlua::Value) = pair?;
        if !k.is_string() {
            return Ok(false);
        }
    }

    return Ok(true);
}

fn lua_table_to_json_object(
    lua: &Lua,
    value: &mlua::Table,
) -> mlua::Result<serde_json::Map<String, serde_json::Value>> {
    let mut result: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for pair in value.pairs() {
        let (k, v): (String, mlua::Value) = pair?;
        result.insert(k, lua_to_json(lua, &v)?);
    }

    Ok(result)
}

pub fn lua_to_json(lua: &Lua, value: &mlua::Value) -> mlua::Result<serde_json::Value> {
    match value {
        mlua::Value::Nil => Ok(serde_json::Value::Null),
        mlua::Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        mlua::Value::Number(n) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*n).ok_or_else(|| {
                mlua::Error::runtime("Unable to convert lua number to json number")
            })?,
        )),
        mlua::Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_owned())),
        mlua::Value::Table(t) => {
            let metatable = match t.metatable() {
                Some(mt) => mt,
                None => lua.create_table()?,
            };

            if let Ok(mlua::Value::Boolean(true)) = metatable.get("is_array") {
                Ok(serde_json::Value::Array(lua_table_to_json_array(lua, t)?))
            } else if let Ok(mlua::Value::Boolean(true)) = metatable.get("is_map") {
                Ok(serde_json::Value::Object(lua_table_to_json_object(lua, t)?))
            } else if lua_table_looks_like_object(t)? {
                Ok(serde_json::Value::Object(lua_table_to_json_object(lua, t)?))
            } else if lua_table_looks_like_array(t)? {
                Ok(serde_json::Value::Array(lua_table_to_json_array(lua, t)?))
            } else {
                Err(mlua::Error::runtime(
                    "Lua table does not look like an array or object",
                ))
            }
        }
        _ => Err(mlua::Error::runtime(format!(
            "Type cannot be converted to json: {}",
            value.type_name()
        ))),
    }
}

fn render(
    generator_context: &GeneratorContext,
    lua: &Lua,
    template: &str,
    output: &str,
    params_opt: &Option<mlua::Table>,
) -> mlua::Result<()> {
    let params = match params_opt {
        Some(t) => lua_table_to_json_object(lua, t)?,
        None => serde_json::Map::new(),
    };

    let mut context_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    context_map.insert(
        String::from("schemas"),
        serde_json::to_value(&generator_context.schemas)
            .map_err(|e| mlua::Error::runtime(format!("{e}")))?,
    );
    context_map.insert(
        String::from("package"),
        serde_json::to_value(&generator_context.package)
            .map_err(|e| mlua::Error::runtime(format!("{e}")))?,
    );
    context_map.append(&mut generator_context.params.clone());
    context_map.insert(String::from("params"), serde_json::Value::Object(params));

    let tera = generator_context.generator.tera.lock().unwrap();
    let context = tera::Context::from_value(serde_json::Value::Object(context_map))
        .map_err(|e| mlua::Error::runtime(format!("{e}")))?;
    let rendered = (&*tera)
        .render(template, &context)
        .map_err(|e| mlua::Error::runtime(format!("{e:?}")))?;

    let output_file = generator_context.output_dir.join(output);
    if let Some(d) = output_file.parent() {
        create_dir_all(d).map_err(|e| mlua::Error::runtime(format!("{e}")))?;
    }

    std::fs::write(output_file, rendered).map_err(|e| mlua::Error::runtime(format!("{e}")))?;

    Ok(())
}

pub fn create_basic_lua_env() -> mlua::Result<Lua> {
    let lua = mlua::Lua::new_with(
        StdLib::STRING | StdLib::MATH | StdLib::TABLE | StdLib::UTF8,
        LuaOptions::new(),
    )?;

    let collections_bytes = include_bytes!("lua/collections.lua");
    let collections: mlua::Table = lua
        .load(&collections_bytes[..])
        .set_name("=collections")
        .call(())?;

    lua.globals()
        .set("map", collections.get::<mlua::Function>("map")?)?;
    lua.globals()
        .set("array", collections.get::<mlua::Function>("array")?)?;

    let strings_bytes = include_bytes!("lua/strings.lua");
    let strings: mlua::Table = lua.load(&strings_bytes[..]).set_name("=strings").call(())?;

    let string_table: mlua::Table = lua.globals().get("string")?;
    for pair in strings.pairs() {
        let (k, v): (String, mlua::Value) = pair?;
        string_table.set(k, v)?;
    }

    Ok(lua)
}

pub fn create_lua_env(context: GeneratorContext) -> mlua::Result<Lua> {
    let lua = create_basic_lua_env()?;

    let render_func = lua.create_function(
        move |lua: &Lua, args: (String, String, Option<mlua::Table>)| {
            render(&context, lua, args.0.as_str(), args.1.as_str(), &args.2)
        },
    )?;

    lua.globals().set("render", render_func)?;

    Ok(lua)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_title_to_snake_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "ThisIsATitleCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_a_title_case_name", snake_case)
        "#).exec()
    }

    #[test]
    fn test_string_title_with_acronym_to_snake_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "ThisIsSCUBATitleCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_scuba_title_case_name", snake_case)
        "#).exec()
    }

    #[test]
    fn test_string_camel_to_snake_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "thisIsACamelCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_a_camel_case_name", snake_case)
        "#).exec()
    }

    #[test]
    fn test_string_camel_with_number_to_snake_case() -> mlua::Result<()> {
        create_basic_lua_env()?.load(r#"
            local title_case = "thisIsA10CamelCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_a_10_camel_case_name", snake_case)
        "#).exec()
    }

    #[test]
    fn test_string_const_to_snake_case_with_word_sep_arg() -> mlua::Result<()> {
        create_basic_lua_env()?.load(r#"
            local const_case = "THIS_IS_A_CONST300_CASE_NAME"
            local snake_case = const_case:to_snake_case("_")
            assert(snake_case == "this_is_a_const300_case_name", snake_case)
        "#).exec()
    }

    #[test]
    fn test_string_title_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "ThisIsATitleCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-a-title-case-name", kebab_case)
        "#).exec()
    }

    #[test]
    fn test_string_title_with_acronym_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "ThisIsSCUBATitleCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-scuba-title-case-name", kebab_case)
        "#).exec()
    }

    #[test]
    fn test_string_camel_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "thisIsACamelCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-a-camel-case-name", kebab_case)
        "#).exec()
    }

    #[test]
    fn test_string_camel_with_number_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "thisIsA10CamelCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-a-10-camel-case-name", kebab_case)
        "#).exec()
    }

    #[test]
    fn test_string_snake_to_title_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local snake_case = "this_is_a_snake_case_name"
            local title_case = snake_case:to_title_case()
            assert(title_case == "ThisIsASnakeCaseName", title_case)
        "#).exec()
    }

    #[test]
    fn test_string_snake_with_number_to_title_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local snake_case = "this_is_a_100_snake_case_name"
            local title_case = snake_case:to_title_case()
            assert(title_case == "ThisIsA100SnakeCaseName", title_case)
        "#).exec()
    }

    #[test]
    fn test_string_title_with_number_to_const_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local title_case = "ThisIsATitleCaseName5000"
            local const_case = title_case:to_const_case()
            assert(const_case == "THIS_IS_A_TITLE_CASE_NAME_5000", const_case)
        "#).exec()
    }

}
