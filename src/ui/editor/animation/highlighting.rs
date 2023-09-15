use super::Syntax;
use std::collections::HashSet;

#[must_use]
pub fn lua() -> Syntax {
    Syntax {
        language: "Lua",
        case_sensitive: true,
        comment: "--",
        comment_multiline: ["--[[", "]]"],
        keywords: HashSet::from([
            "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in",
            "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
        ]),
        types: HashSet::from([
            "nil", "boolean", "number", "string", "nil", "function", "userdata", "thread", "table",
        ]),
        special: HashSet::from([
            "new_buffer",
            "load_buffer",
            "next_frame",
            "fg_rgb",
            "bg_rgb",
            "set_char",
            "get_char",
            "set_fg",
            "get_fg",
            "set_bg",
            "get_bg",
            "print",
            "gotoxy",
            "set_layer_position",
            "get_layer_position",
            "set_layer_visible",
            "get_layer_visible",
            "clear",
        ]),
    }
}
