use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

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
use tokio::sync::Mutex;

pub struct DialogContent {
    pub title: String,
    pub content: String,
    pub yes: String,
    pub no: Option<String>,
}

#[derive(Default)]
pub struct UIStateData {
    exit_flag: bool,
    dialog_choice: Option<bool>,

    pub window_title: String,
    pub label: String,
    pub progress_label: String,
    pub progress: f32,

    pub dialog: Option<DialogContent>,
}

#[derive(Clone)]
pub struct UIState1 {
    inner: Arc<Mutex<UIStateData>>
}

impl UIState1 {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(UIStateData::default())) }
    }

    pub async fn exit(&self) {
        let mut lock = self.lock().await;

        lock.exit_flag = true;
    }

    pub async fn set_title(&self, title: String) {
        let mut lock = self.lock().await;

        lock.window_title = title;
    }

    pub async fn set_label(&self, label: String) {
        let mut lock = self.lock().await;

        lock.label = label;
    }

    pub async fn set_progress(&self, value: f32) {
        let mut lock = self.lock().await;

        lock.progress = value;
    }

    pub async fn set_progress_label(&self, plabel: String) {
        let mut lock = self.lock().await;

        lock.progress_label = plabel;
    }

    pub async fn popup_dialog(&self, dialog: DialogContent) -> bool {
        let mut lock = self.lock().await;

        lock.dialog_choice = None;
        lock.dialog = Some(dialog);

        drop(lock);

        loop {
            let mut lock = self.lock().await;

            if let Some(choice) = lock.dialog_choice {
                lock.dialog_choice = None;

                return choice;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    // pub fn clear_dialog(&self) {
    //     let mut lock = self.lock().await;

    //     lock.dialog = None;
    // }
}

impl Deref for UIState1 {
    type Target = Arc<Mutex<UIStateData>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct AppWindow {
    state: UIState1,
}

impl AppWindow {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: UIState1) -> Self {
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
        let mut state = self.state.blocking_lock();
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