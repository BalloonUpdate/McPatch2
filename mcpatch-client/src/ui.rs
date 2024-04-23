use std::sync::Arc;
use std::sync::RwLock;

use egui::vec2;
use egui::Align2;
use egui::Color32;
use egui::FontId;
use egui::Rect;
use egui::Rounding;
use egui::Sense;

pub struct DialogContent {
    pub title: String,
    pub content: String,
}

#[derive(Default)]
pub struct UIState {
    exit_flag: bool,
    pub label: String,
    pub progress_label: String,
    pub progress: f32,
}

pub struct AppWindow {
    state: Arc<RwLock<UIState>>,
}

impl AppWindow {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: Arc<RwLock<UIState>>) -> Self {
        AppWindow { state }
    }
}

impl eframe::App for AppWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let state = self.state.read().unwrap();
            
            // 主标题
            ui.heading(&state.label);

            const WIDTH: f32 = 300.0;
            const HEIGHT: f32 = 40.0;

            let (response, painter) = ui.allocate_painter(vec2(WIDTH, HEIGHT), Sense::hover());

            let mut progress_size = response.rect.size();
            progress_size.x *= state.progress.clamp(0.0, 1.0);
            
            painter.rect_filled(Rect::from_min_size(response.rect.min, response.rect.size()), Rounding::same(8.0), Color32::GRAY);
            painter.rect_filled(Rect::from_min_size(response.rect.min, progress_size), Rounding::same(8.0), Color32::GOLD);
            painter.text(response.rect.center(), Align2::CENTER_CENTER, &state.progress_label, FontId::monospace(16.0), Color32::WHITE);
        });

        let state = self.state.read().unwrap();

        if state.exit_flag {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}