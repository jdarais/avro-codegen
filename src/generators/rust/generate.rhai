
fn join(arr, sep) {
    arr.reduce(|accum, s, i| if i == 0 { s } else { accum + sep + s }, "")
}

let modules = #{ "": #{ schemas: [], submodules: #{}}};

for schema in schemas.values() {
    let module_path = [];
    for mod in schema.namespace.split(".") {
        let parent_module_path_str = join(module_path, ".");
        module_path.push(mod);
        let module_path_str = join(module_path, ".");
        if !modules.contains(module_path_str) {
            modules[module_path_str] = #{ schemas: [], submodules: #{} };
        }
        if mod.len() > 0 {
            modules[parent_module_path_str].submodules[mod] = true;
        }
    }
 
    modules[schema.namespace].schemas.push(schema);
}

let lib_mod = modules.remove("");
let lib_submodules = lib_mod.submodules.keys();
lib_submodules.sort();
render("mod.tera", "src/lib.rs", #{ submodules: lib_submodules, schemas: lib_mod.schemas });

for mod in modules.keys() {
    let submodules = modules[mod].submodules.keys();
    submodules.sort();
    let mod_path = mod; mod_path.replace(".", "/");
    render("mod.tera", "src/" + mod_path + ".rs", #{ submodules: submodules, schemas: modules[mod].schemas });
}
