use client::program;

#[no_mangle]
pub extern "stdcall" fn Agent_OnLoad(_vm: usize, _options: i8, _reserved: usize) -> i32 {
    println!("Agent_OnLoad !!!!!!!!!!");

    program().0 as i32
}

#[no_mangle]
pub extern "stdcall" fn Agent_OnUnload(_vm: usize) {
    println!("Agent_OnUnload !!!!!!!!!!");
}
