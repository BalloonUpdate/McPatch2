use std::sync::Arc;
use std::sync::Mutex;

use mcpatch_client::run;
use mcpatch_client::ui::AppWindow;
use mcpatch_client::ui::DialogContent;
use mcpatch_client::ui::UIState;
use mcpatch_client::StartupParameter;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    
    let _params = StartupParameter {
        graphic_mode: true,
        standalone_progress: true,
        disable_log_file: false,
    };

    let state1 = Arc::new(Mutex::new(UIState::default()));
    let state2 = state1.clone();

    {
        let mut state = state1.lock().unwrap();
        state.label = "等待初始化".to_owned();
        state.progress = 1.0;
        state.progress_label = "100 / 100 已完成".to_owned();

        // state.dialog = Some(DialogContent {
        //     title: "标题".to_owned(),
        //     content: "哈哈哈哈哈".to_owned(),
        //     yes: "确定".to_owned(),
        //     no: None,
        // })
    }

    runtime.spawn(async move {
        let _handle = tokio::spawn(run(_params, state1)).await;

        // 关闭UI
        // let mut state = state1.write().unwrap();
        // state.exit_flag = true;
    });

    let mut native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Mcpatch")
            .with_inner_size([400.0, 200.0]),
        ..Default::default()
    };
    native_options.centered = true;
    native_options.follow_system_theme = true;
    
    eframe::run_native("Mcpatch", native_options, Box::new(move |cc| Box::new(AppWindow::new(cc, state2)))).unwrap();
}

