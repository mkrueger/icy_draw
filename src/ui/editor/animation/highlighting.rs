use egui_code_editor::ColorTheme;

use super::Syntax;
use std::collections::HashSet;

#[must_use]
pub fn lua() -> Syntax {
    Syntax {
        language: "Lua",
        case_sensitive: true,
        comment: "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@",
        comment_multiline: ["--[[", "]]--"],
        keywords: HashSet::from([
            "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in",
            "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
        ]),
        types: HashSet::from([
            "nil", "boolean", "number", "string", "nil", "function", "userdata", "thread", "table",
        ]),
        special: HashSet::from([
            "load",
            "next_frame",
            "move_layer",
            "set_layer_visible",
            "get_layer_visible",
        ]),
    }
}

pub const DARK: ColorTheme = ColorTheme {
    name: "Dark",
    dark: true,
    bg: "#1c1e1f",
    cursor: "#eeeeec",
    selection: "#245176",
    comments: "#7a976b",
    functions: "#eeeeec",
    keywords: "#729fcf",
    literals: "#eeeeec",
    numerics: "#ad7fa8",
    punctuation: "#d3d7cf",
    strs: "#E6DB74",
    types: "#4ec9b0",
    special: "#b8bb26",
};

pub const LIGHT: ColorTheme = ColorTheme {
    name: "Light",
    dark: false,
    bg: "#ffffff",
    cursor: "#222222",
    selection: "#35456f",
    comments: "#888a85",
    functions: "#222222",
    keywords: "#009695",
    literals: "#db7100",
    numerics: "#db7100",
    punctuation: "#222222",
    strs: "#222222",
    types: "#3465a4",
    special: "#4e9a06",
};
