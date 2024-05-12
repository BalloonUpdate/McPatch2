use std::process::ExitCode;

use libmcpatch_client::program;

fn main() -> ExitCode {
    ExitCode::from(program(false).0 as u8)
    
    // ExitCode::from(0)
}