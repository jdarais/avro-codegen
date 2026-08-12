
local union_cardinalities = {}
local modules = map{ [""] = map{ schemas = array{}, submodules = map{}, unions = map{} } }

function find_unions(schema, out_cardinalities)
    if schema.type == "record" then
        for i, field in ipairs(schema.fields) do
            find_unions(field.type, out_cardinalities)
        end
    elseif schema.type == "array" then
        find_unions(schema.items, out_cardinalities)
    elseif schema.type == "map" then
        find_unions(schema.values, out_cardinalities)
    elseif schema.type == "union" then
        out_cardinalities[#schema.variants] = true
        for i, variant in ipairs(schema.variants) do
            find_unions(variant, out_cardinalities)
        end
    end
end

for i, schema in ipairs(schemas) do
    local module_path = array{}
    for path_segment in schema.namespace:gmatch("[^.]+") do
        local parent_module_path_str = table.concat(module_path, ".")
        module_path:push(path_segment)
        local module_path_str = table.concat(module_path, ".")

        if modules[module_path_str] == nil then
            modules[module_path_str] = map{ schemas = array{}, submodules = map{}, unions = map{} }
        end

        -- Avoid adding a mod statement for empty path segments
        if #path_segment > 0 then
            modules[parent_module_path_str].submodules[path_segment] = true
        end
    end

    local module = modules[schema.namespace]
    module.schemas:push(schema)

    find_unions(schema, union_cardinalities)
end


if params.cargo_toml then
    render("Cargo.toml.jinja", "Cargo.toml")
end

render("io.jinja", "src/_io.rs")

local union_cardinalities_list = array {}
for k, _ in pairs(union_cardinalities) do
    union_cardinalities_list:push(k)
end
render("unions.jinja", "src/_unions.rs", map { union_cardinalities = union_cardinalities_list })

local lib_mod = modules:remove("")
lib_mod.submodules["_io"] = true
lib_mod.submodules["_unions"] = true
local lib_submodules = lib_mod.submodules:keys()
table.sort(lib_submodules)
render(
    "mod.jinja",
    "src/lib.rs",
    map{
        submodules = lib_submodules,
        schemas = lib_mod.schemas,
        unions = lib_mod.unions
    }
)

for name, module in pairs(modules) do
    local submodules = module.submodules:keys()
    table.sort(submodules)
    render(
        "mod.jinja",
        "src/"..name:gsub("[.]", "/")..".rs",
        map{
            submodules = submodules,
            schemas = module.schemas,
            unions = module.unions
        }
    )
end
