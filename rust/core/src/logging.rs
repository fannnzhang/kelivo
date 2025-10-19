#[allow(dead_code)]
pub fn trace(message: &str) {
    #[cfg(feature = "logging")]
    eprintln!("[kelivo::trace] {message}");

    #[cfg(not(feature = "logging"))]
    {
        let _ = message;
    }
}
