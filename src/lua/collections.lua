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
        result = setmetatable({}, getmetatable(self))
        for i, v in ipairs(self) do
            result:push(map_fn(v, i))
        end
        return result
    end
}

local array_metatable = {
    is_array = true,
    __index = array_methods,
    __newindex = function(self, key, value)
        if type(key) ~= "number" then
            error("Only number keys are allowed in arrays")
        end

        if key < 1 or key > #self+1 then
            error("Array index out of bounds: "..key)
        end

        rawset(self, key, value)
    end,
    __add = function(self, other)
        local result = setmetatable({}, getmetatable(self))
        result:append(self)
        result:append(other)
        return result
    end
}

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
    is_map = true,
    __index = map_methods,
    __newindex = function(self, key, value)
        if type(key) ~= "string" then
            error("Only string keys are allowed in maps")
        end

        rawset(self, key, value)
    end,
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
