local function find_ref_namespaces(schema, refs)
    if schema.type == "ref" then
        refs[schema.namespace] = true
    elseif schema.type == "record" then
        for i, field in ipairs(schema.fields) do
            find_ref_namespaces(field.type, refs)
        end
    elseif schema.type == "array" then
        find_ref_namespaces(schema.items, refs)
    elseif schema.type == "map" then
        find_ref_namespaces(schema.values, refs)
    elseif schema.type == "union" then
        for i, variant in ipairs(schema.variants) do
            find_ref_namespaces(variant, refs)
        end
    end
end

local function header_name(ns)
    return package.name.."/"..table.concat(ns:split("[.]"):map(function (s) return s.."/" end)).."types.h"
end

local schemas_by_namespace = map{}

for i, schema in ipairs(schemas) do
    local ns = schema.namespace
    if schemas_by_namespace[ns] == nil then
        schemas_by_namespace[ns] = array{}
    end

    schemas_by_namespace[ns]:push(schema)
end

for ns, schemas in pairs(schemas_by_namespace) do
    local ref_namespaces = map{}
    for i, schema in ipairs(schemas) do
        find_ref_namespaces(schema, ref_namespaces)
    end

    local incl = ref_namespaces:keys():map(header_name)
    table.sort(incl)

    local cpp_namespace = package.name
    if #ns > 0 then
        cpp_namespace = cpp_namespace.."::"..ns:gsub("[.]", "::")
    end

    render("header.tera", "include/"..header_name(ns), {namespace=cpp_namespace, schemas=schemas, includes=incl})
end
