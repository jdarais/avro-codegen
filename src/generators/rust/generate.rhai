
fn join(arr, sep) {
    arr.reduce(|accum, s, i| if i == 0 { s } else { accum + sep + s }, "")
}

// fn find_unions(schema) {
//     let result = [];

//     switch schema.type {
//         "record" => {
//             let field_unions = schema.fields.map(|f| find_unions(f.type));
//             result.append(field_unions);
//         }
//         "array" => {
//             result.append(find_unions(schema.items));
//         }
//         "map" => {
//             result.append(find_unions(schema.values));
//         }
//         "union" => {
//             result.push(schema);
//             for variant in schema.variants {
//                 result.append(find_unions(variant));
//             }
//         }
//         _ => { /* no unions here */}
//     }

//     result
// }

fn get_union_variant_name(schema) {
    switch schema.type {
        "null" => "Null",
        "boolean" => "Bool",
        "int" => "Int",
        "long" => "Long",
        "float" => "Float",
        "double" => "Double",
        "bytes" => "Bytes",
        "string" => "String",
        "array" => get_union_variant_name(schema.items) + "Array",
        "map" => get_union_variant_name(schema.values) + "Map",
        "union" => union_to_name(schema),
        "record" | "enum" | "fixed" | "ref" => schema.fullname.replace(".", "_"),
        _ => "Unknown"
    }
}

fn union_to_name(schema) {
    join(schema.variants.map(get_union_variant_name), "Or")
}

let modules = #{ "": #{ schemas: [], submodules: #{}, unions: #{} } };

for schema in schemas.values() {
    let module_path = [];
    for mod in schema.namespace.split(".") {
        let parent_module_path_str = join(module_path, ".");
        module_path.push(mod);
        let module_path_str = join(module_path, ".");
        if !modules.contains(module_path_str) {
            modules[module_path_str] = #{ schemas: [], submodules: #{}, unions: #{} };
        }
        if mod.len() > 0 {
            modules[parent_module_path_str].submodules[mod] = true;
        }
    }
 
    modules[schema.namespace].schemas.push(schema);
    // for union in find_unions(schema) {
    //     modules[schema.namespace].unions[union_to_name(union)] = union;
    // }
}

let lib_mod = modules.remove("");
let lib_submodules = lib_mod.submodules.keys();
lib_submodules.sort();
render("mod.tera", "src/lib.rs", #{ submodules: lib_submodules, schemas: lib_mod.schemas, unions: lib_mod.unions });

for mod in modules.keys() {
    let submodules = modules[mod].submodules.keys();
    submodules.sort();
    let mod_path = mod; mod_path.replace(".", "/");
    render("mod.tera", "src/" + mod_path + ".rs", #{ submodules: submodules, schemas: modules[mod].schemas, unions: modules[mod].unions });
}
