use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
    LeaveAlternateScreen,
};

use crate::error::AppError;

static KEYBOARD_ENHANCED: AtomicBool = AtomicBool::new(false);

pub struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    pub fn enter() -> Result<Self, AppError> {
        enable_raw_mode()?;
        if let Err(error) = execute!(io::stdout(), EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(AppError::Terminal(error));
        }
        Ok(Self { active: true })
    }

    pub fn restore(&mut self) {
        if self.active {
            emergency_restore();
            self.active = false;
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

pub fn keyboard_enhanced() -> bool {
    KEYBOARD_ENHANCED.load(Ordering::SeqCst)
}

pub fn enable_keyboard_enhancement() {
    if matches!(
        std::env::var("TERM").as_deref(),
        Ok("dumb") | Ok("linux") | Err(_)
    ) {
        return;
    }
    if !matches!(supports_keyboard_enhancement(), Ok(true)) {
        return;
    }
    let flags = KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;
    if execute!(io::stdout(), PushKeyboardEnhancementFlags(flags)).is_ok() {
        KEYBOARD_ENHANCED.store(true, Ordering::SeqCst);
    }
}

pub fn emergency_restore() {
    if KEYBOARD_ENHANCED.swap(false, Ordering::SeqCst) {
        let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
    }
    let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
