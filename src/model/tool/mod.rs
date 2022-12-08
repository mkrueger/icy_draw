use std::{rc::Rc, cell::RefCell};

pub use super::{Editor, Event, Position};

mod brush_tool;
mod click_tool;
mod draw_shape_tool;
mod erase_tool;
mod fill_tool;
mod font_tool;
mod paint_tool;
mod select_tool;

pub trait Tool
{
    fn get_icon_name(&self) -> &'static str;/* 
    fn add_tool_page(&self, window: &ApplicationWindow,parent: &mut gtk4::Box);

    fn handle_key(&self, editor: &mut Editor, key: Key, _key_code: u32, _modifier: ModifierType) -> Event
    {
        match key {
            Key::Down => {
                editor.set_cursor(editor.cursor.pos.x, editor.cursor.pos.y + 1);
            }
            Key::Up => {
                editor.set_cursor(editor.cursor.pos.x, editor.cursor.pos.y - 1);
            }
            Key::Left => {
                editor.set_cursor(editor.cursor.pos.x - 1, editor.cursor.pos.y);
            }
            Key::Right => {
                editor.set_cursor(editor.cursor.pos.x + 1, editor.cursor.pos.y);
            }
            
            Key::Page_Down |
            
            Key::Page_Up => {
                // TODO
            }
            
            Key::Home | Key::KP_Home => {
                editor.set_cursor(0, editor.cursor.pos.y);
            }
            
            Key::End | Key::KP_End => {
                editor.set_cursor(editor.buf.width as i32 - 1, editor.cursor.pos.y);
            }

            Key::Return | Key::KP_Enter => {
                editor.set_cursor(0,editor.cursor.pos.y + 1);
            }

            /*
            
            							case SDLK_DELETE:
								for (int i = caret.getLogicalX(); i < getCurrentBuffer()->getWidth(); ++i) {
									getCurrentBuffer()->getCharacter(caret.getLogicalY(), i) = getCurrentBuffer()->getCharacter(caret.getLogicalY(), i + 1);
									getCurrentBuffer()->getAttribute(caret.getLogicalY(), i) = getCurrentBuffer()->getAttribute(caret.getLogicalY(), i + 1);
								}
								getCurrentBuffer()->getCharacter(caret.getLogicalY(), getCurrentBuffer()->getWidth() - 1) = ' ';
								getCurrentBuffer()->getAttribute(caret.getLogicalY(), getCurrentBuffer()->getWidth() - 1) = 7;
								break;
							case SDLK_INSERT:
								caret.insertMode() = !caret.insertMode();
								break;
							case SDLK_BACKSPACE:
								if (caret.getLogicalX()>0){
									if (caret.fontMode() && FontTyped && cpos > 0)  {
										caret.getX() -= CursorPos[cpos] - 1;
										for (a=0;a<=CursorPos[cpos];a++)
										for (b=0;b<=FontLibrary::getInstance().maxY;b++) {
											getCurrentBuffer()->getCharacter(caret.getLogicalY() + b, caret.getLogicalX()+a) = getUndoBuffer()->getCharacter(caret.getLogicalY() + b, caret.getLogicalX()+a);
											getCurrentBuffer()->getAttribute(caret.getLogicalY() + b, caret.getLogicalX()+a) = getUndoBuffer()->getAttribute(caret.getLogicalY() + b, caret.getLogicalX()+a);
										}
										cpos--;
									} else {	
										cpos=0;
										caret.getX()--;
										if (caret.insertMode()) {
											for (int i = caret.getLogicalX(); i < getCurrentBuffer()->getWidth(); ++i) {
												getCurrentBuffer()->getCharacter(caret.getLogicalY(), i) = getCurrentBuffer()->getCharacter(caret.getLogicalY(), i + 1);
												getCurrentBuffer()->getAttribute(caret.getLogicalY(), i) = getCurrentBuffer()->getAttribute(caret.getLogicalY(), i + 1);
											}
											getCurrentBuffer()->getCharacter(caret.getLogicalY(), getCurrentBuffer()->getWidth() - 1) = ' ';
											getCurrentBuffer()->getAttribute(caret.getLogicalY(), getCurrentBuffer()->getWidth() - 1) = 7;
										} else  {
											getCurrentBuffer()->getCharacter(caret.getLogicalY(), getCurrentBuffer()->getWidth() - 1) = ' ';
											getCurrentBuffer()->getAttribute(caret.getLogicalY(), getCurrentBuffer()->getWidth() - 1) = 7;
										} 
									}
								}
								break;

            */

            _ => { 
                if let Some(key) = key.to_unicode() {
                    
                    if key.len_utf8() == 1 {
                        let mut dst = [0];
                        key.encode_utf8(&mut dst);
                        
                        editor.buf.set_char(editor.cursor.pos, crate::model::DosChar {
                            char_code: dst[0],
                            attribute: editor.cursor.attr,
                        });
                        editor.set_cursor(editor.cursor.pos.x + 1, editor.cursor.pos.y);
                    }
                }
            }
                /* 
                    a = event.key.keysym.unicode;
                    if (caret.fontMode() && a > 32 && a < 127) {
                         renderFontCharacter(a);
                    } else  {
                    if (caret.fontMode() && FontTyped) {
                        cpos++;
                        CursorPos[cpos]=2;
                    }
                    if (caret.eliteMode()) {
                        typeCharacter(translate(a)); 
                    } else {
                        typeCharacter(a);
                    }
                */
        }
        Event::None
    }

*/
fn handle_click(&self, _editor: Rc<RefCell<Editor>>, _button: u32, _pos: Position) -> Event {
    Event::None
}
/* 
    fn handle_drag_begin(&self, _editor: &mut Editor, _start: Position, _cur: Position) -> Event {
        Event::None
    }

    fn handle_drag(&self, _editor: &mut Editor, _start: Position, _cur: Position) -> Event {
        Event::None
    }

    fn handle_drag_end(&self, _editor: &mut Editor, _start: Position, _cur: Position) -> Event {
        Event::None
    }*/
}

pub static mut FONT_TOOL: font_tool::FontTool = font_tool::FontTool { fonts: Vec::new(), selected_font: -1  };
pub static mut TOOLS: Vec<&dyn Tool> = Vec::new();

pub fn init_tools()
{
   /*  unsafe {
        FONT_TOOL.load_fonts();
        TOOLS.push(&click_tool::ClickTool {});
        TOOLS.push(&select_tool::SelectTool {});
        TOOLS.push(&paint_tool::PaintTool{});
        TOOLS.push(&brush_tool::BrushTool{});
        TOOLS.push(&erase_tool::EraseTool{});
        TOOLS.push(&draw_shape_tool::DrawShapeTool{});
        TOOLS.push(&fill_tool::FillTool{});
        TOOLS.push(&FONT_TOOL);
    }*/
}