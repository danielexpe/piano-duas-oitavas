#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Wide,
    Compact,
    Minimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardStyle {
    Full,
    BaseRows,
    CompactBase,
}

pub fn detect(width: u16, height: u16) -> Breakpoint {
    if width >= 110 && height >= 30 {
        Breakpoint::Wide
    } else if width >= 90 && height >= 26 {
        Breakpoint::Compact
    } else {
        Breakpoint::Minimal
    }
}

impl Breakpoint {
    pub const fn keyboard_rows(self) -> usize {
        match self.keyboard_style() {
            KeyboardStyle::Full => 4,
            KeyboardStyle::BaseRows => 2,
            KeyboardStyle::CompactBase => 1,
        }
    }

    pub const fn keyboard_style(self) -> KeyboardStyle {
        match self {
            Self::Wide => KeyboardStyle::Full,
            Self::Compact => KeyboardStyle::BaseRows,
            Self::Minimal => KeyboardStyle::CompactBase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_documented_breakpoints() {
        assert_eq!(detect(110, 30), Breakpoint::Wide);
        assert_eq!(detect(160, 40), Breakpoint::Wide);
        assert_eq!(detect(109, 30), Breakpoint::Compact);
        assert_eq!(detect(90, 26), Breakpoint::Compact);
        assert_eq!(detect(89, 26), Breakpoint::Minimal);
        assert_eq!(detect(80, 24), Breakpoint::Minimal);
    }

    #[test]
    fn shortens_keyboard_rows_as_space_shrinks() {
        assert_eq!(Breakpoint::Wide.keyboard_rows(), 4);
        assert_eq!(Breakpoint::Compact.keyboard_rows(), 2);
        assert_eq!(Breakpoint::Minimal.keyboard_rows(), 1);
    }

    #[test]
    fn selects_keyboard_style_per_breakpoint() {
        assert_eq!(Breakpoint::Wide.keyboard_style(), KeyboardStyle::Full);
        assert_eq!(
            Breakpoint::Compact.keyboard_style(),
            KeyboardStyle::BaseRows
        );
        assert_eq!(
            Breakpoint::Minimal.keyboard_style(),
            KeyboardStyle::CompactBase
        );
    }
}
