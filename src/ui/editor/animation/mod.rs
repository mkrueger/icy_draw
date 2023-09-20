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
use icy_engine_egui::{animations::Animator, show_terminal_area, BufferView, MonitorSettings};

use crate::{
    model::Tool, AnsiEditor, ClipboardHandler, Document, DocumentOptions, Message, TerminalResult,
    UndoHandler, SETTINGS,
};

mod highlighting;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    Gif,
    Ansi,
}
pub struct AnimationEditor {
    gl: Arc<glow::Context>,
    id: usize,

    undostack: usize,

    txt: String,
    buffer_view: Arc<Mutex<BufferView>>,
    animator: Option<Arc<Mutex<Animator>>>,
    error: String,

    parent_path: Option<PathBuf>,
    export_path: PathBuf,
    export_type: ExportType,

    current_monitor_settings: MonitorSettings,

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
        let parent_path = path.parent().map(|p| p.to_path_buf());
        let mut monitor_settings = MonitorSettings::default();
        let animator = if let Ok(animator) = Animator::run(&parent_path, &txt) {
            monitor_settings = animator.lock().display_frame(buffer_view.clone());
            Some(animator)
        } else {
            None
        };

        let export_path = path.with_extension("gif");
        Self {
            gl: gl.clone(),
            id,
            buffer_view,
            animator,
            txt,
            undostack: 0,
            error: "".to_string(),
            export_path,
            export_type: ExportType::Gif,
            parent_path,

            current_monitor_settings: monitor_settings,
            update_thread: None,
            shedule_update: false,
            last_update: Instant::now(),
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

                    let frame_count = self.animator.as_ref().unwrap().lock().frames.len();

                    for frame in 0..frame_count {
                        self.animator.as_ref().unwrap().lock().set_cur_frame(frame);
                        self.animator
                            .as_ref()
                            .unwrap()
                            .lock()
                            .display_frame(self.buffer_view.clone());
                        let opt = icy_engine_egui::TerminalOptions {
                            stick_to_bottom: false,
                            scale: Some(Vec2::new(1.0, 1.0)),
                            monitor_settings: unsafe { SETTINGS.monitor_settings.clone() },
                            marker_settings: unsafe { SETTINGS.marker_settings.clone() },

                            id: Some(Id::new(self.id + 20000)),
                            ..Default::default()
                        };

                        let (size, mut data) =
                            self.buffer_view.lock().render_buffer(&self.gl, &opt);

                        let gif_frame =
                            ::gif::Frame::from_rgba(size.x as u16, size.y as u16, &mut data);
                        encoder.write_frame(&gif_frame)?;
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
                        for (frame, _, _) in &animator.frames {
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
                            if let Some(animator) = &mut self.animator {
                                let animator = &mut animator.lock();
                                let frame_count = animator.frames.len();
                                if animator.is_playing() {
                                    if ui
                                        .add(ImageButton::new(
                                            crate::PAUSE_SVG.texture_id(ui.ctx()),
                                            size_points,
                                        ))
                                        .clicked()
                                    {
                                        animator.set_is_playing(false);
                                    }
                                } else {
                                    let id = if animator.get_cur_frame() + 1 < frame_count {
                                        crate::PLAY_SVG.texture_id(ui.ctx())
                                    } else {
                                        crate::REPLAY_SVG.texture_id(ui.ctx())
                                    };
                                    if ui.add(ImageButton::new(id, size_points)).clicked() {
                                        self.current_monitor_settings =
                                            animator.start_playback(self.buffer_view.clone());
                                    }
                                }
                                if ui
                                    .add_enabled(
                                        animator.get_cur_frame() + 1 < frame_count,
                                        ImageButton::new(
                                            crate::SKIP_NEXT_SVG.texture_id(ui.ctx()),
                                            size_points,
                                        ),
                                    )
                                    .clicked()
                                {
                                    animator.set_cur_frame(frame_count - 1);
                                    self.current_monitor_settings =
                                        animator.display_frame(self.buffer_view.clone());
                                }
                                let is_loop = animator.get_is_loop();
                                if ui
                                    .add(
                                        ImageButton::new(
                                            crate::REPEAT_SVG.texture_id(ui.ctx()),
                                            size_points,
                                        )
                                        .selected(is_loop),
                                    )
                                    .clicked()
                                {
                                    animator.set_is_loop(!is_loop);
                                }

                                let mut cf = animator.get_cur_frame() + 1;
                                if frame_count > 0
                                    && ui
                                        .add(
                                            Slider::new(&mut cf, 1..=frame_count)
                                                .text(format!("of {}", frame_count)),
                                        )
                                        .changed()
                                {
                                    animator.set_cur_frame(cf - 1);
                                    self.current_monitor_settings =
                                        animator.display_frame(self.buffer_view.clone());
                                }
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
                        monitor_settings: self.current_monitor_settings.clone(),

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
                                self.current_monitor_settings =
                                    animator.lock().display_frame(self.buffer_view.clone());
                                self.animator = Some(animator);
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
        if let Some(animator) = &self.animator {
            animator.lock().update_frame(self.buffer_view.clone());
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
