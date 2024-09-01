use std::process::ExitCode;

use client::program;

fn main() -> ExitCode {
    ExitCode::from(program().0 as u8)
}