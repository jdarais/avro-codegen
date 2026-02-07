

local function find_refs(schema, refs)
    if schema.type == "ref" then
        refs[schema.fullname] = schema
    elseif schema.type == "record" then
        for i, field in ipairs(schema.fields) do
            find_refs(field.type, refs)
        end
    elseif schema.type == "array" then
        find_refs(schema.items, refs)
    elseif schema.type == "map" then
        find_refs(schema.values, refs)
    elseif schema.type == "union" then
        for i, variant in ipairs(schema.variants) do
            find_refs(variant, refs)
        end
    end
end

schemas_by_namespace = map{}
for i, schema in ipairs(schemas) do
    if schemas_by_namespace[schema.namespace] == nil then
        schemas_by_namespace[schema.namespace] = map {}
    end
    schemas_by_namespace[schema.namespace][schema.name] = schema
end

local namespaces = schemas_by_namespace:keys()
table.sort(namespaces)

render("package.json.tera", "package.json", {namespaces=namespaces})

render("io.mjs.tera", "dist/node/_io.mjs")
render("io.mjs.tera", "dist/browser/_io.mjs")

render("io.d.mts.tera", "dist/node/_io.d.mts")
render("io.d.mts.tera", "dist/browser/_io.d.mts")

for namespace, schemas in pairs(schemas_by_namespace) do
    local refs = map()
    for name, schema in pairs(schemas) do
        find_refs(schema, refs)
    end
    local refs_keys = refs:keys()
    table.sort(refs_keys)
    local refs_list = array()
    for i, key in ipairs(refs_keys) do
        refs_list:push(refs[key])
    end

    local schema_names = schemas:keys()
    table.sort(schema_names)
    local schemas_list = array {}
    for i, name in ipairs(schema_names) do
        schemas_list:push(schemas[name])
    end

    local impl_file_path = namespace:gsub("[.]", "/"):gsub("(.+)$", "%1/") .. "types.mjs"
    local types_file_path = namespace:gsub("[.]", "/"):gsub("(.+)$", "%1/") .. "types.d.mts"

    render("schema.mjs.tera", "dist/node/" .. impl_file_path, map {namespace=namespace, schemas=schemas_list, refs=refs_list, is_node=true})
    render("schema.mjs.tera", "dist/browser/" .. impl_file_path, map {namespace=namespace, schemas=schemas_list, refs=refs_list, is_node=false})
    
    render("schema.d.mts.tera", "dist/node/" .. types_file_path, map {namespace=namespace, schemas=schemas_list, refs=refs_list, is_node=true})
    render("schema.d.mts.tera", "dist/browser/" .. types_file_path, map {namespace=namespace, schemas=schemas_list, refs=refs_list, is_node=false})
end
