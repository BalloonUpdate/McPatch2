use mcpatch_client::run;
use mcpatch_client::ui::AppWindow;
use mcpatch_client::StartupParameter;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let (ui_cmd, _do_not_touch_this_variable) = AppWindow::new();

    runtime.spawn(async move {
        let _params = StartupParameter {
            graphic_mode: true,
            standalone_progress: true,
            disable_log_file: false,
        };

        let _handle = tokio::spawn(run(_params, ui_cmd)).await;

        // 关闭UI
        // let mut state = state1.write().unwrap();
        // state.exit_flag = true;
    });
   
    nwg::dispatch_thread_events();
}

