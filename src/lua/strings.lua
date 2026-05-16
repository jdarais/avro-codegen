-- Avro-Codegen
-- Copyright (C) 2026 Jeremiah Darais
--
-- This program is licensed under the GPLv3.0 license (https://github.com/jdarais/cobble/blob/main/COPYING)

local function get_next_word_from_reversed(reversed, start_from)
    local next_word, after_index = reversed:match("^([a-z]+[A-Z]?)()", start_from)
    if next_word then
        return next_word, after_index
    end

    next_word, after_index = reversed:match("^([A-Z]+)()", start_from)
    if next_word then
        return next_word, after_index
    end

    next_word, after_index = reversed:match("^([0-9]+)()", start_from)
    if next_word then
        return next_word, after_index
    end

    return nil, start_from + 1
end

local function get_words(str)
    local str_len = #str
    local words = array {}
    local reversed = str:reverse()

    local idx = 1
    while idx <= str_len do
        local next_word, after_index = get_next_word_from_reversed(reversed, idx)
        if next_word then
            words:push(next_word:reverse())
        end
        idx = after_index
    end

    local words_len = #words
    for i = 1, math.floor(words_len/2) do
        local temp = words[i]
        words[i] = words[words_len+1-i]
        words[words_len+1-i] = temp
    end

    return words
end

local function split(str, sep)
    local words = array {}
    local len = #str
    local word_start = 1
    while word_start <= len do
        local s, e = str:find(sep, word_start)
        if not s then
            s = len + 1
            e = len + 1
        end

        words:push(str:sub(word_start,s-1))
        word_start = e + 1
    end

    return words
end

local function to_snake_case(name, word_sep)
    local words = word_sep and name:split(word_sep) or get_words(name)
    return table.concat(words, "_"):lower()
end

local function to_kebab_case(name, word_sep)
    local words = word_sep and name:split(word_sep) or get_words(name)
    return table.concat(words, "-"):lower()
end

local function to_title_case(name, word_sep)
    local words = word_sep and name:split(word_sep) or get_words(name)
    local words_title_case = words:map(function(w) return w:sub(1,1):upper()..w:sub(2):lower() end)
    return table.concat(words_title_case, "")
end

local function to_camel_case(name, word_sep)
    local title_case = to_title_case(name, word_sep)
    return title_case:sub(1,1):lower()..title_case:sub(2)
end

local function to_const_case(name, word_sep)
    local words = word_sep and name:split(word_sep) or get_words(name)
    return table.concat(words, "_"):upper()
end

return {
    to_snake_case = to_snake_case,
    to_kebab_case = to_kebab_case,
    to_title_case = to_title_case,
    to_camel_case = to_camel_case,
    to_const_case = to_const_case,
    split = split
}
