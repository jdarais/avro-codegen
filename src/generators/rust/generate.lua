

local union_variant_names = (function()
    local x = {}
    x["null"] =     function () return "Null" end
    x["boolean"] =  function () return "Boolean" end
    x["int"] =      function () return "Int" end
    x["long"] =     function () return "Long" end
    x["float"] =    function () return "Float" end
    x["double"] =   function () return "Double" end
    x["bytes"] =    function () return "Bytes" end
    x["string"] =   function () return "String" end
    x["array"] = function (schema) return x[schema.items].."Array" end
    x["map"] = function (schema) return x[schema.values].."Map" end
    x["union"] = function (schema) return union_to_name(schema) end
    x["record"] = function (schema) return schema.name end
    x["enum"] = x["record"]
    x["fixed"] = x["record"]
    x["ref"] = x["record"]
    return x
end)()

function union_to_name(schema)
    return table.concat(
        array(schema.variants):map(function (v) return union_variant_names[v.type](v) end),
        "Or"
    )
end

function find_unions(schema, path)
    local result = array{}

    if schema.type == "record" and #path == 1 then
        local field_unions = schema.fields:map(function (f)
            return find_unions(f.type, path + {f.name})
        end)

        for i, unions in ipairs(field_unions) do
            result:append(unions)
        end
    elseif schema.type == "array" then
        result:append(find_unions(schema.items, path + {"arritem"}))
    elseif schema.type == "map" then
        result:append(find_unions(schema.values, path + {"mapval"}))
    elseif schema.type == "union" then
        result:push(map{schema=schema, path=path})
        for i, variant in ipairs(schema.variants) do
            result:append(find_unions(variant, path + {"variant"..tostring(i)}))
        end
    end

    return result
end

local modules = map{ [""] = map{ schemas = array{}, submodules = map{}, unions = map{} } }

for name, schema in pairs(schemas) do
    local module_path = array{}
    for mod in schema.namespace:gmatch("[^.]+") do
        local parent_module_path_str = table.concat(module_path, ".")
        module_path:push(mod)
        local module_path_str = table.concat(module_path, ".")
        if modules[module_path_str] == nil then
            modules[module_path_str] = map{ schemas = array{}, submodules = map{}, unions = map{} }
        end
        if #mod > 0 then
            modules[parent_module_path_str].submodules[mod] = true
        end
    end

    local module = modules[schema.namespace]
    module.schemas:push(schema)

    for i, union in ipairs(find_unions(schema, array {schema.name})) do
        local union_name = union_to_name(union.schema)
        module.unions[union_name] = union
    end
end

render("Cargo.toml.tera", "Cargo.toml")

local lib_mod = modules:remove("")
local lib_submodules = lib_mod.submodules:keys()
table.sort(lib_submodules)
render("mod.tera", "src/lib.rs", map{ submodules = lib_submodules, schemas = lib_mod.schemas, unions = lib_mod.unions })

for name, module in pairs(modules) do
    local submodules = module.submodules:keys()
    table.sort(submodules)
    render("mod.tera", "src/"..name:gsub("[.]", "/")..".rs", map{ submodules = submodules, schemas = module.schemas, unions = module.unions })
end
