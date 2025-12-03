
local function to_snake_case(name)
    local words = array {}
    local name_deconst_cased = name
        :gsub("_([A-Z0-9]*)%f[_]", function(s) return "_"..s:lower() end)
        :gsub("^([A-Z0-9]*)_", function(s) return s:lower().."_" end)
        :gsub("_([A-Z0-9]*)$", function(s) return "_"..s:lower() end)

    for word in name_deconst_cased:reverse():gmatch("[a-z0-9]*[A-Z]?") do
        words:push(word)
    end
    return table.concat(words, "_"):reverse():lower()
end

local function to_kebab_case(name)
    local words = array {}
    local name_deconst_cased = name
        :gsub("_([A-Z0-9]*)%f[_]", function(s) return "_"..s:lower() end)
        :gsub("^([A-Z0-9]*)_", function(s) return s:lower().."_" end)
        :gsub("_([A-Z0-9]*)$", function(s) return "_"..s:lower() end)

    for word in name_deconst_cased:reverse():gmatch("[a-z0-9]*[A-Z]?") do
        words:push(word)
    end
    return table.concat(words, "-"):reverse():lower()
end

local function to_title_case(name)
    local words = array {}
    local name_deconst_cased = name
        :gsub("_([A-Z0-9]*)%f[_]", function(s) return "_"..s:lower() end)
        :gsub("^([A-Z0-9]*)_", function(s) return s:lower().."_" end)
        :gsub("_([A-Z0-9]*)$", function(s) return "_"..s:lower() end)

    for word in name_deconst_cased:reverse():gmatch("[a-z0-9]*[A-Z]?") do
        words:push(word:sub(1,-2):lower()..word:sub(-1):upper())
    end
    return table.concat(words, ""):reverse()
end

local function to_camel_case(name)
    local title_case = to_title_case(name)
    return title_case:sub(1,1):lower()..title_case:sub(2)
end

local function to_const_case(name)
    local words = array {}
    local name_deconst_cased = name
        :gsub("_([A-Z0-9]*)%f[_]", function(s) return "_"..s:lower() end)
        :gsub("^([A-Z0-9]*)_", function(s) return s:lower().."_" end)
        :gsub("_([A-Z0-9]*)$", function(s) return "_"..s:lower() end)

    for word in name_deconst_cased:reverse():gmatch("[a-z0-9]*[A-Z]?") do
        words:push(word)
    end
    return table.concat(words, "_"):reverse():upper()
end

return {
    to_snake_case = to_snake_case,
    to_kebab_case = to_kebab_case,
    to_title_case = to_title_case,
    to_camel_case = to_camel_case,
    to_const_case = to_const_case
}
