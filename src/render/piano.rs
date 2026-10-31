use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::core::note::MidiNote;
use crate::render::scene::{NoteView, Scene};
use crate::theme::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Surface,
    White,
    WhitePressed,
    WhiteGlow,
    WhiteSustain,
    Black,
    BlackPressed,
    BlackGlow,
    BlackSustain,
    Accent,
    AccentSoft,
    TextPrimary,
    TextSecondary,
    TextOnLight,
    Muted,
    Danger,
    Success,
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub ink: Ink,
}

impl Cell {
    const fn blank() -> Self {
        Self {
            ch: ' ',
            ink: Ink::Surface,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCol {
    pub note: MidiNote,
    pub start: u16,
    pub end: u16,
}

#[derive(Debug, Clone)]
pub struct PianoBoard {
    pub width: u16,
    pub height: u16,
    pub whites: Vec<KeyCol>,
    pub blacks: Vec<KeyCol>,
    cells: Vec<Vec<Cell>>,
}

impl PianoBoard {
    pub fn build(scene: &Scene, width: u16, height: u16) -> Self {
        let mut board = Self {
            width,
            height,
            whites: Vec::new(),
            blacks: Vec::new(),
            cells: vec![vec![Cell::blank(); usize::from(width)]; usize::from(height)],
        };
        if width == 0 || height == 0 {
            return board;
        }

        let white_notes: Vec<&NoteView> = scene.white_keys().collect();
        if white_notes.is_empty() {
            return board;
        }

        let count = u16::try_from(white_notes.len()).unwrap_or(1);
        let mut bounds = Vec::with_capacity(white_notes.len());
        for index in 0..count {
            let start = u32::from(width) * u32::from(index) / u32::from(count);
            let end = u32::from(width) * u32::from(index + 1) / u32::from(count);
            bounds.push((
                u16::try_from(start).unwrap_or(0),
                u16::try_from(end).unwrap_or(width),
            ));
        }

        for (view, (start, end)) in white_notes.iter().zip(bounds.iter().copied()) {
            board.whites.push(KeyCol {
                note: view.note,
                start,
                end,
            });
            let ink = if view.held {
                Ink::WhitePressed
            } else {
                Ink::White
            };
            board.fill_rect(start, 0, end, height, ' ', ink);
        }

        let black_height = black_height(height);
        let label_rows = label_rows(height, black_height);

        for (view, (start, end)) in white_notes.iter().zip(bounds.iter().copied()) {
            let width = end.saturating_sub(start) as usize;
            if let Some(row) = label_rows.0 {
                let text = white_name(view, width);
                let ink = if view.held {
                    Ink::WhitePressed
                } else {
                    Ink::White
                };
                board.put_centered(start, end, row, &text, ink);
            }
            if let Some(row) = label_rows.1 {
                let key = view.key.to_ascii_uppercase().to_string();
                let text = if view.held && width >= 3 {
                    format!("[{key}]")
                } else {
                    key
                };
                let ink = if view.held {
                    Ink::WhitePressed
                } else {
                    Ink::White
                };
                board.put_centered(start, end, row, &text, ink);
            }
            board.mark_white_state(view, start, end, height, scene.hud.sustain_active);
        }

        let mut white_index_by_midi = [None; 128];
        for (index, column) in board.whites.iter().enumerate() {
            white_index_by_midi[usize::from(column.note.midi())] = Some(index);
        }

        for view in scene.black_keys() {
            let lower = MidiNote::from_midi(view.note.midi().saturating_sub(1));
            let Some(lower) = lower else {
                continue;
            };
            let Some(index) = white_index_by_midi[usize::from(lower.midi())] else {
                continue;
            };
            let boundary = board.whites[index].end;
            let key_width = board.whites[index]
                .end
                .saturating_sub(board.whites[index].start);
            let black_width = (key_width / 2).clamp(2, 3).min(key_width.max(1));
            let start = boundary.saturating_sub(black_width / 2);
            let end = (start + black_width).min(width);
            let ink = if view.held {
                Ink::BlackPressed
            } else {
                Ink::Black
            };
            board.blacks.push(KeyCol {
                note: view.note,
                start,
                end,
            });
            board.fill_rect(start, 0, end, black_height, ' ', ink);

            let label_width = end.saturating_sub(start) as usize;
            let key = view.key.to_ascii_uppercase().to_string();
            let text = if view.held && label_width >= 3 {
                format!("[{key}]")
            } else {
                key
            };
            board.put_centered(start, end, black_height - 1, &text, ink);

            if view.held && label_width < 3 {
                board.put_centered(start, end, 0, "*", Ink::BlackPressed);
            } else if scene.hud.sustain_active && view.active {
                board.put_centered(start, end, 0, "=", Ink::BlackSustain);
            } else if view.glow > 0.0 {
                board.put_centered(start, end, 0, "~", Ink::BlackGlow);
            }
        }

        board
    }

    fn fill_rect(
        &mut self,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
        ch: char,
        ink: Ink,
    ) {
        let end_x = end_x.min(self.width);
        let end_y = end_y.min(self.height);
        for y in start_y..end_y {
            for x in start_x..end_x {
                self.put(x, y, ch, ink);
            }
        }
    }

    fn put(&mut self, x: u16, y: u16, ch: char, ink: Ink) {
        if x >= self.width || y >= self.height {
            return;
        }
        if let Some(row) = self.cells.get_mut(usize::from(y)) {
            if let Some(cell) = row.get_mut(usize::from(x)) {
                *cell = Cell { ch, ink };
            }
        }
    }

    fn put_centered(&mut self, start: u16, end: u16, row: u16, text: &str, ink: Ink) {
        let width = usize::from(end.saturating_sub(start));
        if width == 0 {
            return;
        }
        let chars: Vec<char> = text.chars().take(width).collect();
        let padding = (width - chars.len()) / 2;
        for (offset, ch) in chars.into_iter().enumerate() {
            let x = start + u16::try_from(padding + offset).unwrap_or(0);
            self.put(x, row, ch, ink);
        }
    }

    fn mark_white_state(
        &mut self,
        view: &NoteView,
        start: u16,
        end: u16,
        height: u16,
        sustain_active: bool,
    ) {
        let bottom = height.saturating_sub(1);
        if sustain_active && view.active {
            self.fill_rect(start, bottom, end, height, '=', Ink::WhiteSustain);
        } else if view.glow > 0.0 {
            self.put_centered(start, end, bottom, "~", Ink::WhiteGlow);
        }
    }

    pub fn row_text(&self, row: u16) -> String {
        self.cells
            .get(usize::from(row))
            .map(|cells| cells.iter().map(|cell| cell.ch).collect())
            .unwrap_or_default()
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines_with(&Palette::default())
    }

    pub fn lines_with(&self, palette: &Palette) -> Vec<Line<'static>> {
        self.cells
            .iter()
            .map(|row| {
                let mut spans = Vec::new();
                let mut current = None;
                let mut text = String::new();
                for cell in row {
                    if current == Some(cell.ink) {
                        text.push(cell.ch);
                    } else {
                        if let Some(ink) = current {
                            spans.push(Span::styled(
                                std::mem::take(&mut text),
                                style_for_with(ink, palette),
                            ));
                        }
                        current = Some(cell.ink);
                        text.push(cell.ch);
                    }
                }
                if let Some(ink) = current {
                    spans.push(Span::styled(text, style_for_with(ink, palette)));
                }
                Line::from(spans)
            })
            .collect()
    }
}

pub fn style_for(ink: Ink) -> Style {
    style_for_with(ink, &Palette::default())
}

pub fn style_for_with(ink: Ink, palette: &Palette) -> Style {
    let mono = palette.mode.is_mono();
    match ink {
        Ink::Surface => Style::new().fg(palette.text_secondary).bg(palette.surface),
        Ink::White => Style::new().fg(palette.text_on_light).bg(palette.ivory),
        Ink::WhitePressed => Style::new()
            .fg(palette.text_on_light)
            .bg(palette.ivory_pressed)
            .add_modifier(Modifier::BOLD)
            .add_modifier(if mono {
                Modifier::REVERSED
            } else {
                Modifier::empty()
            }),
        Ink::WhiteGlow => Style::new().fg(palette.accent).bg(palette.ivory),
        Ink::WhiteSustain => Style::new()
            .fg(palette.success)
            .bg(palette.ivory)
            .add_modifier(Modifier::BOLD)
            .add_modifier(if mono {
                Modifier::UNDERLINED
            } else {
                Modifier::empty()
            }),
        Ink::Black => Style::new().fg(palette.text_primary).bg(palette.ebony),
        Ink::BlackPressed => Style::new()
            .fg(palette.text_on_light)
            .bg(palette.ebony_pressed)
            .add_modifier(Modifier::BOLD)
            .add_modifier(if mono {
                Modifier::REVERSED
            } else {
                Modifier::empty()
            }),
        Ink::BlackGlow => Style::new().fg(palette.accent).bg(palette.ebony),
        Ink::BlackSustain => Style::new()
            .fg(palette.success)
            .bg(palette.ebony)
            .add_modifier(Modifier::BOLD)
            .add_modifier(if mono {
                Modifier::UNDERLINED
            } else {
                Modifier::empty()
            }),
        Ink::Accent => Style::new().fg(palette.accent),
        Ink::AccentSoft => Style::new().fg(palette.accent_soft),
        Ink::TextPrimary => Style::new().fg(palette.text_primary),
        Ink::TextSecondary => Style::new().fg(palette.text_secondary),
        Ink::TextOnLight => Style::new().fg(palette.text_on_light),
        Ink::Muted => Style::new()
            .fg(palette.text_secondary)
            .bg(palette.surface_raised)
            .add_modifier(if mono {
                Modifier::REVERSED
            } else {
                Modifier::empty()
            }),
        Ink::Danger => Style::new().fg(palette.danger),
        Ink::Success => Style::new().fg(palette.success),
    }
}

fn white_name(view: &NoteView, width: usize) -> String {
    let name = view.note.name();
    if !view.held {
        return name;
    }
    if width >= name.chars().count() + 2 {
        format!("[{name}]")
    } else {
        format!("*{name}")
    }
}

fn black_height(height: u16) -> u16 {
    if height <= 2 {
        height
    } else {
        ((u32::from(height) * 5 / 8) as u16).clamp(2, height - 1)
    }
}

fn label_rows(height: u16, black_height: u16) -> (Option<u16>, Option<u16>) {
    let available = height.saturating_sub(black_height);
    match available {
        0 => (None, None),
        1 => (Some(black_height), None),
        _ => {
            let padding = (available - 2) / 2;
            (
                Some(black_height + padding),
                Some(black_height + padding + 1),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::i18n::Language;
    use crate::input::Action;
    use crate::state::Session;
    use std::time::Duration;

    fn app() -> App {
        App::new(Session::new(3, 0.32, Language::Pt, false).unwrap())
    }

    fn scene(app: &App) -> Scene {
        Scene::from_app(app)
    }

    #[test]
    fn lays_out_fourteen_white_and_ten_black_keys() {
        let app = app();
        let board = PianoBoard::build(&scene(&app), 78, 14);
        assert_eq!(board.whites.len(), 14);
        assert_eq!(board.blacks.len(), 10);
        assert_eq!(board.whites[0].start, 0);
        assert_eq!(board.whites.last().unwrap().end, 78);
        for pair in board.whites.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
        assert!(board.blacks.iter().all(|key| key.start < key.end));
    }

    #[test]
    fn rows_are_fully_drawn_ascii_cells() {
        let app = app();
        let board = PianoBoard::build(&scene(&app), 80, 12);
        for row in 0..board.height {
            let text = board.row_text(row);
            assert_eq!(text.chars().count(), usize::from(board.width));
            assert!(text.is_ascii());
        }
    }

    #[test]
    fn labels_idle_and_pressed_keys() {
        let mut app = app();
        let idle = PianoBoard::build(&scene(&app), 80, 14);
        assert!(idle.rows_contain("C3"));
        app.handle(Action::NoteOn(
            crate::core::note::MidiNote::from_midi(48).unwrap(),
        ));
        let pressed = PianoBoard::build(&scene(&app), 80, 14);
        assert!(pressed.rows_contain("[C3]"));
        assert!(pressed.rows_contain("[Z]"));
    }

    #[test]
    fn black_keys_show_physical_labels() {
        let app = app();
        let board = PianoBoard::build(&scene(&app), 80, 14);
        assert!(board.rows_contain("S"));
        assert!(board.rows_contain("7"));
    }

    #[test]
    fn sustain_draws_a_static_bar_on_active_white_keys() {
        let mut app = app();
        app.handle(Action::NoteOn(
            crate::core::note::MidiNote::from_midi(48).unwrap(),
        ));
        app.handle(Action::SustainPress);
        let board = PianoBoard::build(&scene(&app), 80, 14);
        let bottom = board.row_text(board.height - 1);
        assert!(bottom.contains('='));
    }

    #[test]
    fn released_note_shows_glow_marker() {
        let mut app = app();
        let note = crate::core::note::MidiNote::from_midi(48).unwrap();
        app.handle(Action::NoteOn(note));
        app.handle(Action::NoteOff(note));
        app.advance(Duration::from_millis(40));
        let board = PianoBoard::build(&scene(&app), 80, 14);
        assert!(board.rows_contain("~"));
    }

    #[test]
    fn black_keys_overlay_ebony_cells_on_white_keys() {
        let app = app();
        let board = PianoBoard::build(&scene(&app), 80, 14);
        let black = board.blacks[0];
        assert_eq!(board.cells[0][usize::from(black.start)].ink, Ink::Black);
        assert_eq!(board.cells[0][0].ink, Ink::White);
        assert_eq!(board.cells[board.height as usize - 1][0].ink, Ink::White);
    }

    #[test]
    fn pressed_black_key_changes_ink() {
        let mut app = app();
        let note =
            crate::core::note::MidiNote::from_pitch(crate::core::note::PitchClass::CSharp, 3)
                .unwrap();
        app.handle(Action::NoteOn(note));
        let board = PianoBoard::build(&scene(&app), 80, 14);
        let black = board.blacks[0];
        assert_eq!(
            board.cells[0][usize::from(black.start)].ink,
            Ink::BlackPressed
        );
    }

    #[test]
    fn sustain_bars_use_the_dedicated_ink() {
        let mut app = app();
        app.handle(Action::NoteOn(
            crate::core::note::MidiNote::from_midi(48).unwrap(),
        ));
        app.handle(Action::SustainPress);
        let board = PianoBoard::build(&scene(&app), 80, 14);
        let white = board.whites[0];
        assert_eq!(
            board.cells[board.height as usize - 1][usize::from(white.start)].ink,
            Ink::WhiteSustain
        );
    }

    impl PianoBoard {
        fn rows_contain(&self, needle: &str) -> bool {
            (0..self.height).any(|row| self.row_text(row).contains(needle))
        }
    }
}
