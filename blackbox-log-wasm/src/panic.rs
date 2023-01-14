#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
#[link(wasm_import_module = "main")]
extern "C" {
    fn panic(len: usize, data: *const u8);
}

wasm_export! {
    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    fn set_panic_hook() {
        std::panic::set_hook(Box::new(panic_hook));
    }

    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    fn set_panic_hook() {}
}

#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
fn panic_hook(info: &std::panic::PanicInfo) {
    let message = info.to_string().into_boxed_str();
    let message = Box::leak(message);

    let ptr = message.as_ptr();
    let len = message.len();

    // SAFETY: the message was leaked
    unsafe { panic(len, ptr) };
}
