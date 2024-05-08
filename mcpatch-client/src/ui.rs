use std::sync::Arc;
use std::sync::Mutex;

use egui::vec2;
use egui::Align2;
use egui::Color32;
use egui::FontData;
use egui::FontDefinitions;
use egui::FontFamily;
use egui::FontId;
use egui::Rect;
use egui::Rounding;
use egui::Sense;

pub struct DialogContent {
    pub title: String,
    pub content: String,
    pub yes: String,
    pub no: Option<String>,
}

#[derive(Default)]
pub struct UIState {
    exit_flag: bool,

    pub window_title: String,
    pub label: String,
    pub progress_label: String,
    pub progress: f32,

    pub dialog: Option<DialogContent>,
}

pub struct AppWindow {
    state: Arc<Mutex<UIState>>,
}

impl AppWindow {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: Arc<Mutex<UIState>>) -> Self {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert("simhei".to_owned(), FontData::from_static(include_bytes!("simhei.ttf")));
        // fonts.font_data.insert("XiaolaiSC".to_owned(), FontData::from_static(include_bytes!("XiaolaiSC-Regular.ttf")));

        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "simhei".to_owned());
        
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .push("simhei".to_owned());

        _cc.egui_ctx.set_fonts(fonts);

        AppWindow { state }
    }
}

impl eframe::App for AppWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();
        let mut close_dialog = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(dialog) = &state.dialog {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(dialog.title.to_owned()));

                ui.label(&dialog.content);

                if ui.button("Yes").clicked() {
                    close_dialog = true;
                }
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(state.window_title.to_owned()));

                // 主标题
                ui.heading(&state.label);

                const WIDTH: f32 = 300.0;
                const HEIGHT: f32 = 40.0;

                let (response, painter) = ui.allocate_painter(vec2(WIDTH, HEIGHT), Sense::hover());

                let mut progress_size = response.rect.size();
                progress_size.x *= state.progress.clamp(0.0, 1.0);

                painter.rect_filled(Rect::from_min_size(response.rect.min, response.rect.size()), Rounding::same(8.0), Color32::GRAY);
                painter.rect_filled(Rect::from_min_size(response.rect.min, progress_size), Rounding::same(8.0), Color32::GOLD);
                painter.text(response.rect.center(), Align2::CENTER_CENTER, &state.progress_label, FontId::new(24.0, FontFamily::Proportional), Color32::WHITE);
            }
        });

        if ctx.input(|i| i.viewport().close_requested() && state.dialog.is_some()) {
            close_dialog = true;
        }

        if close_dialog {
            state.dialog = None;
        }

        if state.exit_flag {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}