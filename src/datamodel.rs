use apache_avro::schema::Schema;
use apache_avro::AvroResult;
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


pub fn schema_to_json(schema: &Schema, schema_info: &SchemaInfo, schemata: &[Schema]) -> AvroResult<serde_json::Value> {
    let schema_json = schema.independent_canonical_form(schemata)?;
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
                    "name": record_schema.name.name,
                    "namespace": record_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": record_schema.name.fullname(None)
                }))
            } else {
                let mut fields: Vec<serde_json::Value> = Vec::with_capacity(record_schema.fields.len());
                for field in record_schema.fields.iter() {
                    fields.push(json!({
                        "name": field.name,
                        "type": schema_to_json(&field.schema, schema_info, schemata)?,
                    }));
                }
    
                Ok(json!({
                    "type": "record",
                    "name": record_schema.name.name,
                    "aliases": record_schema.aliases,
                    "namespace": record_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "full_name": schema_info.full_name,
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
                    "name": enum_schema.name.name,
                    "namespace": enum_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": enum_schema.name.fullname(None),
                }))
            } else {
                Ok(json!({
                    "type": "enum",
                    "name": enum_schema.name.name,
                    "aliases": enum_schema.aliases,
                    "namespace": enum_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": enum_schema.name.fullname(None),
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
                    "name": fixed_schema.name.name,
                    "namespace": fixed_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": fixed_schema.name.fullname(None),
                }))
            } else {
                Ok(json!({
                    "type": "fixed",
                    "name": fixed_schema.name.name,
                    "aliases": fixed_schema.aliases,
                    "namespace": fixed_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": fixed_schema.name.fullname(None),
                    "size": fixed_schema.size,
                    "json": schema_json
                }))
            }
        }
        Schema::Decimal(decimal_schema) => {
            let inner = schema_to_json(&decimal_schema.inner.as_ref(), schema_info, schemata)?;

            match inner {
                serde_json::Value::Object(mut obj) => {
                    obj.insert(String::from("logical_type"), serde_json::Value::String("decimal".into()));
                    obj.insert(String::from("precision"), serde_json::Value::Number(decimal_schema.precision.into()));
                    obj.insert(String::from("scale"), serde_json::Value::Number(decimal_schema.scale.into()));
                    obj.insert(String::from("json"), serde_json::Value::String(schema_json.into()));
                    Ok(serde_json::Value::Object(obj))
                }
                _ => { panic!("Expected schema_to_json to return a json object"); }
            }
        }
        Schema::BigDecimal => Ok(json!({
            "type": "bytes",
            "logical_type": "big-decimal",
            "json": schema_json,
        })),
        Schema::Uuid => Ok(json!({
            "type": "string",
            "logical_type": "uuid",
            "json": schema_json,
        })),
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
        Schema::Duration => Ok(json!({
            "type": "fixed",
            "logical_type": "duration",
            "name": "DurationLogicalTYpe",
            "size": 12,
            "json": schema_json,
        })),
        Schema::Ref{ name} => Ok(json!({
            "type": "ref",
            "name": name.name,
            "namespace": name.namespace.as_ref().cloned().unwrap_or_else(String::new),
            "fullname": name.fullname(None)
        }))
    }
}
