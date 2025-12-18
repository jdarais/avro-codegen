use std::borrow::Cow;
use std::collections::HashMap;

use serde_json::Value;
use tera::Tera;

use crate::lua_env::create_basic_lua_env;

pub fn head(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(arr[0].clone()),
        },
        _ => Err(tera::Error::msg("value is not an array")),
    }
}

pub fn rhead(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(arr[arr.len() - 1].clone()),
        },
        _ => Err(tera::Error::msg("value is not an array")),
    }
}

pub fn tail(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(Value::Array(
                arr.iter().skip(1).map(|v| v.clone()).collect(),
            )),
        },
        _ => Err(tera::Error::msg("value is not an array")),
    }
}

pub fn rtail(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(Value::Array(
                arr.iter().take(arr.len() - 1).map(|v| v.clone()).collect(),
            )),
        },
        _ => Err(tera::Error::msg("value is not an array")),
    }
}

pub fn create_obj(args: &HashMap<String, Value>) -> tera::Result<Value> {
    Ok(Value::Object(
        args.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
    ))
}

pub fn set_attrs(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Object(obj) => {
            let mut result = obj.clone();
            for (k, v) in args {
                result.insert(k.clone(), v.clone());
            }
            Ok(Value::Object(result))
        }
        _ => Err(tera::Error::msg("value is not an object")),
    }
}

pub fn set_attr(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Object(obj) => {
            let mut result = obj.clone();

            let k: Cow<str> = match args.get("key") {
                Some(Value::String(s)) => Cow::Borrowed(s.as_str()),
                Some(Value::Bool(b)) => {
                    if *b {
                        Cow::Owned(String::from("true"))
                    } else {
                        Cow::Owned(String::from("false"))
                    }
                }
                Some(Value::Number(n)) => Cow::Owned(format!("{}", n)),
                _ => {
                    return Err(tera::Error::msg(
                        "key must be able to be converted to a string (string, boolean, or number)",
                    ));
                }
            };

            let v = args
                .get("val")
                .map(|v| v.clone())
                .ok_or_else(|| tera::Error::msg("'val' argument is required"))?;

            result.insert(k.into_owned(), v);
            Ok(Value::Object(result))
        }
        _ => Err(tera::Error::msg("value is not an object")),
    }
}

fn to_snake_case(lua: &mlua::Lua, val: &Value) -> mlua::Result<Value> {
    match val {
        Value::String(s) => {
            let lua_val = lua.create_string(s)?;
            lua.load(r#"return string.to_snake_case(...)"#)
                .call((lua_val,))
                .map(|s: String| Value::String(s))
        }
        _ => Err(mlua::Error::runtime(
            "Argument must be a string",
        )),
    }
}

fn to_kebab_case(lua: &mlua::Lua, val: &Value) -> mlua::Result<Value> {
    match val {
        Value::String(s) => {
            let lua_val = lua.create_string(s)?;
            lua.load(r#"return string.to_kebab_case(...)"#)
                .call((lua_val,))
                .map(|s: String| Value::String(s))
        }
        _ => Err(mlua::Error::runtime(
            "Argument must be a string",
        )),
    }
}

fn to_camel_case(lua: &mlua::Lua, val: &Value) -> mlua::Result<Value> {
    match val {
        Value::String(s) => {
            let lua_val = lua.create_string(s)?;
            lua.load(r#"return string.to_camel_case(...)"#)
                .call((lua_val,))
                .map(|s: String| Value::String(s))
        }
        _ => Err(mlua::Error::runtime(
            "Argument must be a string",
        )),
    }
}

fn to_title_case(lua: &mlua::Lua, val: &Value) -> mlua::Result<Value> {
    match val {
        Value::String(s) => {
            let lua_val = lua.create_string(s)?;
            lua.load(r#"return string.to_title_case(...)"#)
                .call((lua_val,))
                .map(|s: String| Value::String(s))
        }
        _ => Err(mlua::Error::runtime(
            "Argument must be a string",
        )),
    }
}

fn to_const_case(lua: &mlua::Lua, val: &Value) -> mlua::Result<Value> {
    match val {
        Value::String(s) => {
            let lua_val = lua.create_string(s)?;
            lua.load(r#"return string.to_const_case(...)"#)
                .call((lua_val,))
                .map(|s: String| Value::String(s))
        }
        _ => Err(mlua::Error::runtime(
            "Argument must be a string",
        )),
    }
}

pub fn create_tera() -> Tera {
    let mut tera = Tera::default();

    tera.register_filter("head", head);
    tera.register_filter("rhead", rhead);

    tera.register_filter("tail", tail);
    tera.register_filter("rtail", rtail);

    tera.register_function("create_obj", create_obj);
    tera.register_filter("set_attrs", set_attrs);
    tera.register_filter("set_attr", set_attr);

    // Make some functions and filters available that are implemented in Lua
    if let Ok(lua) = create_basic_lua_env() {
        let lua_clone = lua.clone();
        tera.register_filter(
            "snake_case",
            move |val: &Value, _: &HashMap<String, Value>| {
                to_snake_case(&lua_clone, val).map_err(|e| tera::Error::msg(e.to_string()))
            },
        );

        let lua_clone = lua.clone();
        tera.register_filter(
            "kebab_case",
            move |val: &Value, _: &HashMap<String, Value>| {
                to_kebab_case(&lua_clone, val).map_err(|e| tera::Error::msg(e.to_string()))
            }
        );

        let lua_clone = lua.clone();
        tera.register_filter(
            "camel_case",
            move |val: &Value, _: &HashMap<String, Value>| {
                to_camel_case(&lua_clone, val).map_err(|e| tera::Error::msg(e.to_string()))
            },
        );

        let lua_clone = lua.clone();
        tera.register_filter(
            "title_case",
            move |val: &Value, _: &HashMap<String, Value>| {
                to_title_case(&lua_clone, val).map_err(|e| tera::Error::msg(e.to_string()))
            },
        );

        let lua_clone = lua.clone();
        tera.register_filter(
            "const_case",
            move |val: &Value, _: &HashMap<String, Value>| {
                to_const_case(&lua_clone, val).map_err(|e| tera::Error::msg(e.to_string()))
            },
        );
    }

    tera
}
