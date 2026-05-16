-- Avro-Codegen
-- Copyright (C) 2026 Jeremiah Darais
--
-- This program is licensed under the GPLv3.0 license (https://github.com/jdarais/cobble/blob/main/COPYING)

local array_metatable = ...

local array_methods = {
    push = function(self, value)
        table.insert(self, value)
    end,

    append = function(self, other)
        for i, v in ipairs(other) do
            table.insert(self, v)
        end
    end,

    map = function(self, map_fn)
        local result = setmetatable({}, array_metatable)
        for i, v in ipairs(self) do
            -- index and value going into map_fn are intentionally reversed.  This allows simple
            -- map functions that just operate on the value, but the index is available if needed
            result:push(map_fn(v, i))
        end
        return result
    end
}

array_metatable.__index = array_methods
function array_metatable.__newindex(self, key, value)
    if type(key) ~= "number" then
        error("Only number keys are allowed in arrays")
    end

    if key < 1 or key > #self+1 then
        error("Array index out of bounds: "..key)
    end

    rawset(self, key, value)
end

function array_metatable.__add(self, other)
    local result = setmetatable({}, array_metatable)
    result:append(self)
    result:append(other)
    return result
end

function array_metatable.__tostring(self, visited)
    if visited == nil then
        visited = {}
    end
    
    if visited[self] then
        return "..."
    end

    visited[self] = true

    local s = "{ "
    for i, v in ipairs(self) do
        s = s..tostring(i, visited).." = "..tostring(v, visited)..", "
    end
    s = s.."}"
    return s
end


local function array(init)
    local result = setmetatable({}, array_metatable)
    if init ~= nil then
        result:append(init)
    end
    return result
end

local map_methods = {
    update = function(self, other)
        for k, v in pairs(other) do
            self[k] = v
        end
    end,

    remove = function(self, key)
        local removed = self[key]
        self[key] = nil
        return removed
    end,

    keys = function(self)
        local result = array()
        for k, v in pairs(self) do
            result:push(k)
        end
        return result
    end,

    values = function(self)
        local result = array()
        for k, v in pairs(self) do
            result:push(v)
        end
        return result
    end
}

local map_metatable = {
    __index = map_methods,
    __newindex = function(self, key, value)
        if type(key) ~= "string" then
            error("Only string keys are allowed in maps")
        end

        rawset(self, key, value)
    end,
    __tostring = function(self, visited)
        if visited == nil then
            visited = {}
        end
        
        if visited[self] then
            return "..."
        end

        visited[self] = true
        
        local s = "{ "
        for k, v in pairs(self) do
            s = s..tostring(k, visited).." = "..tostring(v, visited)..", "
        end
        s = s.."}"
        return s
    end
}

local function map(init)
    local result = setmetatable({}, map_metatable)
    if init ~= nil then
        result:update(init)
    end
    return result
end

return {
    map = map,
    array = array
}
