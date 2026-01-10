


render("package.json.tera", "package.json")

for i, schema in ipairs(schemas) do
    local file_path = "src/" .. schema.fullname:gsub("[.]", "/") .. ".ts"
    render("schema.tera", file_path, map {schema=schema})
end
