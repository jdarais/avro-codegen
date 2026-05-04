local function find_ref_namespaces(schema, refs, namespaces)
    if schema.type == "ref" then
        refs[schema.fullname] = schema
        namespaces[schema.namespace] = true
    elseif schema.type == "record" then
        for i, field in ipairs(schema.fields) do
            find_ref_namespaces(field.type, refs, namespaces)
        end
    elseif schema.type == "array" then
        find_ref_namespaces(schema.items, refs, namespaces)
    elseif schema.type == "map" then
        find_ref_namespaces(schema.values, refs, namespaces)
    elseif schema.type == "union" then
        for i, variant in ipairs(schema.variants) do
            find_ref_namespaces(variant, refs, namespaces)
        end
    end
end

local function header_name(ns)
    return table.concat(ns:split("[.]"):map(function (s) return s.."/" end)).."types.h"
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
    local refs = map{}
    local ref_namespaces = map{}
    for i, schema in ipairs(schemas) do
        find_ref_namespaces(schema, refs, ref_namespaces)
    end

    local incl = ref_namespaces:keys():map(header_name)
    table.sort(incl)

    local record_ref_names = refs:keys()
    table.sort(record_ref_names)
    local record_refs = record_ref_names:map(function (n) return refs[n] end)

    local cpp_namespace = ns:gsub("[.]", "::")

    render(
        "header.tera",
        "include/"..header_name(ns),
        {
            namespace=cpp_namespace,
            schemas=schemas,
            includes=incl,
            record_refs=record_refs
        }
    )
end
