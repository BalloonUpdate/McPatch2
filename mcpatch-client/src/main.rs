use mcpatch_client::log::log_error;
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

    let ui_cmd2 = ui_cmd.clone();
    let _old_handler = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log_error("program paniked!!!");
        log_error(format!("{:#?}", info));

        let backtrace = std::backtrace::Backtrace::force_capture();
        log_error(format!("Backtrace: "));
        log_error(backtrace.to_string());

        let p = nwg::MessageParams {
            title: "Fatal error occurred",
            content: "程序出现错误已经崩溃，相关错误信息已经写入日志文件",
            buttons: nwg::MessageButtons::Ok,
            icons: nwg::MessageIcons::Error
        };

        nwg::message(&p);
        
        let ui_cmd3 = ui_cmd2.clone();
        std::thread::spawn(move || {
            ui_cmd3.sync_exit();
        });
    }));
    
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
                ui_cmd.async_exit().await;
            }
        }
    });
    
    nwg::dispatch_thread_events();
    
    // 失败说明工作线程已经结束运行了
    let _ = window_close_signal.0.send(());
    
    // 异步方法中panic会返回Err
    match runtime.block_on(work) {
        Ok(_) => (),
        Err(_e) => (),
    }
}
    
