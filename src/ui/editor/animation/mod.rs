use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use crate::{model::Tool, AnsiEditor, ClipboardHandler, Document, DocumentOptions, Message, TerminalResult, UndoHandler};
use eframe::{
    egui::{self, Id, ImageButton, RichText, Slider, TextEdit, TopBottomPanel},
    epaint::Vec2,
};
use egui::Image;
use egui_code_editor::{CodeEditor, Syntax};
use i18n_embed_fl::fl;
use icy_engine::{Buffer, EngineResult, Size, TextPane};
use icy_engine_egui::{animations::Animator, show_terminal_area, BufferView, MonitorSettings};

mod highlighting;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    Gif,
}
pub struct AnimationEditor {
    gl: Arc<glow::Context>,
    id: usize,

    undostack: usize,

    txt: String,
    buffer_view: Arc<eframe::epaint::mutex::Mutex<BufferView>>,
    animator: Arc<std::sync::Mutex<Animator>>,
    next_animator: Option<Arc<std::sync::Mutex<Animator>>>,
    set_frame: usize,

    parent_path: Option<PathBuf>,
    export_path: PathBuf,
    export_type: ExportType,

    first_frame: bool,

    shedule_update: bool,
    last_update: Instant,
}

impl AnimationEditor {
    pub fn new(gl: &Arc<glow::Context>, id: usize, path: &Path, txt: String) -> Self {
        let mut buffer = Buffer::new(Size::new(80, 25));
        buffer.is_terminal_buffer = false;
        let mut buffer_view = BufferView::from_buffer(gl, buffer);
        buffer_view.interactive = false;
        let buffer_view = Arc::new(eframe::epaint::mutex::Mutex::new(buffer_view));
        let parent_path = path.parent().map(|p| p.to_path_buf());
        let animator = Animator::run(&parent_path, txt.clone());
        let export_path = path.with_extension("gif");
        Self {
            gl: gl.clone(),
            id,
            buffer_view,
            animator,
            txt,
            undostack: 0,
            export_path,
            export_type: ExportType::Gif,
            parent_path,
            set_frame: 0,
            next_animator: None,
            shedule_update: false,
            last_update: Instant::now(),
            first_frame: true,
        }
    }

    fn export(&mut self) -> TerminalResult<()> {
        match self.export_type {
            ExportType::Gif => {
                if let Ok(mut image) = File::create(&self.export_path) {
                    if self.animator.lock().unwrap().success() {
                        let size = self.buffer_view.lock().get_buffer().get_size();
                        let dim = self.buffer_view.lock().get_buffer().get_font_dimensions();
                        let width = (size.width * dim.width) as u16;
                        let height = (size.height * dim.height) as u16;

                        let Ok(mut encoder) = ::gif::Encoder::new(&mut image, width, height, &[]) else {
                            return Err(anyhow::anyhow!("Could not create encoder"));
                        };
                        encoder.set_repeat(::gif::Repeat::Infinite).unwrap();

                        let frame_count = self.animator.lock().unwrap().frames.len();

                        for frame in 0..frame_count {
                            self.animator.lock().unwrap().set_cur_frame(frame);
                            let monitor_settings = self.animator.lock().unwrap().display_frame(self.buffer_view.clone());
                            let opt = icy_engine_egui::TerminalOptions {
                                stick_to_bottom: false,
                                scale: Some(Vec2::new(1.0, 1.0)),
                                monitor_settings,

                                id: Some(Id::new(self.id + 20000)),
                                ..Default::default()
                            };

                            let (size, mut data) = self.buffer_view.lock().render_buffer(&self.gl, &opt);

                            let gif_frame = ::gif::Frame::from_rgba(size.x as u16, size.y as u16, &mut data);
                            encoder.write_frame(&gif_frame)?;
                        }
                    } else {
                        return Err(anyhow::anyhow!("Could not create file"));
                    }
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

    fn show_ui(&mut self, ui: &mut eframe::egui::Ui, _cur_tool: &mut Box<dyn Tool>, _selected_tool: usize, options: &DocumentOptions) -> Option<Message> {
        let mut message = None;

        if self.first_frame && self.animator.lock().unwrap().success() {
            let animator = &mut self.animator.lock().unwrap();
            let frame_count = animator.frames.len();
            if frame_count > 0 {
                animator.set_cur_frame(self.set_frame);
                animator.display_frame(self.buffer_view.clone());
            }
            self.first_frame = false;
        }
        if let Some(next) = &self.next_animator {
            if next.lock().unwrap().success() {
                self.animator = next.clone();
                self.next_animator = None;
                let animator = &mut self.animator.lock().unwrap();
                animator.set_cur_frame(self.set_frame);
                animator.display_frame(self.buffer_view.clone());
            }
        }
        egui::SidePanel::left("movie_panel")
            .exact_width(ui.available_width() / 2.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                TopBottomPanel::top("move_top_panel").exact_height(34.).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if !self.animator.lock().unwrap().error.is_empty() {
                            ui.set_enabled(false);
                        }

                        if self.animator.lock().unwrap().success() {
                            let animator = &mut self.animator.lock().unwrap();
                            let frame_count = animator.frames.len();
                            if animator.is_playing() {
                                if ui.add(ImageButton::new(crate::PAUSE_SVG.clone())).clicked() {
                                    animator.set_is_playing(false);
                                }
                            } else {
                                let image: &Image<'static> = if animator.get_cur_frame() + 1 < frame_count {
                                    &crate::PLAY_SVG
                                } else {
                                    &crate::REPLAY_SVG
                                };
                                if ui.add(ImageButton::new(image.clone())).clicked() {
                                    if animator.get_cur_frame() + 1 >= frame_count {
                                        animator.set_cur_frame(0);
                                    }
                                    animator.start_playback(self.buffer_view.clone());
                                }
                            }
                            if ui
                                .add_enabled(animator.get_cur_frame() + 1 < frame_count, ImageButton::new(crate::SKIP_NEXT_SVG.clone()))
                                .clicked()
                            {
                                animator.set_cur_frame(frame_count - 1);
                                animator.display_frame(self.buffer_view.clone());
                            }
                            let is_loop = animator.get_is_loop();
                            if ui.add(ImageButton::new(crate::REPEAT_SVG.clone()).selected(is_loop)).clicked() {
                                animator.set_is_loop(!is_loop);
                            }

                            let mut cf = animator.get_cur_frame() + 1;
                            if frame_count > 0 && ui.add(Slider::new(&mut cf, 1..=frame_count).text(format!("of {}", frame_count))).changed() {
                                animator.set_cur_frame(cf - 1);
                                animator.display_frame(self.buffer_view.clone());
                            }

                            if ui
                                .add_enabled(animator.get_cur_frame() > 0, ImageButton::new(crate::NAVIGATE_PREV.clone()))
                                .clicked()
                            {
                                let cf = animator.get_cur_frame() - 1;
                                animator.set_cur_frame(cf);
                                animator.display_frame(self.buffer_view.clone());
                            }

                            if ui
                                .add_enabled(
                                    animator.get_cur_frame() < animator.frames.len() - 1,
                                    ImageButton::new(crate::NAVIGATE_NEXT.clone()),
                                )
                                .clicked()
                            {
                                let cf = animator.get_cur_frame() + 1;
                                animator.set_cur_frame(cf);
                                animator.display_frame(self.buffer_view.clone());
                            }
                        }
                    });
                });

                TopBottomPanel::bottom("export_panel").exact_height(100.).show_inside(ui, |ui| {
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
                            .selectable_label(self.export_type == ExportType::Gif, fl!(crate::LANGUAGE_LOADER, "animation_editor_gif_label"))
                            .clicked()
                        {
                            self.export_type = ExportType::Gif;
                            self.export_path.set_extension("gif");
                        }
                    });
                    ui.add_space(8.0);
                    if ui.button(fl!(crate::LANGUAGE_LOADER, "animation_editor_export_button")).clicked() {
                        if let Err(err) = self.export() {
                            message = Some(Message::ShowError(format!("Could not export: {}", err)));
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

                    if self.animator.lock().unwrap().success() {
                        let cur_frame = self.animator.lock().unwrap().get_cur_frame();

                        let monitor_settings = if let Some((_, settings, _)) = self.animator.lock().unwrap().frames.get(cur_frame) {
                            settings.clone()
                        } else {
                            MonitorSettings::default()
                        };

                        let opt = icy_engine_egui::TerminalOptions {
                            stick_to_bottom: false,
                            scale: Some(Vec2::new(1.0, 1.0)),
                            monitor_settings,
                            id: Some(Id::new(self.id + 20000)),
                            ..Default::default()
                        };
                        self.buffer_view.lock().get_caret_mut().set_is_visible(false);
                        let (_, _) = show_terminal_area(ui, self.buffer_view.clone(), opt);
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            TopBottomPanel::bottom("code_error_bottom_panel").exact_height(200.).show_inside(ui, |ui| {
                if !self.animator.lock().unwrap().error.is_empty() {
                    ui.colored_label(ui.style().visuals.error_fg_color, RichText::new(&self.animator.lock().unwrap().error).small());
                } else {
                    egui::ScrollArea::vertical().max_width(f32::INFINITY).show(ui, |ui| {
                        self.animator.lock().unwrap().log.iter().for_each(|line| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Frame {}:", line.frame)).strong());
                                ui.label(RichText::new(&line.text));
                                ui.add_space(ui.available_width());
                            });
                        });
                    });
                }
            });

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

            if self.shedule_update && self.last_update.elapsed().as_millis() > 1000 {
                self.shedule_update = false;

                let path = self.parent_path.clone();
                let txt = self.txt.clone();
                self.set_frame = self.animator.lock().unwrap().get_cur_frame();
                self.next_animator = Some(Animator::run(&path, txt));
            }

            if r.response.changed {
                self.shedule_update = true;
                self.last_update = Instant::now();
                self.undostack += 1;
            }
        });
        let buffer_view = self.buffer_view.clone();
        if self.animator.lock().unwrap().success() {
            self.animator.lock().unwrap().update_frame(buffer_view);
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
