use std::io::{self, IsTerminal};
use std::panic;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::app::App;
use crate::error::AppError;
use crate::i18n::{Key, Language};
use crate::input;
use crate::platform;
use crate::render::layout::{self, Breakpoint, KeyboardStyle};
use crate::render::piano::PianoBoard;
use crate::render::scene::{self, Keycap, Scene};
use crate::state::KeyUpMode;
use crate::theme;
use crate::util::tty::{self, TerminalGuard};

const IDLE_TIMEOUT: Duration = Duration::from_millis(33);
const ACTIVE_TIMEOUT: Duration = Duration::from_millis(16);

pub fn run(mut app: App) -> Result<(), AppError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(AppError::BackendUnavailable(
            "TUI requires an interactive terminal",
        ));
    }

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        crate::util::tty::emergency_restore();
        previous_hook(info);
    }));

    platform::signals::install()?;
    let _guard = TerminalGuard::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let started = Instant::now();
    terminal.draw(|frame| draw(frame, &app))?;
    tty::enable_keyboard_enhancement();
    if tty::keyboard_enhanced() {
        app.session_mut().set_keyup(KeyUpMode::Real);
    }

    loop {
        app.advance(started.elapsed());
        terminal.draw(|frame| draw(frame, &app))?;

        if platform::signals::requested() {
            break;
        }

        let timeout = if app.animating() {
            ACTIVE_TIMEOUT
        } else {
            IDLE_TIMEOUT
        };
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = input::map_key(&key, app.session().base_octave()) {
                    app.handle(action);
                }
            }
        }

        if app.should_quit() {
            break;
        }
    }

    app.release_all();
    app.shutdown();
    Ok(())
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let scene = Scene::from_app(app);
    let palette = theme::Palette::new(app.color_mode());
    let breakpoint = layout::detect(area.width, area.height);

    frame.render_widget(
        Block::default().style(Style::new().bg(palette.surface)),
        area,
    );

    let keyboard_rows = u16::try_from(breakpoint.keyboard_rows()).unwrap_or(1);
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(7),
        Constraint::Length(1),
        Constraint::Length(keyboard_rows + 2),
        Constraint::Length(1),
    ])
    .split(area);

    draw_header(frame, rows[0], &scene, breakpoint, &palette);
    draw_floating(frame, rows[1], &scene, &palette);
    draw_piano(frame, rows[2], &scene, &palette);
    draw_status(frame, rows[3], &scene, &palette);
    draw_keyboard(frame, rows[4], &scene, breakpoint, &palette);
    draw_footer(frame, rows[5], &scene, &palette);

    if scene.help_open {
        dim_background(frame);
        draw_help(frame, &scene, app, &palette);
    }
}

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    scene: &Scene,
    breakpoint: Breakpoint,
    palette: &theme::Palette,
) {
    let mut title = vec![Span::styled(
        Key::AppTitle.text(scene.language),
        Style::new().fg(palette.accent).add_modifier(Modifier::BOLD),
    )];
    if breakpoint == Breakpoint::Wide {
        title.push(Span::raw("  "));
        title.push(Span::styled(
            Key::AppSubtitle.text(scene.language),
            Style::new().fg(palette.text_secondary),
        ));
    }
    let hud = Line::from(vec![
        Span::styled(
            scene.hud.octave.clone(),
            Style::new().fg(palette.text_primary),
        ),
        Span::raw("   "),
        Span::styled(
            scene.hud.volume.clone(),
            Style::new().fg(palette.text_primary),
        ),
        Span::raw("   "),
        Span::styled(
            scene.hud.sound.clone(),
            Style::new().fg(if scene.hud.muted {
                palette.danger
            } else {
                palette.success
            }),
        ),
        Span::raw("   "),
        Span::styled(
            scene.hud.language.clone(),
            Style::new().fg(palette.text_secondary),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(vec![Line::from(title), hud]).style(Style::new().bg(palette.surface)),
        area,
    );
}

fn draw_floating(frame: &mut Frame, area: Rect, scene: &Scene, palette: &theme::Palette) {
    let line = match scene.floating {
        Some(floating) => {
            let style = if floating.progress < 0.65 {
                Style::new().fg(palette.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(palette.text_secondary)
            };
            Line::from(Span::styled(floating.note.name(), style))
        }
        None => Line::from(""),
    };
    frame.render_widget(
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .style(Style::new().bg(palette.surface)),
        area,
    );
}

fn draw_piano(frame: &mut Frame, area: Rect, scene: &Scene, palette: &theme::Palette) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(palette.accent_soft))
        .style(Style::new().bg(palette.surface))
        .title(Span::styled(
            scene.hud.octave.clone(),
            Style::new().fg(palette.accent),
        ));
    let inner = block.inner(area);
    let board = PianoBoard::build(scene, inner.width, inner.height);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(board.lines_with(palette)).style(Style::new().bg(palette.surface)),
        inner,
    );
}

fn draw_status(frame: &mut Frame, area: Rect, scene: &Scene, palette: &theme::Palette) {
    let mut spans = vec![
        Span::styled(
            scene.hud.sustain.clone(),
            Style::new().fg(if scene.hud.sustain_active {
                palette.success
            } else {
                palette.text_secondary
            }),
        ),
        Span::raw("   "),
        Span::styled(
            scene.hud.keyup.clone(),
            Style::new().fg(palette.text_secondary),
        ),
    ];
    if scene.hud.audio_unavailable {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            Key::HudAudioUnavailable.text(scene.language),
            Style::new().fg(palette.danger),
        ));
    }
    if scene.hud.muted {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            Key::HudMuted.text(scene.language),
            Style::new().fg(palette.danger).add_modifier(Modifier::BOLD),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::new().bg(palette.surface)),
        area,
    );
}

fn draw_keyboard(
    frame: &mut Frame,
    area: Rect,
    scene: &Scene,
    breakpoint: Breakpoint,
    palette: &theme::Palette,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(palette.accent_soft))
        .style(Style::new().bg(palette.surface))
        .title(Span::styled(
            Key::KeyboardTitle.text(scene.language),
            Style::new().fg(palette.text_secondary),
        ));
    let inner = block.inner(area);
    let lines = keyboard_lines(scene, breakpoint.keyboard_style(), palette);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(lines).style(Style::new().bg(palette.surface)),
        inner,
    );
}

fn keyboard_lines(
    scene: &Scene,
    style: KeyboardStyle,
    palette: &theme::Palette,
) -> Vec<Line<'static>> {
    match style {
        KeyboardStyle::Full => scene
            .keyboard
            .iter()
            .map(|row| keyboard_row(row, true, palette))
            .collect(),
        KeyboardStyle::BaseRows => scene
            .keyboard
            .iter()
            .skip(2)
            .map(|row| keyboard_row(row, true, palette))
            .collect(),
        KeyboardStyle::CompactBase => vec![compact_keyboard_row(scene, palette)],
    }
}

fn keyboard_row(row: &[Keycap], with_note: bool, palette: &theme::Palette) -> Line<'static> {
    let mut spans = Vec::new();
    for keycap in row {
        spans.extend(keycap_spans(keycap, with_note, palette));
    }
    Line::from(spans)
}

fn compact_keyboard_row(scene: &Scene, palette: &theme::Palette) -> Line<'static> {
    let mut spans = Vec::new();
    for label in scene::BASE_OCTAVE_KEYCAPS {
        if let Some(keycap) = scene.keycap(label) {
            spans.extend(keycap_spans(keycap, false, palette));
        }
    }
    Line::from(spans)
}

fn keycap_spans(keycap: &Keycap, with_note: bool, palette: &theme::Palette) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    spans.push(Span::raw(" "));
    let key_style = if keycap.held {
        Style::new()
            .fg(palette.text_on_light)
            .bg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else if keycap.note.is_some() {
        Style::new().fg(palette.accent)
    } else {
        Style::new()
            .fg(palette.text_secondary)
            .add_modifier(Modifier::DIM)
    };
    spans.push(Span::styled(keycap.label.to_string(), key_style));
    if with_note {
        match keycap.note {
            Some(note) => {
                let note_style = if keycap.held {
                    Style::new().fg(palette.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(palette.text_secondary)
                };
                spans.push(Span::styled(format!(":{}", note.name()), note_style));
            }
            None => spans.push(Span::styled(":--", Style::new().fg(palette.surface_raised))),
        }
    }
    spans.push(Span::raw(" "));
    spans
}

fn draw_footer(frame: &mut Frame, area: Rect, scene: &Scene, palette: &theme::Palette) {
    let line = Line::from(vec![
        Span::styled(
            Key::FooterHelp.text(scene.language),
            Style::new().fg(palette.text_secondary),
        ),
        Span::raw("   "),
        Span::styled(
            Key::FooterLanguage.text(scene.language),
            Style::new().fg(palette.text_secondary),
        ),
        Span::raw("   "),
        Span::styled(
            Key::FooterMute.text(scene.language),
            Style::new().fg(palette.text_secondary),
        ),
        Span::raw("   "),
        Span::styled(
            Key::FooterQuit.text(scene.language),
            Style::new().fg(palette.text_secondary),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::new().bg(palette.surface)),
        area,
    );
}

fn draw_help(frame: &mut Frame, scene: &Scene, app: &App, palette: &theme::Palette) {
    let lines = help_lines(scene.language, app.session().keyup(), palette);
    let area = modal_rect(frame.area(), lines.len());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(palette.accent))
        .style(Style::new().bg(palette.panel))
        .title(Span::styled(
            Key::HelpTitle.text(scene.language),
            Style::new().fg(palette.accent).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn help_lines(
    language: Language,
    keyup: KeyUpMode,
    palette: &theme::Palette,
) -> Vec<Line<'static>> {
    let normal = Style::new().fg(palette.text_primary);
    let heading = Style::new().fg(palette.accent).add_modifier(Modifier::BOLD);
    let secondary = Style::new().fg(palette.text_secondary);
    let keyup_line = match keyup {
        KeyUpMode::Real => Key::HelpKeyupReal,
        KeyUpMode::Auto => Key::HelpKeyupFallback,
    };
    vec![
        Line::from(Span::styled(Key::HelpNotesTitle.text(language), heading)),
        Line::from(Span::styled(Key::HelpNotesWhite.text(language), normal)),
        Line::from(Span::styled(Key::HelpNotesBlack.text(language), normal)),
        Line::from(Span::styled(Key::HelpSolfeges.text(language), normal)),
        Line::from(""),
        Line::from(Span::styled(Key::HelpControlsTitle.text(language), heading)),
        Line::from(Span::styled(Key::HelpSustain.text(language), normal)),
        Line::from(Span::styled(Key::HelpOctave.text(language), normal)),
        Line::from(Span::styled(Key::HelpVolume.text(language), normal)),
        Line::from(Span::styled(Key::HelpMute.text(language), normal)),
        Line::from(Span::styled(Key::HelpLanguage.text(language), normal)),
        Line::from(Span::styled(Key::HelpHelp.text(language), normal)),
        Line::from(Span::styled(Key::HelpQuit.text(language), normal)),
        Line::from(""),
        Line::from(Span::styled(keyup_line.text(language), secondary)),
        Line::from(Span::styled(
            Key::HelpNoPersistence.text(language),
            secondary,
        )),
        Line::from(""),
        Line::from(Span::styled(Key::HelpClose.text(language), secondary)),
    ]
}

fn dim_background(frame: &mut Frame) {
    for cell in &mut frame.buffer_mut().content {
        cell.modifier.insert(Modifier::DIM);
    }
}

fn modal_rect(area: Rect, content_lines: usize) -> Rect {
    let width = area.width.saturating_sub(4).min(88);
    let height = area.height.saturating_sub(2).min(
        u16::try_from(content_lines)
            .unwrap_or(u16::MAX)
            .saturating_add(2),
    );
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::i18n::Language;
    use crate::state::Session;
    use crate::theme::ColorMode;
    use ratatui::backend::TestBackend;

    fn app() -> App {
        App::new(Session::new(3, 0.32, Language::Pt, false).unwrap())
    }

    fn render(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut output = String::new();
        for y in 0..height {
            for x in 0..width {
                output.push_str(buffer.cell((x, y)).unwrap().symbol());
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn renders_all_breakpoints_without_panicking() {
        for (width, height) in [(120, 34), (100, 28), (80, 24)] {
            let text = render(&app(), width, height);
            assert_eq!(text.lines().count(), usize::from(height));
            assert!(text.contains("C3"));
        }
    }

    #[test]
    fn mono_mode_keeps_non_color_state_markers() {
        let mut app = app();
        app.set_color_mode(ColorMode::Mono);
        app.handle(crate::input::Action::NoteOn(
            crate::core::note::MidiNote::from_midi(48).unwrap(),
        ));
        app.handle(crate::input::Action::SustainPress);
        let text = render(&app, 80, 24);
        assert!(text.contains("[C3]"), "pressed marker ausente");
        assert!(text.contains("Sustain: latch"), "estado de sustain ausente");
        assert!(text.contains('='), "barra de sustain ausente");
    }

    #[test]
    fn help_wraps_on_a_narrow_screen() {
        let mut app = app();
        app.handle(crate::input::Action::ToggleLanguage);
        app.handle(crate::input::Action::ToggleHelp);
        let text = render(&app, 60, 20);
        assert_eq!(text.lines().count(), 20);
        assert!(text.contains("Help"));
    }

    #[test]
    #[ignore = "benchmark manual de frame da TUI"]
    fn frame_budget_benchmark() {
        use std::time::Instant;

        let mut active_app = app();
        active_app.handle(crate::input::Action::NoteOn(
            crate::core::note::MidiNote::from_midi(48).unwrap(),
        ));
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();

        let start = Instant::now();
        for _ in 0..200 {
            terminal.draw(|frame| draw(frame, &active_app)).unwrap();
        }
        let active = start.elapsed().as_secs_f64() * 1000.0 / 200.0;

        let mut idle = app();
        idle.handle(crate::input::Action::NoteOff(
            crate::core::note::MidiNote::from_midi(48).unwrap(),
        ));
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        let start = Instant::now();
        for _ in 0..200 {
            terminal.draw(|frame| draw(frame, &idle)).unwrap();
        }
        let idle_ms = start.elapsed().as_secs_f64() * 1000.0 / 200.0;

        println!("frame_tui_active_ms={active:.3}");
        println!("frame_tui_idle_ms={idle_ms:.3}");
        assert!(active < 16.0, "frame ativo acima de 16 ms: {active:.3}");
        assert!(idle_ms < 33.0, "frame ocioso acima de 33 ms: {idle_ms:.3}");
    }

    #[test]
    fn help_modal_fits_small_screen() {
        let mut app = app();
        app.handle(crate::input::Action::ToggleHelp);
        let text = render(&app, 80, 24);
        assert!(text.contains("Ajuda"));
        assert!(text.contains("F1 ou Esc para fechar"));
    }
}
