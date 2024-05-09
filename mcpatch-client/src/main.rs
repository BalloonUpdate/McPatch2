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

    let window_close_signal = tokio::sync::oneshot::channel::<()>();

    let (mut ui_cmd, _do_not_touch_this_variable) = AppWindow::new();

    let work = runtime.spawn(async move {
        let _params = StartupParameter {
            graphic_mode: true,
            standalone_progress: true,
            disable_log_file: false,
        };

        tokio::select! {
            _ = window_close_signal.1 => {
                println!("interupted!")
            },
            _ = run(_params, &mut ui_cmd) => {
                // 补发一下
                ui_cmd.exit().await;
            }
        }
    });
   
    nwg::dispatch_thread_events();

    // 失败说明工作线程已经结束运行了
    let _ = window_close_signal.0.send(());

    runtime.block_on(work).unwrap();
}

