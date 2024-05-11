use std::process::ExitCode;

use libmcpatch::program;

fn main() -> ExitCode {
    ExitCode::from(program().0 as u8)
    
    // ExitCode::from(0)
}