pub mod global_config;
pub mod error;
pub mod log;
pub mod work;
pub mod network;
pub mod speed_sampler;
pub mod utils;

#[cfg(target_os = "windows")]
pub mod ui;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use shared::utility::is_running_under_cargo;

use crate::global_config::GlobalConfig;
use crate::log::log_error;
use crate::work::run;

pub struct AppContext {
    pub working_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub public_dir: PathBuf,
    pub index_file: PathBuf,
    pub config: GlobalConfig
}

pub struct StartupParameter {
    pub graphic_mode: bool,
    pub standalone_progress: bool,
    pub disable_log_file: bool,
    // pub external_config_file: String,
}

pub struct McpatchExitCode(pub i8);

pub fn program() -> McpatchExitCode {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    #[cfg(target_os = "windows")]
    {
        nwg::init().expect("Failed to init Native Windows GUI");
        nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    }

    #[cfg(target_os = "windows")]
    let window_close_signal = tokio::sync::oneshot::channel::<()>();
    
    #[cfg(target_os = "windows")]
    let (ui_cmd, _ui) = crate::ui::main_ui::MainWindow::new();
    let panic_info_captured = Arc::new(Mutex::new(Option::<String>::None));

    // 捕获异常
    let panic_info_captured2 = panic_info_captured.clone();
    let old_handler = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let text = format!("program paniked!!!\n{:#?}\nBacktrace: \n{}", _info, backtrace);

        log_error(format!("-----------\n{}-----------", text));
        *panic_info_captured2.lock().unwrap() = Some(text);

        #[cfg(target_os = "windows")]
        popup_error_dialog(_info, backtrace);

        if !is_running_under_cargo() {
            old_handler(_info);
        }
    }));

    let params = StartupParameter {
        graphic_mode: true,
        standalone_progress: true,
        disable_log_file: false,
    };
    
    // 带ui的逻辑
    #[cfg(target_os = "windows")]
    {
        // 开始执行更新逻辑
        let mut ui_cmd2 = ui_cmd.clone();
        let work = runtime.spawn(async move {
            tokio::select! {
                _ = window_close_signal.1 => McpatchExitCode(0),
                code = run(params, &mut ui_cmd2) => code
            }
        });
    
        // 守护逻辑，用于关闭ui
        let guard = runtime.spawn(async move {
            let result = work.await;
    
            // work结束运行后，无论是正常结束，还是panic导致的结束，都要关闭ui
            ui_cmd.exit().await;
    
            match result {
                Ok(code) => code,
                Err(_) => McpatchExitCode(1),
            }
        });
        
        // 开始ui事件循环
        #[cfg(target_os = "windows")]
        nwg::dispatch_thread_events();
    
        // 发送成功代表用户手动关闭了窗口
        if let Ok(_) = window_close_signal.0.send(()) {
            println!("interupted by user");
        }
        
        // guard不允许出现panic
        return runtime.block_on(guard).unwrap();
    }

    // 不带ui的逻辑
    #[cfg(not(target_os = "windows"))]
    {
        // 开始执行更新逻辑
        return runtime.block_on(run(params, ()));
    }
}

/// 报错弹框
#[cfg(target_os = "windows")]
fn popup_error_dialog(info: &std::panic::PanicHookInfo, backtrace: std::backtrace::Backtrace) {
    let mp = nwg::MessageParams {
        title: "Fatal error occurred",
        content: "程序出现错误，即将结束运行。点击确定直接退出，点击取消打印错误信息",
        buttons: nwg::MessageButtons::OkCancel,
        icons: nwg::MessageIcons::Error
    };

    match nwg::message(&mp) {
        nwg::MessageChoice::Ok => {},
        nwg::MessageChoice::Cancel => {
            nwg::error_message("Error detail", &format!("{:?}\n{}", info, backtrace));
        },
        _ => (),
    }
}