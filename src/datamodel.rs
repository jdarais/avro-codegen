// Avro-Codegen
// Copyright (C) 2026 Jeremiah Darais
//
// This program is licensed under the GPLv3.0 license (https://github.com/jdarais/avro-codegen/blob/main/COPYING)]

use anyhow::anyhow;
use apache_avro::schema::{InnerDecimalSchema, RecordField, Schema, UnionSchema, UuidSchema};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize, Clone)]
pub struct SchemaInfo {
    pub name: String,
    pub namespace: String,
    pub full_name: String,
    pub file_path: String,
    pub schema: Schema,
}

#[derive(Serialize, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

pub fn denormalize_schema(schema: &Schema, schemata: &[Schema]) -> anyhow::Result<Schema> {
    match schema {
        Schema::Ref { name } => {
            let replacement_schema = schemata
                .iter()
                .find(|s| s.name().map(|n| *n == *name).unwrap_or(false));
            if let Some(schema) = replacement_schema {
                let denorm = denormalize_schema(schema, schemata)?;
                Ok(denorm)
            } else {
                Err(anyhow!("Error resolving schema with name {}", name.clone()))
            }
        }
        Schema::Record(record_schema) => {
            let mut denorm_schema = record_schema.clone();
            let mut fields: Vec<RecordField> = Vec::with_capacity(record_schema.fields.len());

            for mut field in denorm_schema.fields.drain(..) {
                field.schema = denormalize_schema(&field.schema, schemata)?;
                fields.push(field);
            }

            denorm_schema.fields = fields;

            Ok(Schema::Record(denorm_schema))
        }
        Schema::Array(array_schema) => {
            let mut denorm_schema = array_schema.clone();
            denorm_schema.items = Box::new(denormalize_schema(&array_schema.items, schemata)?);
            Ok(Schema::Array(denorm_schema))
        }
        Schema::Map(map_schema) => {
            let mut denorm_schema = map_schema.clone();
            denorm_schema.types = Box::new(denormalize_schema(&map_schema.types, schemata)?);
            Ok(Schema::Map(denorm_schema))
        }
        Schema::Union(union_schema) => {
            let mut variant_schemas: Vec<Schema> =
                Vec::with_capacity(union_schema.variants().len());

            for variant_schema in union_schema.variants() {
                variant_schemas.push(denormalize_schema(&variant_schema, schemata)?);
            }

            Ok(Schema::Union(UnionSchema::new(variant_schemas)?))
        }
        _ => Ok(schema.clone()),
    }
}

fn get_ref_type(name: &apache_avro::schema::Name, schemata: &[Schema]) -> &'static str {
    let ref_schema = schemata
        .iter()
        .find(|&s| s.name().map(|n| n == name).unwrap_or(false));
    match ref_schema {
        Some(Schema::Record(_)) => "record",
        Some(Schema::Enum(_)) => "enum",
        Some(Schema::Fixed(_)) => "fixed",
        Some(Schema::Decimal(_)) => "fixed",
        Some(Schema::Ref { name }) => get_ref_type(name, schemata),
        Some(x) => panic!(
            "Tried to follow named reference, but got an unnamed type: {:?}",
            x
        ),
        None => panic!(
            "Tried to follow reference for {}, but schema was not found",
            name
        ),
    }
}

pub fn schema_to_json(
    schema: &Schema,
    schema_info: &SchemaInfo,
    schemata: &[Schema],
) -> anyhow::Result<serde_json::Value> {
    let schema_json = serde_json::to_string(&denormalize_schema(schema, schemata)?)?;
    match schema {
        Schema::Null => Ok(json!({
            "type": "null",
            "json": schema_json,
        })),
        Schema::Boolean => Ok(json!({
            "type": "boolean",
            "json": schema_json,
        })),
        Schema::Int => Ok(json!({
            "type": "int",
            "json": schema_json,
        })),
        Schema::Long => Ok(json!({
            "type": "long",
            "json": schema_json,
        })),
        Schema::Float => Ok(json!({
            "type": "float",
            "json": schema_json,
        })),
        Schema::Double => Ok(json!({
            "type": "double",
            "json": schema_json,
        })),
        Schema::Bytes => Ok(json!({
            "type": "bytes",
            "json": schema_json,
        })),
        Schema::String => Ok(json!({
            "type": "string",
            "json": schema_json,
        })),
        Schema::Array(array_schema) => Ok(json!({
            "type": "array",
            "items": schema_to_json(&array_schema.items, schema_info, schemata)?,
            "json": schema_json,
        })),
        Schema::Map(map_schema) => Ok(json!({
            "type": "map",
            "values": schema_to_json(&map_schema.types, schema_info, schemata)?,
            "json": schema_json,
        })),
        Schema::Union(union_schema) => {
            let mut variants: Vec<serde_json::Value> = Vec::new();
            for variant_schema in union_schema.variants() {
                variants.push(schema_to_json(variant_schema, schema_info, schemata)?);
            }
            Ok(json!({
                "type": "union",
                "variants": serde_json::Value::Array(variants),
                "json": schema_json
            }))
        }
        Schema::Record(record_schema) => {
            if record_schema.name.fullname(None) != schema_info.full_name {
                Ok(json!({
                    "type": "ref",
                    "name": record_schema.name.name(),
                    "namespace": record_schema.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": record_schema.name.fullname(None),
                    "ref_type": "record",
                }))
            } else {
                let mut fields: Vec<serde_json::Value> =
                    Vec::with_capacity(record_schema.fields.len());
                for field in record_schema.fields.iter() {
                    fields.push(json!({
                        "name": field.name,
                        "type": schema_to_json(&field.schema, schema_info, schemata)?,
                    }));
                }

                Ok(json!({
                    "type": "record",
                    "name": record_schema.name.name(),
                    "aliases": record_schema.aliases,
                    "namespace": record_schema.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": schema_info.full_name,
                    "doc": record_schema.doc,
                    "file_path": schema_info.file_path,
                    "fields": fields,
                    "json": schema_json,
                }))
            }
        }
        Schema::Enum(enum_schema) => {
            if enum_schema.name.fullname(None) != schema_info.full_name {
                Ok(json!({
                    "type": "ref",
                    "name": enum_schema.name.name(),
                    "namespace": enum_schema.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": enum_schema.name.fullname(None),
                    "ref_type": "enum",
                }))
            } else {
                Ok(json!({
                    "type": "enum",
                    "name": enum_schema.name.name(),
                    "aliases": enum_schema.aliases,
                    "namespace": enum_schema.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": enum_schema.name.fullname(None),
                    "file_path": schema_info.file_path,
                    "doc": enum_schema.doc,
                    "symbols": enum_schema.symbols,
                    "default": enum_schema.default,
                    "json": schema_json,
                }))
            }
        }
        Schema::Fixed(fixed_schema) => {
            if fixed_schema.name.fullname(None) != schema_info.full_name {
                Ok(json!({
                    "type": "ref",
                    "name": fixed_schema.name.name(),
                    "namespace": fixed_schema.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": fixed_schema.name.fullname(None),
                    "ref_type": "fixed",
                }))
            } else {
                Ok(json!({
                    "type": "fixed",
                    "name": fixed_schema.name.name(),
                    "aliases": fixed_schema.aliases,
                    "namespace": fixed_schema.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": fixed_schema.name.fullname(None),
                    "file_path": schema_info.file_path,
                    "size": fixed_schema.size,
                    "json": schema_json
                }))
            }
        }
        Schema::Decimal(decimal_schema) => {
            let inner: serde_json::Value = match &decimal_schema.inner {
                InnerDecimalSchema::Bytes => {
                    let mut bytes_schema: serde_json::Map<String, serde_json::Value> =
                        serde_json::Map::new();
                    bytes_schema.insert(
                        String::from("type"),
                        serde_json::Value::String("bytes".into()),
                    );
                    serde_json::Value::Object(bytes_schema)
                }
                InnerDecimalSchema::Fixed(fixed_schema) => {
                    schema_to_json(&Schema::Fixed(fixed_schema.clone()), schema_info, schemata)?
                }
            };

            match inner {
                serde_json::Value::Object(mut obj) => {
                    match obj.get("type").and_then(serde_json::Value::as_str) {
                        Some("ref") => Ok(serde_json::Value::Object(obj)),
                        _ => {
                            obj.insert(
                                String::from("logical_type"),
                                serde_json::Value::String("decimal".into()),
                            );
                            obj.insert(
                                String::from("precision"),
                                serde_json::Value::Number(decimal_schema.precision.into()),
                            );
                            obj.insert(
                                String::from("scale"),
                                serde_json::Value::Number(decimal_schema.scale.into()),
                            );
                            obj.insert(
                                String::from("json"),
                                serde_json::Value::String(schema_json.into()),
                            );
                            Ok(serde_json::Value::Object(obj))
                        }
                    }
                }
                _ => panic!("Expected schema_to_json to return a json object"),
            }
        }
        Schema::BigDecimal => Ok(json!({
            "type": "bytes",
            "logical_type": "big-decimal",
            "json": schema_json,
        })),
        Schema::Uuid(uuid_schema) => match uuid_schema {
            UuidSchema::Bytes => Ok(json!({
                "type": "bytes",
                "logical_type": "uuid",
                "json": schema_json,
            })),
            UuidSchema::String => Ok(json!({
                "type": "string",
                "logical_type": "uuid",
                "json": schema_json
            })),
            UuidSchema::Fixed(fixed_schema) => {
                let inner =
                    schema_to_json(&Schema::Fixed(fixed_schema.clone()), schema_info, schemata)?;

                match inner {
                    serde_json::Value::Object(mut obj) => {
                        match obj.get("type").and_then(serde_json::Value::as_str) {
                            Some("ref") => Ok(serde_json::Value::Object(obj)),
                            _ => {
                                obj.insert(
                                    String::from("logical_type"),
                                    serde_json::Value::String("uuid".into()),
                                );
                                Ok(serde_json::Value::Object(obj))
                            }
                        }
                    }
                    _ => panic!("Expected schema_to_json to return an object"),
                }
            }
        },
        Schema::Date => Ok(json!({
            "type": "int",
            "logical_type": "date",
            "json": schema_json,
        })),
        Schema::TimeMillis => Ok(json!({
            "type": "int",
            "logical_type": "time-millis",
            "json": schema_json,
        })),
        Schema::TimeMicros => Ok(json!({
            "type": "long",
            "logical_type": "time-micros",
            "json": schema_json,
        })),
        Schema::TimestampMillis => Ok(json!({
            "type": "long",
            "logical_type": "timestamp-millis",
            "json": schema_json,
        })),
        Schema::TimestampMicros => Ok(json!({
            "type": "long",
            "logical_type": "timestamp-micros",
            "json": schema_json,
        })),
        Schema::TimestampNanos => Ok(json!({
            "type": "long",
            "logical_type": "timestamp-nanos",
            "json": schema_json,
        })),
        Schema::LocalTimestampMillis => Ok(json!({
            "type": "long",
            "logical_type": "local-timestamp-millis",
            "json": schema_json,
        })),
        Schema::LocalTimestampMicros => Ok(json!({
            "type": "long",
            "logical_type": "local-timestamp-micros",
            "json": schema_json,
        })),
        Schema::LocalTimestampNanos => Ok(json!({
            "type": "long",
            "logical_type": "local-timestamp-nanos",
            "json": schema_json,
        })),
        Schema::Duration(fixed) => {
            if fixed.name.fullname(None) != schema_info.full_name {
                Ok(json!({
                    "type": "ref",
                    "name": fixed.name.name(),
                    "namespace": fixed.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": fixed.name.fullname(None),
                    "ref_type": "fixed",
                }))
            } else {
                Ok(json!({
                    "type": "fixed",
                    "logical_type": "duration",
                    "name": fixed.name.name(),
                    "namespace": fixed.name.namespace().map(str::to_string).unwrap_or_else(String::new),
                    "fullname": fixed.name.fullname(None),
                    "file_path": schema_info.file_path,
                    "size": 12,
                    "json": schema_json,
                }))
            }
        }
        Schema::Ref { name } => Ok(json!({
            "type": "ref",
            "name": name.name(),
            "namespace": name.namespace().map(str::to_string).unwrap_or_else(String::new),
            "fullname": name.fullname(None),
            "ref_type": get_ref_type(name, schemata),
        })),
    }
}
