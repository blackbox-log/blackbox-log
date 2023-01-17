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
    use crate::str::WasmStr;

    let message = info.to_string().into_boxed_str();
    let message = Box::leak(message);
    let message = WasmStr::from(&*message);

    // SAFETY: the message was leaked
    unsafe { crate::panic(message) };
}
