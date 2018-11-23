pub const DEBUG: bool = true;

macro_rules! dprintln {
    () => ({
        use console::DEBUG_LOG;
        DEBUG_LOG.push_str("\n".to_string());
    });
    ($($arg:tt)*) => ({
        use debug::DEBUG;
        use console::DEBUG_LOG;
        if DEBUG {
            DEBUG_LOG.lock().unwrap().push_str(&format!($($arg)*));
            DEBUG_LOG.lock().unwrap().push('\n')
        };
    })
}