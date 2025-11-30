use apache_avro::schema::Schema;
use serde::Serialize;

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

