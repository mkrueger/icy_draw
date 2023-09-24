use icy_engine::{AttributedChar, Position, TextAttribute};

const FULL_BLOCK: char = 219 as char;
const HALF_BLOCK_TOP: char = 223 as char;
const HALF_BLOCK_BOTTOM: char = 220 as char;
const HALF_BLOCK_LEFT: char = 221 as char;
const HALF_BLOCK_RIGHT: char = 222 as char;

const SPACE: char = 32 as char;
const ZERO: char = 0 as char;
const XFF: char = 255 as char;

struct HalfBlock {
    pub block: AttributedChar,
    pub is_blocky: bool,
    pub is_vertically_blocky: bool,

    pub upper_block_color: u32,
    pub lower_block_color: u32,
    pub is_top: bool,
}

impl HalfBlock {
    pub fn from(block: AttributedChar, pos: Position) -> Self {
        let is_top = pos.y % 2 == 0;

        let mut upper_block_color = 0;
        let mut lower_block_color = 0;
        let mut is_blocky = false;
        let mut is_vertically_blocky = false;
        match block.ch {
            ZERO | SPACE | XFF => {
                // all blank characters
                upper_block_color = block.attribute.get_background();
                lower_block_color = block.attribute.get_background();
                is_blocky = true;
            }
            HALF_BLOCK_BOTTOM => {
                upper_block_color = block.attribute.get_background();
                lower_block_color = block.attribute.get_foreground();
                is_blocky = true;
            }
            HALF_BLOCK_TOP => {
                upper_block_color = block.attribute.get_foreground();
                lower_block_color = block.attribute.get_background();
                is_blocky = true;
            }
            FULL_BLOCK => {
                upper_block_color = block.attribute.get_foreground();
                lower_block_color = block.attribute.get_foreground();
                is_blocky = true;
            }
            HALF_BLOCK_LEFT | HALF_BLOCK_RIGHT => {
                is_vertically_blocky = true;
            }
            _ => {
                if block.attribute.get_foreground() == block.attribute.get_background() {
                    is_blocky = true;
                    upper_block_color = block.attribute.get_foreground();
                    lower_block_color = block.attribute.get_foreground();
                } else {
                    is_blocky = false;
                }
            }
        }

        Self {
            block,
            is_top,
            is_blocky,
            is_vertically_blocky,
            upper_block_color,
            lower_block_color,
        }
    }
}

pub fn get_half_block(cur_char: AttributedChar, pos: Position, color: u32) -> AttributedChar {
    let half_block = HalfBlock::from(cur_char, pos);

    let ch = if half_block.is_blocky {
        if (half_block.is_top && half_block.lower_block_color == color)
            || (!half_block.is_top && half_block.upper_block_color == color)
        {
            AttributedChar::new(FULL_BLOCK, TextAttribute::new(color, 0))
        } else if half_block.is_top {
            AttributedChar::new(
                HALF_BLOCK_TOP,
                TextAttribute::new(color, half_block.lower_block_color),
            )
        } else {
            AttributedChar::new(
                HALF_BLOCK_BOTTOM,
                TextAttribute::new(color, half_block.upper_block_color),
            )
        }
    } else if half_block.is_top {
        AttributedChar::new(
            HALF_BLOCK_TOP,
            TextAttribute::new(color, half_block.block.attribute.get_background()),
        )
    } else {
        AttributedChar::new(
            HALF_BLOCK_BOTTOM,
            TextAttribute::new(color, half_block.block.attribute.get_background()),
        )
    };
    optimize_block(ch, &half_block)
}

fn flip_colors(attribute: icy_engine::TextAttribute) -> icy_engine::TextAttribute {
    let mut result = attribute;
    result.set_foreground(attribute.get_background());
    result.set_background(attribute.get_foreground());
    result
}

fn optimize_block(mut block: AttributedChar, half_block: &HalfBlock) -> AttributedChar {
    if block.attribute.get_foreground() == 0 {
        if block.attribute.get_background() == 0 || block.ch == FULL_BLOCK {
            block.ch = ' ';
            return block;
        }
        match block.ch {
            HALF_BLOCK_BOTTOM => {
                return AttributedChar::new(HALF_BLOCK_TOP, flip_colors(block.attribute));
            }
            HALF_BLOCK_TOP => {
                return AttributedChar::new(HALF_BLOCK_BOTTOM, flip_colors(block.attribute));
            }
            _ => {}
        }
    } else if block.attribute.get_foreground() < 8 && block.attribute.get_background() >= 8 {
        if half_block.is_blocky {
            match block.ch {
                HALF_BLOCK_BOTTOM => {
                    return AttributedChar::new(HALF_BLOCK_TOP, flip_colors(block.attribute));
                }
                HALF_BLOCK_TOP => {
                    return AttributedChar::new(HALF_BLOCK_BOTTOM, flip_colors(block.attribute));
                }
                _ => {}
            }
        } else if half_block.is_vertically_blocky {
            match block.ch {
                HALF_BLOCK_LEFT => {
                    return AttributedChar::new(HALF_BLOCK_RIGHT, flip_colors(block.attribute));
                }
                HALF_BLOCK_RIGHT => {
                    return AttributedChar::new(HALF_BLOCK_LEFT, flip_colors(block.attribute));
                }
                _ => {}
            }
        }
    }
    block
}
