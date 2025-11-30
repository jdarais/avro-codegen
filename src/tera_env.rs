use std::borrow::Cow;
use std::collections::HashMap;

use serde_json::Value;
use tera::Tera;

pub fn head(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(arr[0].clone())
        },
        _ => Err(tera::Error::msg("value is not an array"))
    }
}

pub fn rhead(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(arr[arr.len()-1].clone())
        },
        _ => Err(tera::Error::msg("value is not an array"))
    }
}

pub fn tail(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(Value::Array(arr.iter().skip(1).map(|v| v.clone()).collect()))
        },
        _ => Err(tera::Error::msg("value is not an array"))
    }
}

pub fn rtail(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Array(arr) => match arr.len() {
            0 => Err(tera::Error::msg("array is empty")),
            _ => Ok(Value::Array(arr.iter().take(arr.len()-1).map(|v| v.clone()).collect()))
        },
        _ => Err(tera::Error::msg("value is not an array"))
    }
}

pub fn create_obj(args: &HashMap<String, Value>) -> tera::Result<Value> {
    Ok(Value::Object(args.iter().map(|(k,v)| (k.clone(), v.clone())).collect()))
}

pub fn set_attrs(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Object(obj) => {
            let mut result = obj.clone();
            for (k, v) in args {
                result.insert(k.clone(), v.clone());
            }
            Ok(Value::Object(result))
        },
        _ => Err(tera::Error::msg("value is not an object"))
    }
}

pub fn set_attr(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    match val {
        Value::Object(obj) => {
            let mut result = obj.clone();

            let k: Cow<str> = match args.get("key") {
                Some(Value::String(s)) => Cow::Borrowed(s.as_str()),
                Some(Value::Bool(b)) => if *b { Cow::Owned(String::from("true")) } else { Cow::Owned(String::from("false")) },
                Some(Value::Number(n)) => Cow::Owned(format!("{}", n)),
                _ => { return Err(tera::Error::msg("key must be able to be converted to a string (string, boolean, or number)")); }
            };

            let v = args.get("val")
                .map(|v| v.clone())
                .ok_or_else(|| tera::Error::msg("'val' argument is required"))?;

            result.insert(k.into_owned(), v);
            Ok(Value::Object(result))
        },
        _ => Err(tera::Error::msg("value is not an object"))
    }
}

pub fn schema_type_of_value(schema: &Value) -> tera::Result<Value> {
    match schema {
        Value::String(s) => Ok(Value::String(s.clone())),
        Value::Object(o) => match o.get("type") {
            Some(t) => schema_type_of_value(t),
            None => Err(tera::Error::msg("Schema object is missing a 'type' property"))
        },
        _ => Err(tera::Error::msg("Schema must be a string or object"))
    }
}

pub fn schema_type(args: &HashMap<String, Value>) -> tera::Result<Value> {
    match args.get("schema") {
        None => Err(tera::Error::msg("Missing argument: schema")),
        Some(schema) => schema_type_of_value(schema)

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

    tera.register_function("schema_type", schema_type);

    tera
}
