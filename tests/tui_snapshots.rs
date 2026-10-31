use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use piano_duas_oitavas::app::App;
use piano_duas_oitavas::core::note::{MidiNote, PitchClass};
use piano_duas_oitavas::i18n::Language;
use piano_duas_oitavas::input::Action;
use piano_duas_oitavas::render::tui;
use piano_duas_oitavas::state::{KeyUpMode, Session};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn app() -> App {
    App::new(Session::new(3, 0.32, Language::Pt, false).unwrap())
}

fn note(pitch: PitchClass) -> MidiNote {
    MidiNote::from_pitch(pitch, 3).unwrap()
}

fn render(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| tui::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer.cell((x, y)).unwrap().symbol());
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }
    output
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_dir().join(name);
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::create_dir_all(snapshot_dir()).unwrap();
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "snapshot {} ausente ({error}); rode UPDATE_SNAPSHOTS=1 cargo test",
            path.display()
        )
    });
    if expected != actual {
        panic!(
            "snapshot divergente: {}\n{}",
            path.display(),
            diff(&expected, actual)
        );
    }
}

fn diff(expected: &str, actual: &str) -> String {
    let mut message = String::new();
    for (index, (left, right)) in expected.lines().zip(actual.lines()).enumerate() {
        if left != right {
            message.push_str(&format!(
                "linha {} esperada:\n{left}\nlinha {} obtida:\n{right}\n",
                index + 1,
                index + 1
            ));
            break;
        }
    }
    if expected.lines().count() != actual.lines().count() {
        message.push_str(&format!(
            "linhas esperadas: {}, obtidas: {}\n",
            expected.lines().count(),
            actual.lines().count()
        ));
    }
    message
}

#[test]
fn wide_idle_pt() {
    let text = render(&app(), 120, 36);
    assert_snapshot("wide_idle_pt.txt", &text);
}

#[test]
fn wide_idle_en() {
    let mut app = app();
    app.handle(Action::ToggleLanguage);
    let text = render(&app, 120, 36);
    assert_snapshot("wide_idle_en.txt", &text);
}

#[test]
fn wide_pressed_note() {
    let mut app = app();
    app.handle(Action::NoteOn(note(PitchClass::C)));
    let text = render(&app, 120, 36);
    assert_snapshot("wide_pressed_note.txt", &text);
}

#[test]
fn wide_release_glow() {
    let mut app = app();
    app.handle(Action::NoteOn(note(PitchClass::C)));
    app.handle(Action::NoteOff(note(PitchClass::C)));
    app.advance(Duration::from_millis(40));
    let text = render(&app, 120, 36);
    assert_snapshot("wide_release_glow.txt", &text);
}

#[test]
fn wide_sustain_latch() {
    let mut app = app();
    app.handle(Action::NoteOn(note(PitchClass::C)));
    app.handle(Action::SustainPress);
    let text = render(&app, 120, 36);
    assert_snapshot("wide_sustain_latch.txt", &text);
}

#[test]
fn wide_help_pt() {
    let mut app = app();
    app.handle(Action::ToggleHelp);
    let text = render(&app, 120, 36);
    assert_snapshot("wide_help_pt.txt", &text);
}

#[test]
fn wide_help_real_keyup() {
    let mut app = app();
    app.session_mut().set_keyup(KeyUpMode::Real);
    app.handle(Action::ToggleHelp);
    let text = render(&app, 120, 36);
    assert_snapshot("wide_help_real_keyup.txt", &text);
}

#[test]
fn wide_muted_en() {
    let mut app = app();
    app.handle(Action::ToggleLanguage);
    app.handle(Action::ToggleMute);
    let text = render(&app, 120, 36);
    assert_snapshot("wide_muted_en.txt", &text);
}

#[test]
fn compact_idle_pt() {
    let text = render(&app(), 100, 28);
    assert_snapshot("compact_idle_pt.txt", &text);
}

#[test]
fn compact_pressed_note() {
    let mut app = app();
    app.handle(Action::NoteOn(note(PitchClass::C)));
    let text = render(&app, 100, 28);
    assert_snapshot("compact_pressed_note.txt", &text);
}

#[test]
fn minimal_idle_pt() {
    let text = render(&app(), 80, 24);
    assert_snapshot("minimal_idle_pt.txt", &text);
}

#[test]
fn minimal_pressed_note() {
    let mut app = app();
    app.handle(Action::NoteOn(note(PitchClass::C)));
    let text = render(&app, 80, 24);
    assert_snapshot("minimal_pressed_note.txt", &text);
}

#[test]
fn minimal_help_pt() {
    let mut app = app();
    app.handle(Action::ToggleHelp);
    let text = render(&app, 80, 24);
    assert_snapshot("minimal_help_pt.txt", &text);
}

#[test]
fn minimal_no_animations_pressed() {
    let mut app = App::with_options(Session::new(3, 0.32, Language::Pt, false).unwrap(), true);
    app.handle(Action::NoteOn(note(PitchClass::C)));
    let text = render(&app, 80, 24);
    assert_snapshot("minimal_no_animations_pressed.txt", &text);
}

#[test]
fn resize_keeps_state_and_is_stable() {
    let mut app = app();
    app.handle(Action::NoteOn(note(PitchClass::G)));
    let wide_first = render(&app, 120, 36);
    let _minimal = render(&app, 80, 24);
    let wide_again = render(&app, 120, 36);
    assert_eq!(wide_first, wide_again);
    assert!(app.session().voices().is_held(note(PitchClass::G)));
}
