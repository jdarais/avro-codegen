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

// fn schema_to_lua(lua: &mlua::Lua, schema: &Schema) -> mlua::Result<mlua::Value> {
//     let result = lua.create_table()?;
//     match schema {
//         Schema::Null => {
//             result.set("type", "null")?;
//         }
//         Schema::Boolean => {
//             result.set("type", "boolean")?;
//         }
//         Schema::Int => {
//             result.set("type", "int")?;
//         }
//         Schema::Long => {
//             result.set("type", "long")?;
//         }
//         Schema::Float => {
//             result.set("type", "float")?;
//         }
//         Schema::Double => {
//             result.set("type", "double")?;
//         }
//         Schema::Ref { name } => {
//             result.set("type", "ref")?;
//             result.set("name", name.fullname(None))?;
//         }
//         Schema::Record(record) => {
//             result.set("type", "record")?;
//             result.set("name", record.name.name.clone())?;

//             if let Some(ns) = record.name.namespace.as_ref() {
//                 result.set("namespace", ns.clone())?;
//             }

//             if let Some(d) = record.doc.as_ref() {
//                 result.set("doc", d.clone())?;
//             }

//             if let Some(aliases) = &record.aliases {
//                 let alias_names: Vec<String> = aliases.iter().map(|a| a.fullname(None)).collect();
//                 result.set("aliases", alias_names)?;
//             }

//             let fields_table = lua.create_table()?;
//             for field in &record.fields {
//                 let field_table = lua.create_table()?;
//                 field_table.set("name", field.name.clone())?;
                
//                 if let Some(d) = field.doc.as_ref() {
//                     field_table.set("doc", d.clone())?;
//                 }

//                 field_table.set("type", schema_to_lua(lua, &field.schema)?)?;

//                 if let Some(aliases) = field.aliases.as_ref() {
//                     field_table.set("aliases", aliases.clone())?;
//                 }

//                 // if let Some(default) = field.default.as_ref() {
//                 //     field_table.set("default", default)?;
//                 // }

//                 fields_table.push(field_table)?;
//             }

//             result.set("fields", fields_table)?;
//         }
//         _ => {
//             result.set("type", "unknown")?;
//         }
//     };

//     Ok(mlua::Value::Table(result))
// }

// impl mlua::IntoLua for SchemaInfo {
//     fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
//         let result = lua.create_table()?;
//         result.set("name", self.name.clone())?;
//         result.set("namespace", self.namespace.clone())?;
//         result.set("full_name", self.full_name.clone())?;
//         result.set("file_path", self.file_path.clone())?;
//         result.set("schema", schema_to_lua(lua, &self.schema)?)?;

//         Ok(mlua::Value::Table(result))
//     }
// }

#[derive(Serialize, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

// impl mlua::IntoLua for PackageInfo {
//     fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
//         let result = lua.create_table()?;
//         result.set("name", self.name.clone())?;
//         result.set("version", self.version.clone())?;
//         result.set("description", self.description.clone())?;

//         Ok(mlua::Value::Table(result))
//     }
// }
