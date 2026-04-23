use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;

use mlua::{Lua, LuaOptions, LuaSerdeExt, StdLib};

use crate::datamodel::PackageInfo;
use crate::generator::Generator;

pub struct GeneratorContext {
    output_dir: PathBuf,
    generator: Arc<Generator>,
    schemas: Vec<serde_json::Value>,
    package: PackageInfo,
    params: serde_json::Map<String, serde_json::Value>,
}

impl GeneratorContext {
    pub fn new(
        output_dir: PathBuf,
        generator: Arc<Generator>,
        schemas: Vec<serde_json::Value>,
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

fn render(
    generator_context: &GeneratorContext,
    lua: &Lua,
    template: &str,
    output: &str,
    params_opt: &Option<mlua::Table>,
) -> mlua::Result<()> {
    let mut combined_params = generator_context.params.clone();

    let mut params: serde_json::Map<String, serde_json::Value> = match params_opt {
        Some(t) => lua.from_value(mlua::Value::Table(t.clone()))?,
        None => serde_json::Map::new(),
    };
    combined_params.append(&mut params);

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
    context_map.insert(
        String::from("params"),
        serde_json::Value::Object(combined_params),
    );

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
        .call((lua.array_metatable(),))?;

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

    let params: mlua::Value = lua.to_value(&serde_json::Value::Object(context.params.clone()))?;
    lua.globals().set("params", params)?;

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
    fn test_nested_map() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(r#"
            local name_groups = array {
                [1] = array { [1] = "Jerry", [2] = "George", [3] = "Elaine", [4] = "Kramer" },
                [2] = array { [1] = "Homer", [2] = "Marge", [3] = "Bart", [4] = "Lisa", [5] = "Maggie" }
            }
            local greetings = name_groups:map(function (names)
                return "I would like to greet you all: "..table.concat(names:map(function(name) return "Hi "..name.."! " end))
            end)

            assert(greetings[1] == "I would like to greet you all: Hi Jerry! Hi George! Hi Elaine! Hi Kramer! ", greetings[1])
            assert(greetings[2] == "I would like to greet you all: Hi Homer! Hi Marge! Hi Bart! Hi Lisa! Hi Maggie! ", greetings[2])
        
        "#).exec()
    }

    #[test]
    fn test_string_title_to_snake_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "ThisIsATitleCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_a_title_case_name", snake_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_title_with_acronym_to_snake_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "ThisIsSCUBATitleCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_scuba_title_case_name", snake_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_camel_to_snake_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "thisIsACamelCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_a_camel_case_name", snake_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_camel_with_number_to_snake_case() -> mlua::Result<()> {
        create_basic_lua_env()?
            .load(
                r#"
            local title_case = "thisIsA10CamelCaseName"
            local snake_case = title_case:to_snake_case()
            assert(snake_case == "this_is_a_10_camel_case_name", snake_case)
        "#,
            )
            .exec()
    }

    #[test]
    fn test_string_const_to_snake_case_with_word_sep_arg() -> mlua::Result<()> {
        create_basic_lua_env()?
            .load(
                r#"
            local const_case = "THIS_IS_A_CONST300_CASE_NAME"
            local snake_case = const_case:to_snake_case("_")
            assert(snake_case == "this_is_a_const300_case_name", snake_case)
        "#,
            )
            .exec()
    }

    #[test]
    fn test_string_title_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "ThisIsATitleCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-a-title-case-name", kebab_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_title_with_acronym_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "ThisIsSCUBATitleCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-scuba-title-case-name", kebab_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_camel_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "thisIsACamelCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-a-camel-case-name", kebab_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_camel_with_number_to_kebab_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "thisIsA10CamelCaseName"
            local kebab_case = title_case:to_kebab_case()
            assert(kebab_case == "this-is-a-10-camel-case-name", kebab_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_snake_to_title_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local snake_case = "this_is_a_snake_case_name"
            local title_case = snake_case:to_title_case()
            assert(title_case == "ThisIsASnakeCaseName", title_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_snake_with_number_to_title_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local snake_case = "this_is_a_100_snake_case_name"
            local title_case = snake_case:to_title_case()
            assert(title_case == "ThisIsA100SnakeCaseName", title_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_with_spaces_to_title_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local space_case = "this is a string with spaces"
            local title_case = space_case:to_title_case()
            assert(title_case == "ThisIsAStringWithSpaces", title_case)
        "#,
        )
        .exec()
    }

    #[test]
    fn test_string_title_with_number_to_const_case() -> mlua::Result<()> {
        let lua = create_basic_lua_env()?;
        lua.load(
            r#"
            local title_case = "ThisIsATitleCaseName5000"
            local const_case = title_case:to_const_case()
            assert(const_case == "THIS_IS_A_TITLE_CASE_NAME_5000", const_case)
        "#,
        )
        .exec()
    }
}
