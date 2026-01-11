

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

render("package.json.tera", "package.json")
render("tsconfig.json.tera", "tsconfig.json")

for i, schema in ipairs(schemas) do
    local refs = map()
    find_refs(schema, refs)
    local refs_keys = refs:keys()
    table.sort(refs_keys)
    local refs_list = array()
    for i, key in ipairs(refs_keys) do
        refs_list:push(refs[key])
    end

    local file_path = "src/" .. schema.fullname:gsub("[.]", "/") .. ".mts"
    render("schema.tera", file_path, map {schema=schema, refs=refs_list})
end
