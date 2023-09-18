use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Instant,
};

use eframe::{
    egui::{self, Id, ImageButton, RichText, Slider, TextEdit, TopBottomPanel},
    epaint::{mutex::Mutex, Vec2},
};
use egui_code_editor::{CodeEditor, Syntax};
use i18n_embed_fl::fl;
use icy_engine::{Buffer, EngineResult, SaveOptions, Size, TextPane};
use icy_engine_egui::{show_terminal_area, BufferView, MonitorSettings};

use crate::{
    model::Tool, AnsiEditor, ClipboardHandler, Document, DocumentOptions, Message, TerminalResult,
    UndoHandler,
};

use self::animator::Animator;
mod highlighting;

mod animator;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    Gif,
    Ansi,
}
pub struct AnimationEditor {
    id: usize,

    undostack: usize,

    txt: String,
    buffer_view: Arc<Mutex<BufferView>>,
    animator: Option<Arc<Mutex<Animator>>>,
    is_playing: bool,
    is_loop: bool,
    instant: Instant,
    cur_frame: usize,
    frame_count: usize,
    error: String,

    parent_path: Option<PathBuf>,
    export_path: PathBuf,
    export_type: ExportType,

    update_thread: Option<thread::JoinHandle<mlua::Result<Arc<Mutex<Animator>>>>>,
    shedule_update: bool,
    last_update: Instant,
}

impl AnimationEditor {
    pub fn new(gl: &Arc<glow::Context>, id: usize, path: &Path, txt: String) -> Self {
        let mut buffer = Buffer::new(Size::new(80, 25));
        buffer.is_terminal_buffer = true;
        let mut buffer_view = BufferView::from_buffer(gl, buffer, glow::NEAREST as i32);
        buffer_view.interactive = false;
        let buffer_view = Arc::new(Mutex::new(buffer_view));
        let mut frame_count = 0;
        let parent_path = path.parent().map(|p| p.to_path_buf());
        let animator = if let Ok(animator) = Animator::run(&parent_path, &txt) {
            frame_count = animator.lock().frames.len();
            Some(animator)
        } else {
            None
        };

        let export_path = path.with_extension("gif");
        let mut result = Self {
            id,
            buffer_view,
            is_playing: false,
            animator,
            txt,
            is_loop: false,
            instant: Instant::now(),
            cur_frame: 0,
            undostack: 0,
            frame_count,
            error: "".to_string(),
            export_path,
            export_type: ExportType::Gif,
            parent_path,

            update_thread: None,
            shedule_update: false,
            last_update: Instant::now(),
        };
        result.show_frame();

        result
    }

    fn show_frame(&mut self) {
        if let Some(animator) = &self.animator {
            let animator = animator.lock();
            if let Some(scene) = animator.frames.get(self.cur_frame) {
                let mut frame = Buffer::new(scene.get_size());
                frame.is_terminal_buffer = true;
                frame.layers = scene.layers.clone();
                frame.terminal_state = scene.terminal_state.clone();
                frame.palette = scene.palette.clone();
                frame.layers = scene.layers.clone();
                frame.clear_font_table();
                for f in scene.font_iter() {
                    frame.set_font(*f.0, f.1.clone());
                }

                self.buffer_view.lock().set_buffer(frame);
            }
        }
    }

    fn export(&mut self) -> TerminalResult<()> {
        match self.export_type {
            ExportType::Gif => {
                if let Ok(mut image) = File::create(&self.export_path) {
                    let size = self.buffer_view.lock().get_buffer().get_size();
                    let dim = self.buffer_view.lock().get_buffer().get_font_dimensions();
                    let width = (size.width * dim.width) as u16;
                    let height = (size.height * dim.height) as u16;

                    let Ok(mut encoder) = ::gif::Encoder::new(&mut image, width, height, &[])
                    else {
                        return Err(anyhow::anyhow!("Could not create encoder"));
                    };
                    encoder.set_repeat(::gif::Repeat::Infinite).unwrap();
                    if let Some(animator) = &self.animator {
                        let animator = animator.lock();
                        for frame in &animator.frames {
                            let mut data = frame.render_to_rgba(frame.get_rectangle());
                            let gif_frame = ::gif::Frame::from_rgba(
                                data.0.width as u16,
                                data.0.height as u16,
                                &mut data.1,
                            );
                            encoder.write_frame(&gif_frame)?;
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("Could not create file"));
                }
            }
            ExportType::Ansi => {
                let opt = SaveOptions::default();
                if let Ok(mut image) = File::create(&self.export_path) {
                    if let Some(animator) = &self.animator {
                        let animator = animator.lock();
                        for frame in &animator.frames {
                            let _ = image.write_all(b"\x1BP0;1;0!z\x1b[2J\x1b[0; D");
                            let _ = image.write_all(&frame.to_bytes("ans", &opt)?);
                            let _ = image.write_all(b"\x1B\\ - next frame -  \x1b[0*z");
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("Could not create file"));
                }
            }
        }
        Ok(())
    }
}

impl ClipboardHandler for AnimationEditor {
    fn can_copy(&self) -> bool {
        false
    }

    fn copy(&mut self) -> EngineResult<()> {
        Ok(())
    }

    fn can_paste(&self) -> bool {
        false
    }

    fn paste(&mut self) -> EngineResult<()> {
        Ok(())
    }
}

impl UndoHandler for AnimationEditor {
    fn undo_description(&self) -> Option<String> {
        None
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn undo(&mut self) -> EngineResult<Option<Message>> {
        Ok(None)
    }

    fn redo_description(&self) -> Option<String> {
        None
    }

    fn can_redo(&self) -> bool {
        false
    }

    fn redo(&mut self) -> EngineResult<Option<Message>> {
        Ok(None)
    }
}

impl Document for AnimationEditor {
    fn default_extension(&self) -> &'static str {
        "icyanim"
    }

    fn undo_stack_len(&self) -> usize {
        self.undostack
    }

    fn show_ui(
        &mut self,
        ui: &mut eframe::egui::Ui,
        _cur_tool: &mut Box<dyn Tool>,
        _selected_tool: usize,
        options: &DocumentOptions,
    ) -> Option<Message> {
        let mut message = None;
        egui::SidePanel::left("movie_panel")
            .exact_width(ui.available_width() / 2.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                TopBottomPanel::top("move_top_panel")
                    .exact_height(24.)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if !self.error.is_empty() {
                                ui.set_enabled(false);
                            }
                            let size_points = Vec2::new(22.0, 22.0);
                            if self.is_playing {
                                if ui
                                    .add(ImageButton::new(
                                        crate::PAUSE_SVG.texture_id(ui.ctx()),
                                        size_points,
                                    ))
                                    .clicked()
                                {
                                    self.is_playing = false;
                                }
                            } else {
                                let id = if self.cur_frame + 1 < self.frame_count {
                                    crate::PLAY_SVG.texture_id(ui.ctx())
                                } else {
                                    crate::REPLAY_SVG.texture_id(ui.ctx())
                                };
                                if ui.add(ImageButton::new(id, size_points)).clicked() {
                                    self.is_playing = true;
                                    self.instant = Instant::now();
                                    self.cur_frame = 0;
                                    self.show_frame();
                                }
                            }
                            if ui
                                .add_enabled(
                                    self.cur_frame + 1 < self.frame_count,
                                    ImageButton::new(
                                        crate::SKIP_NEXT_SVG.texture_id(ui.ctx()),
                                        size_points,
                                    ),
                                )
                                .clicked()
                            {
                                self.cur_frame = self.frame_count - 1;
                                self.show_frame();
                            }
                            if ui
                                .add(
                                    ImageButton::new(
                                        crate::REPEAT_SVG.texture_id(ui.ctx()),
                                        size_points,
                                    )
                                    .selected(self.is_loop),
                                )
                                .clicked()
                            {
                                self.is_loop = !self.is_loop;
                            }

                            let mut cf = self.cur_frame + 1;
                            if self.frame_count > 0
                                && ui
                                    .add(
                                        Slider::new(&mut cf, 1..=self.frame_count)
                                            .text(format!("of {}", self.frame_count)),
                                    )
                                    .changed()
                            {
                                self.cur_frame = cf - 1;
                                self.show_frame();
                            }
                        });
                    });

                TopBottomPanel::bottom("export_panel")
                    .exact_height(100.)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(fl!(crate::LANGUAGE_LOADER, "animation_editor_path_label"));
                            let mut path_edit = self.export_path.to_str().unwrap().to_string();
                            let response = ui.add(
                                //    ui.available_size(),
                                TextEdit::singleline(&mut path_edit),
                            );
                            if response.changed() {
                                self.export_path = path_edit.into();
                            }

                            if ui
                                .selectable_label(
                                    self.export_type == ExportType::Gif,
                                    fl!(crate::LANGUAGE_LOADER, "animation_editor_gif_label"),
                                )
                                .clicked()
                            {
                                self.export_type = ExportType::Gif;
                                self.export_path.set_extension("gif");
                            }
                            if ui
                                .selectable_label(
                                    self.export_type == ExportType::Ansi,
                                    fl!(crate::LANGUAGE_LOADER, "animation_editor_ansi_label"),
                                )
                                .clicked()
                            {
                                self.export_type = ExportType::Ansi;
                                self.export_path.set_extension("ans");
                            }
                        });
                        ui.add_space(8.0);
                        if ui
                            .button(fl!(
                                crate::LANGUAGE_LOADER,
                                "animation_editor_export_button"
                            ))
                            .clicked()
                        {
                            if let Err(err) = self.export() {
                                message =
                                    Some(Message::ShowError(format!("Could not export: {}", err)));
                            }
                        }
                    });

                /*
                if ui.button("Export").clicked() {

                    let mut out_vec = Vec::new();

                    self.is_playing = false;
                }*/

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let mut scale = options.get_scale();
                    if self.buffer_view.lock().get_buffer().use_aspect_ratio() {
                        scale.y *= 1.35;
                    }
                    let opt = icy_engine_egui::TerminalOptions {
                        stick_to_bottom: false,
                        scale: Some(Vec2::new(1.0, 1.0)),
                        settings: MonitorSettings {
                            ..Default::default()
                        },
                        id: Some(Id::new(self.id + 20000)),
                        ..Default::default()
                    };
                    self.buffer_view.lock().get_caret_mut().is_visible = false;
                    let (_, _) = show_terminal_area(ui, self.buffer_view.clone(), opt);
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let r = CodeEditor::default()
                .id_source("code editor")
                .with_rows(12)
                .with_fontsize(14.0)
                .with_theme(if ui.style().visuals.dark_mode {
                    egui_code_editor::ColorTheme::GITHUB_DARK
                } else {
                    egui_code_editor::ColorTheme::GITHUB_LIGHT
                })
                .with_syntax(highlighting::lua())
                .with_numlines(true)
                .show(ui, &mut self.txt);
            if !self.error.is_empty() {
                TopBottomPanel::bottom("code_error_bottom_panel")
                    .exact_height(100.)
                    .show_inside(ui, |ui| {
                        ui.colored_label(
                            ui.style().visuals.error_fg_color,
                            RichText::new(&self.error).small(),
                        );
                    });
            }

            if self.shedule_update && self.last_update.elapsed().as_millis() > 1000 {
                self.shedule_update = false;

                let path = self.parent_path.clone();
                let txt = self.txt.clone();
                self.update_thread = Some(thread::spawn(move || Animator::run(&path, &txt)));
            }

            if let Some(handle) = &self.update_thread {
                if handle.is_finished() {
                    if let Ok(result) = self.update_thread.take().unwrap().join() {
                        match result {
                            Ok(animator) => {
                                self.frame_count = animator.lock().frames.len();
                                self.cur_frame = self.cur_frame.min(self.frame_count.max(1) - 1);
                                self.animator = Some(animator);
                                self.show_frame();
                                self.error = "".to_string();
                            }
                            Err(e) => {
                                self.error = format!("Error: {}", e);
                            }
                        }
                    }
                }
            }

            if r.response.changed {
                self.shedule_update = true;
                self.last_update = Instant::now();
                self.undostack += 1;
            }
        });

        if self.is_playing && self.instant.elapsed().as_millis() > 100 {
            self.cur_frame += 1;
            if self.cur_frame >= self.frame_count {
                if self.is_loop {
                    self.cur_frame = 0;
                } else {
                    self.is_playing = false;
                }
            }
            self.instant = Instant::now();
            self.show_frame();
        }

        message
    }

    fn get_bytes(&mut self, _path: &Path) -> TerminalResult<Vec<u8>> {
        Ok(self.txt.as_bytes().to_vec())
    }

    fn get_ansi_editor_mut(&mut self) -> Option<&mut AnsiEditor> {
        None
    }

    fn get_ansi_editor(&self) -> Option<&AnsiEditor> {
        None
    }

    fn destroy(&self, gl: &glow::Context) -> Option<Message> {
        self.buffer_view.lock().destroy(gl);
        None
    }
}
