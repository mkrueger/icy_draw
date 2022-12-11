use eframe::{egui::{self, ImageButton}, epaint::Vec2};
use crate::MainWindow;

pub fn add_tool_switcher(ctx: &egui::Context, ui: &mut egui::Ui, arg: &mut MainWindow) {

    ui.horizontal_wrapped(|ui| {
        for i in 0..arg.tools.len() {
            let t = &arg.tools[i];
            if ui.add(ImageButton::new(t.get_icon_name().texture_id(ctx), Vec2::new(28., 28.)).selected(i == arg.selected_tool)).clicked() {
                arg.selected_tool = i;
            }
        }
    });
}