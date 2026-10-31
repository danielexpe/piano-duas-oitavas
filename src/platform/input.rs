use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::time::{Duration, Instant};

use crate::core::keymap;
use crate::error::AppError;
use crate::input::Action;

const ESC: u8 = 0x1b;
const ESC_TIMEOUT: Duration = Duration::from_millis(20);
const READ_BUFFER: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseResult {
    Action(Option<Action>),
    Incomplete,
}

pub struct Reader {
    tty: File,
    pending: Vec<u8>,
}

impl Reader {
    pub fn open() -> Result<Self, AppError> {
        let tty = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_CLOEXEC)
            .open("/dev/tty")
            .map_err(AppError::Terminal)?;
        Ok(Self {
            tty,
            pending: Vec::new(),
        })
    }

    pub fn next_action(
        &mut self,
        base_octave: u8,
        timeout: Duration,
    ) -> Result<Option<Action>, AppError> {
        let deadline = Instant::now() + timeout;

        loop {
            if !self.pending.is_empty() {
                match parse(&mut self.pending, base_octave) {
                    ParseResult::Action(Some(action)) => return Ok(Some(action)),
                    ParseResult::Action(None) => continue,
                    ParseResult::Incomplete => {}
                }
            }

            let now = Instant::now();
            let remaining = deadline.saturating_duration_since(now);
            if remaining.is_zero() {
                if self.pending == [ESC] {
                    self.pending.clear();
                    return Ok(Some(Action::CloseOrQuit));
                }
                self.pending.clear();
                return Ok(None);
            }

            let wait = if self.pending == [ESC] {
                remaining.min(ESC_TIMEOUT)
            } else {
                remaining
            };
            if !self.wait_readable(wait)? {
                if self.pending == [ESC] {
                    self.pending.clear();
                    return Ok(Some(Action::CloseOrQuit));
                }
                if !self.pending.is_empty() {
                    self.pending.clear();
                }
                return Ok(None);
            }

            let mut buffer = [0u8; READ_BUFFER];
            match self.tty.read(&mut buffer) {
                Ok(0) => {
                    return Err(AppError::Terminal(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "/dev/tty encerrou",
                    )));
                }
                Ok(read) => self.pending.extend_from_slice(&buffer[..read]),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(AppError::Terminal(error)),
            }
        }
    }

    fn wait_readable(&self, timeout: Duration) -> io::Result<bool> {
        let mut descriptor = libc::pollfd {
            fd: self.tty.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let millis = libc::c_int::try_from(timeout.as_millis()).unwrap_or(libc::c_int::MAX);
        // SAFETY: `descriptor` is a valid `pollfd` and the timeout is a plain
        // integer; poll only writes into `revents`.
        let result = unsafe { libc::poll(&mut descriptor, 1, millis) };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                return Ok(false);
            }
            return Err(error);
        }
        Ok(result > 0 && descriptor.revents & libc::POLLIN != 0)
    }
}

fn parse(pending: &mut Vec<u8>, base_octave: u8) -> ParseResult {
    if pending.is_empty() {
        return ParseResult::Action(None);
    }
    if pending[0] != ESC {
        let byte = pending.remove(0);
        return ParseResult::Action(action_for_byte(byte, base_octave));
    }
    if pending.len() == 1 {
        return ParseResult::Incomplete;
    }

    match pending[1] {
        b'[' => parse_csi(pending),
        b'O' => {
            if pending.len() < 3 {
                return ParseResult::Incomplete;
            }
            let action = match pending[2] {
                b'P' => Some(Action::ToggleHelp),
                b'Q' => Some(Action::ToggleLanguage),
                b'R' => Some(Action::ToggleMute),
                _ => None,
            };
            pending.drain(0..3);
            ParseResult::Action(action)
        }
        ESC => {
            pending.remove(0);
            ParseResult::Action(Some(Action::CloseOrQuit))
        }
        _ => {
            pending.drain(0..2);
            ParseResult::Action(None)
        }
    }
}

fn parse_csi(pending: &mut Vec<u8>) -> ParseResult {
    if pending.len() < 3 {
        return ParseResult::Incomplete;
    }
    let Some(end) = pending
        .iter()
        .enumerate()
        .skip(2)
        .find(|(_, byte)| (0x40..=0x7e).contains(*byte))
        .map(|(index, _)| index)
    else {
        return ParseResult::Incomplete;
    };
    let action = match &pending[..=end] {
        b"\x1b[D" => Some(Action::OctaveDown),
        b"\x1b[C" => Some(Action::OctaveUp),
        b"\x1b[11~" => Some(Action::ToggleHelp),
        b"\x1b[12~" => Some(Action::ToggleLanguage),
        b"\x1b[13~" => Some(Action::ToggleMute),
        _ => None,
    };
    pending.drain(0..=end);
    ParseResult::Action(action)
}

fn action_for_byte(byte: u8, base_octave: u8) -> Option<Action> {
    match byte {
        0x03 => Some(Action::Quit),
        0x1b => Some(Action::CloseOrQuit),
        b' ' => Some(Action::SustainPress),
        b'+' | b'=' => Some(Action::VolumeUp),
        b'-' | b'_' => Some(Action::VolumeDown),
        0x00..=0x1f | 0x7f => None,
        _ => keymap::note_for_key(char::from(byte), base_octave).map(Action::NoteOn),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::note::{MidiNote, PitchClass};

    fn action(bytes: &[u8]) -> Option<Action> {
        let mut pending = bytes.to_vec();
        match parse(&mut pending, 3) {
            ParseResult::Action(action) => action,
            ParseResult::Incomplete => panic!("incomplete sequence: {bytes:?}"),
        }
    }

    fn note(pitch: PitchClass) -> MidiNote {
        MidiNote::from_pitch(pitch, 3).unwrap()
    }

    #[test]
    fn maps_plain_notes_and_control_bytes() {
        assert_eq!(action(b"z"), Some(Action::NoteOn(note(PitchClass::C))));
        assert_eq!(
            action(b"@"),
            Some(Action::NoteOn(
                MidiNote::from_pitch(PitchClass::CSharp, 4).unwrap()
            ))
        );
        assert_eq!(action(&[0x03]), Some(Action::Quit));
        assert_eq!(action(b" "), Some(Action::SustainPress));
        assert_eq!(action(b"+"), Some(Action::VolumeUp));
        assert_eq!(action(b"-"), Some(Action::VolumeDown));
        assert_eq!(action(b"a"), None);
    }

    #[test]
    fn maps_arrow_sequences() {
        assert_eq!(action(b"\x1b[D"), Some(Action::OctaveDown));
        assert_eq!(action(b"\x1b[C"), Some(Action::OctaveUp));
        assert_eq!(action(b"\x1b[A"), None);
    }

    #[test]
    fn maps_function_keys_in_both_encodings() {
        assert_eq!(action(b"\x1bOP"), Some(Action::ToggleHelp));
        assert_eq!(action(b"\x1bOQ"), Some(Action::ToggleLanguage));
        assert_eq!(action(b"\x1bOR"), Some(Action::ToggleMute));
        assert_eq!(action(b"\x1b[11~"), Some(Action::ToggleHelp));
        assert_eq!(action(b"\x1b[12~"), Some(Action::ToggleLanguage));
        assert_eq!(action(b"\x1b[13~"), Some(Action::ToggleMute));
    }

    #[test]
    fn ignores_alt_shifted_and_unknown_sequences() {
        assert_eq!(action(b"\x1bz"), None);
        assert_eq!(action(b"\x1b[99~"), None);
    }

    #[test]
    fn keeps_incomplete_sequences_pending() {
        let mut pending = b"\x1b".to_vec();
        assert_eq!(parse(&mut pending, 3), ParseResult::Incomplete);
        assert_eq!(pending, b"\x1b");

        let mut pending = b"\x1b[".to_vec();
        assert_eq!(parse(&mut pending, 3), ParseResult::Incomplete);
        assert_eq!(pending, b"\x1b[");
    }

    #[test]
    fn parses_multiple_events_from_a_buffer() {
        let mut pending = b"zq".to_vec();
        assert_eq!(
            parse(&mut pending, 3),
            ParseResult::Action(Some(Action::NoteOn(note(PitchClass::C))))
        );
        assert_eq!(
            parse(&mut pending, 3),
            ParseResult::Action(Some(Action::NoteOn(
                MidiNote::from_pitch(PitchClass::C, 4).unwrap()
            )))
        );
        assert!(pending.is_empty());
    }
}
