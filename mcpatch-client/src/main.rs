use std::process::ExitCode;

use mcpatch_client::program;

fn main() -> ExitCode {
    ExitCode::from(program().0 as u8)
}