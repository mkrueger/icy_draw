use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use eframe::epaint::mutex::Mutex;
use icy_engine::{Buffer, TextPane};
use mlua::Lua;

#[derive(Default)]
pub struct Animator {
    pub scene: Option<Buffer>,
    pub frames: Vec<Buffer>,
}

const MAX_FRAMES: usize = 4096;
impl Animator {
    pub fn next_frame(&mut self) -> mlua::Result<()> {
        // Need to limit it a bit to avoid out of memory & slowness
        // Not sure how large the number should be but it's easy to define millions of frames
        if self.frames.len() > MAX_FRAMES {
            return Err(mlua::Error::RuntimeError(
                "Maximum number of frames reached".to_string(),
            ));
        }
        if let Some(scene) = &self.scene {
            let mut frame = Buffer::new(scene.get_size());
            frame.layers = scene.layers.clone();
            frame.terminal_state = scene.terminal_state.clone();
            frame.palette = scene.palette.clone();
            frame.layers = scene.layers.clone();
            frame.clear_font_table();
            for f in scene.font_iter() {
                frame.set_font(*f.0, f.1.clone());
            }
            self.frames.push(frame);
            Ok(())
        } else {
            Err(mlua::Error::SyntaxError {
                message: "No scene set up.".to_string(),
                incomplete_input: false,
            })
        }
    }

    pub fn move_layer(&mut self, layer: usize, x: i32, y: i32) -> mlua::Result<()> {
        if let Some(scene) = &mut self.scene {
            if layer < scene.layers.len() {
                scene.layers[layer].set_offset((x, y));
                Ok(())
            } else {
                Err(mlua::Error::SyntaxError {
                    message: format!("Layer {} out of range (0..<{})", layer, scene.layers.len()),
                    incomplete_input: false,
                })
            }
        } else {
            Err(mlua::Error::SyntaxError {
                message: "No scene set up.".to_string(),
                incomplete_input: false,
            })
        }
    }

    pub fn set_layer_visible(&mut self, layer: usize, is_visible: bool) -> mlua::Result<()> {
        if let Some(scene) = &mut self.scene {
            if layer < scene.layers.len() {
                scene.layers[layer].is_visible = is_visible;
                Ok(())
            } else {
                Err(mlua::Error::SyntaxError {
                    message: format!("Layer {} out of range (0..<{})", layer, scene.layers.len()),
                    incomplete_input: false,
                })
            }
        } else {
            Err(mlua::Error::SyntaxError {
                message: "No scene set up.".to_string(),
                incomplete_input: false,
            })
        }
    }

    pub fn get_layer_visible(&mut self, layer: usize) -> mlua::Result<bool> {
        if let Some(scene) = &mut self.scene {
            if layer < scene.layers.len() {
                Ok(scene.layers[layer].is_visible)
            } else {
                Err(mlua::Error::SyntaxError {
                    message: format!("Layer {} out of range (0..<{})", layer, scene.layers.len()),
                    incomplete_input: false,
                })
            }
        } else {
            Err(mlua::Error::SyntaxError {
                message: "No scene set up.".to_string(),
                incomplete_input: false,
            })
        }
    }
    //   set_layer_visible(1, !get_layer_visible(1))
    pub fn run(parent: &Option<PathBuf>, txt: &str) -> mlua::Result<Arc<Mutex<Self>>> {
        let lua = Lua::new();
        let globals = lua.globals();
        let animator = Arc::new(Mutex::new(Animator::default()));

        let a = animator.clone();
        let parent = parent.clone();
        let load_func = lua.create_function(move |_lua, file: String| {
            let mut file_name = Path::new(&file).to_path_buf();
            if file_name.is_relative() {
                if let Some(parent) = &parent {
                    file_name = parent.join(&file_name);
                }
            }
            if !file_name.exists() {
                return Err(mlua::Error::RuntimeError(format!(
                    "File not found {}",
                    file
                )));
            }
            if let Ok(buffer) = icy_engine::Buffer::load_buffer(&file_name, true) {
                a.lock().scene = Some(buffer);
                Ok(())
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Could not load file {}",
                    file
                )))
            }
        })?;
        globals.set("load", load_func).unwrap();

        let a = animator.clone();
        let next_frame_func = lua.create_function(move |_lua, _: ()| a.lock().next_frame())?;
        globals.set("next_frame", next_frame_func).unwrap();

        let a = animator.clone();
        let next_frame_func =
            lua.create_function(move |_lua, (layer, x, y): (usize, i32, i32)| {
                a.lock().move_layer(layer, x, y)
            })?;
        globals.set("move_layer", next_frame_func).unwrap();

        let a = animator.clone();
        let next_frame_func =
            lua.create_function(move |_lua, (layer, is_visible): (usize, bool)| {
                a.lock().set_layer_visible(layer, is_visible)
            })?;
        globals.set("set_layer_visible", next_frame_func).unwrap();

        let a = animator.clone();
        let next_frame_func =
            lua.create_function(move |_lua, layer: usize| a.lock().get_layer_visible(layer))?;
        globals.set("get_layer_visible", next_frame_func).unwrap();

        lua.load(txt).exec()?;
        Ok(animator)
    }
}
