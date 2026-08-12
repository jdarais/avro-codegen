// Avro-Codegen
// Copyright (C) 2026 Jeremiah Darais
//
// This program is licensed under the GPLv3.0 license (https://github.com/jdarais/cobble/blob/main/COPYING)

use minijinja::Environment;

use crate::lua_env::create_basic_lua_env;

pub fn head(arr: &Vec<minijinja::Value>) -> Result<minijinja::Value, minijinja::Error> {
    match arr.len() {
        0 => Err(minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "array is empty",
        )),
        _ => Ok(arr[0].clone()),
    }
}

pub fn rhead(arr: &Vec<minijinja::Value>) -> Result<minijinja::Value, minijinja::Error> {
    match arr.len() {
        0 => Err(minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "array is empty",
        )),
        _ => Ok(arr[arr.len() - 1].clone()),
    }
}

pub fn tail(arr: &Vec<minijinja::Value>) -> Result<Vec<minijinja::Value>, minijinja::Error> {
    match arr.len() {
        0 => Err(minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "array is empty",
        )),
        _ => Ok(arr
            .iter()
            .skip(1)
            .map(|v| v.clone())
            .collect::<Vec<minijinja::Value>>()),
    }
}

pub fn rtail(arr: &Vec<minijinja::Value>) -> Result<Vec<minijinja::Value>, minijinja::Error> {
    match arr.len() {
        0 => Err(minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "array is empty",
        )),
        _ => Ok(arr
            .iter()
            .take(arr.len() - 1)
            .map(|v| v.clone())
            .collect::<Vec<minijinja::Value>>()),
    }
}

fn to_snake_case(lua: &mlua::Lua, val: &str) -> mlua::Result<String> {
    let lua_val = lua.create_string(val)?;
    lua.load(r#"return string.to_snake_case(...)"#)
        .call((lua_val,))
}

fn to_kebab_case(lua: &mlua::Lua, val: &str) -> mlua::Result<String> {
    let lua_val = lua.create_string(val)?;
    lua.load(r#"return string.to_kebab_case(...)"#)
        .call((lua_val,))
}

fn to_camel_case(lua: &mlua::Lua, val: &str) -> mlua::Result<String> {
    let lua_val = lua.create_string(val)?;
    lua.load(r#"return string.to_camel_case(...)"#)
        .call((lua_val,))
}

fn to_title_case(lua: &mlua::Lua, val: &str) -> mlua::Result<String> {
    let lua_val = lua.create_string(val)?;
    lua.load(r#"return string.to_title_case(...)"#)
        .call((lua_val,))
}

fn to_const_case(lua: &mlua::Lua, val: &str) -> mlua::Result<String> {
    let lua_val = lua.create_string(val)?;
    lua.load(r#"return string.to_const_case(...)"#)
        .call((lua_val,))
}

pub fn create_env() -> Environment<'static> {
    let mut env: Environment<'static> = Environment::new();

    env.add_filter("head", head);
    env.add_filter("rhead", rhead);

    env.add_filter("tail", tail);
    env.add_filter("rtail", rtail);

    // Make some functions and filters available that are implemented in Lua
    if let Ok(lua) = create_basic_lua_env() {
        let lua_clone = lua.clone();
        env.add_filter("snake_case", move |val: &str| {
            to_snake_case(&lua_clone, val).map_err(|e| {
                minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, format!("{e:?}"))
            })
        });

        let lua_clone = lua.clone();
        env.add_filter("kebab_case", move |val: &str| {
            to_kebab_case(&lua_clone, val).map_err(|e| {
                minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, format!("{e:?}"))
            })
        });

        let lua_clone = lua.clone();
        env.add_filter("camel_case", move |val: &str| {
            to_camel_case(&lua_clone, val).map_err(|e| {
                minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, format!("{e:?}"))
            })
        });

        let lua_clone = lua.clone();
        env.add_filter("title_case", move |val: &str| {
            to_title_case(&lua_clone, val).map_err(|e| {
                minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, format!("{e:?}"))
            })
        });

        let lua_clone = lua.clone();
        env.add_filter("const_case", move |val: &str| {
            to_const_case(&lua_clone, val).map_err(|e| {
                minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, format!("{e:?}"))
            })
        });
    }

    env
}
