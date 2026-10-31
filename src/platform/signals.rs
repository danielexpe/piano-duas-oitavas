use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::AppError;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_signal: libc::c_int) {
    SHUTDOWN.store(true, Ordering::SeqCst);
}

pub fn install() -> Result<(), AppError> {
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        // SAFETY: `on_signal` has the C signal-handler signature and only
        // stores to an atomic, which is async-signal-safe.
        let handler = on_signal as *const () as libc::sighandler_t;
        let previous = unsafe { libc::signal(signal, handler) };
        if previous == libc::SIG_ERR {
            return Err(AppError::Signals(format!("sinal {signal}")));
        }
    }
    Ok(())
}

pub fn requested() -> bool {
    SHUTDOWN.load(Ordering::SeqCst)
}

pub fn reset() {
    SHUTDOWN.store(false, Ordering::SeqCst);
}
