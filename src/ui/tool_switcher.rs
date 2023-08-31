use crate::MainWindow;
use eframe::{
    egui::{self, ImageButton},
    epaint::Vec2,
};

pub fn add_tool_switcher(ctx: &egui::Context, ui: &mut egui::Ui, arg: &mut MainWindow) {
    ui.horizontal_wrapped(|ui| {
        if let Ok(tools) = arg.tab_viewer.tools.lock() {
            for i in 0..tools.len() {
                let t = &tools[i];
                if ui
                    .add(
                        ImageButton::new(t.get_icon_name().texture_id(ctx), Vec2::new(28., 28.))
                            .selected(i == arg.tab_viewer.selected_tool),
                    )
                    .clicked()
                {
                    arg.tab_viewer.selected_tool = i;
                }
            }
        }
    });
}
