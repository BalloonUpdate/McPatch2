use mcpatch_client::run;
use mcpatch_client::StartupParameter;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let params = StartupParameter {
        graphic_mode: true,
        standalone_progress: true,
        disable_log_file: false,
    };
    
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(run(params));
}
