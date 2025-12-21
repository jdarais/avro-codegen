use apache_avro::schema::Schema;
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


pub fn schema_to_json(schema: &Schema, schema_info: &SchemaInfo) -> serde_json::Value {
    match schema {
        Schema::Null => json!({
            "type": "null",
            "json": schema.canonical_form(),
        }),
        Schema::Boolean => json!({
            "type": "boolean",
            "json": schema.canonical_form(),
        }),
        Schema::Int => json!({
            "type": "int",
            "json": schema.canonical_form(),
        }),
        Schema::Long => json!({
            "type": "long",
            "json": schema.canonical_form(),
        }),
        Schema::Float => json!({
            "type": "float",
            "json": schema.canonical_form(),
        }),
        Schema::Double => json!({
            "type": "double",
            "json": schema.canonical_form(),
        }),
        Schema::Bytes => json!({
            "type": "bytes",
            "json": schema.canonical_form(),
        }),
        Schema::String => json!({
            "type": "string",
            "json": schema.canonical_form(),
        }),
        Schema::Array(array_schema) => json!({
            "type": "array",
            "items": schema_to_json(&array_schema.items, schema_info),
            "json": schema.canonical_form(),
        }),
        Schema::Map(map_schema) => json!({
            "type": "map",
            "values": schema_to_json(&map_schema.types, schema_info),
            "json": schema.canonical_form(),
        }),
        Schema::Union(union_schema) => json!({
            "type": "union",
            "variants": serde_json::Value::Array(union_schema.variants()
                .iter()
                .map(|s| schema_to_json(s, schema_info))
                .collect()),
            "json": schema.canonical_form()
        }),
        Schema::Record(record_schema) => {
            if record_schema.name.fullname(None) != schema_info.full_name {
                json!({
                    "type": "ref",
                    "name": record_schema.name.name,
                    "namespace": record_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": record_schema.name.fullname(None)
                })
            } else {
                let mut fields: Vec<serde_json::Value> = Vec::with_capacity(record_schema.fields.len());
                for field in record_schema.fields.iter() {
                    fields.push(json!({
                        "name": field.name,
                        "type": schema_to_json(&field.schema, schema_info),
                    }));
                }
    
                json!({
                    "type": "record",
                    "name": record_schema.name.name,
                    "aliases": record_schema.aliases,
                    "namespace": record_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "full_name": schema_info.full_name,
                    "doc": record_schema.doc,
                    "file_path": schema_info.file_path,
                    "fields": fields,
                    "json": schema.canonical_form(),
                })
            }
        }
        Schema::Enum(enum_schema) => {
            if enum_schema.name.fullname(None) != schema_info.full_name {
                json!({
                    "type": "ref",
                    "name": enum_schema.name.name,
                    "namespace": enum_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": enum_schema.name.fullname(None),
                })
            } else {
                json!({
                    "type": "enum",
                    "name": enum_schema.name.name,
                    "aliases": enum_schema.aliases,
                    "namespace": enum_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": enum_schema.name.fullname(None),
                    "doc": enum_schema.doc,
                    "symbols": enum_schema.symbols,
                    "default": enum_schema.default,
                    "json": schema.canonical_form(),
                })
            }
        }
        Schema::Fixed(fixed_schema) => {
            if fixed_schema.name.fullname(None) != schema_info.full_name {
                json!({
                    "type": "ref",
                    "name": fixed_schema.name.name,
                    "namespace": fixed_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": fixed_schema.name.fullname(None),
                })
            } else {
                json!({
                    "type": "fixed",
                    "name": fixed_schema.name.name,
                    "aliases": fixed_schema.aliases,
                    "namespace": fixed_schema.name.namespace.as_ref().cloned().unwrap_or_else(String::new),
                    "fullname": fixed_schema.name.fullname(None),
                    "size": fixed_schema.size,
                    "json": schema.canonical_form()
                })
            }
        }
        Schema::Decimal(decimal_schema) => {
            let inner = schema_to_json(&decimal_schema.inner.as_ref(), schema_info);

            match inner {
                serde_json::Value::Object(mut obj) => {
                    obj.insert(String::from("logical_type"), serde_json::Value::String("decimal".into()));
                    obj.insert(String::from("precision"), serde_json::Value::Number(decimal_schema.precision.into()));
                    obj.insert(String::from("scale"), serde_json::Value::Number(decimal_schema.scale.into()));
                    obj.insert(String::from("json"), serde_json::Value::String(schema.canonical_form().into()));
                    serde_json::Value::Object(obj)
                }
                _ => { panic!("Expected schema_to_json to return a json object"); }
            }
        }
        Schema::BigDecimal => json!({
            "type": "bytes",
            "logical_type": "big-decimal",
            "json": schema.canonical_form(),
        }),
        Schema::Uuid => json!({
            "type": "string",
            "logical_type": "uuid",
            "json": schema.canonical_form(),
        }),
        Schema::Date => json!({
            "type": "int",
            "logical_type": "date",
            "json": schema.canonical_form(),
        }),
        Schema::TimeMillis => json!({
            "type": "int",
            "logical_type": "time-millis",
            "json": schema.canonical_form(),
        }),
        Schema::TimeMicros => json!({
            "type": "long",
            "logical_type": "time-micros",
            "json": schema.canonical_form(),
        }),
        Schema::TimestampMillis => json!({
            "type": "long",
            "logical_type": "timestamp-millis",
            "json": schema.canonical_form(),
        }),
        Schema::TimestampMicros => json!({
            "type": "long",
            "logical_type": "timestamp-micros",
            "json": schema.canonical_form(),
        }),
        Schema::TimestampNanos => json!({
            "type": "long",
            "logical_type": "timestamp-nanos",
            "json": schema.canonical_form(),
        }),
        Schema::LocalTimestampMillis => json!({
            "type": "long",
            "logical_type": "local-timestamp-millis",
            "json": schema.canonical_form(),
        }),
        Schema::LocalTimestampMicros => json!({
            "type": "long",
            "logical_type": "local-timestamp-micros",
            "json": schema.canonical_form(),
        }),
        Schema::LocalTimestampNanos => json!({
            "type": "long",
            "logical_type": "local-timestamp-nanos",
            "json": schema.canonical_form(),
        }),
        Schema::Duration => json!({
            "type": "fixed",
            "logical_type": "duration",
            "name": "DurationLogicalTYpe",
            "size": 12,
            "json": schema.canonical_form(),
        }),
        Schema::Ref{ name} => json!({
            "type": "ref",
            "name": name.name,
            "namespace": name.namespace.as_ref().cloned().unwrap_or_else(String::new),
            "fullname": name.fullname(None)
        })
    }
}
