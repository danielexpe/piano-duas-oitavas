use std::panic;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tiny_skia::{Color, FilterQuality, Paint, Pixmap, PixmapPaint, Rect, Transform};

use crate::app::App;
use crate::assets;
use crate::core::note::MidiNote;
use crate::error::AppError;
use crate::i18n::{Key, Language};
use crate::platform;
use crate::platform::fb::{ConsoleGuard, Framebuffer};
use crate::platform::input::Reader;
use crate::render::layout::{Breakpoint, KeyboardStyle};
use crate::render::scene::{Keycap, NoteView, Scene, BASE_OCTAVE_KEYCAPS, KEYBOARD_ROWS};
use crate::render::text::TextRenderer;
use crate::state::KeyUpMode;
use crate::theme::{self, ColorMode};

const IDLE_TIMEOUT: Duration = Duration::from_millis(33);
const ACTIVE_TIMEOUT: Duration = Duration::from_millis(16);
const CANVAS_USAGE: f32 = 0.96;

pub fn run(mut app: App, device: PathBuf) -> Result<(), AppError> {
    let mut framebuffer = Framebuffer::open(&device)?;

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        platform::fb::emergency_restore();
        previous_hook(info);
    }));

    platform::signals::install()?;
    let mut console = ConsoleGuard::enter()?;
    framebuffer.unblank();
    let mut reader = Reader::open()?;
    let mut renderer = FbRenderer::new()?;
    let colors = Colors::new(app.color_mode());
    let started = Instant::now();
    let mut last_scene: Option<Scene> = None;

    loop {
        app.advance(started.elapsed());
        let scene = Scene::from_app(&app);
        let active = effects_active(&scene);
        if last_scene.as_ref() != Some(&scene) {
            let frame = renderer.render_scene(
                &scene,
                app.session().keyup(),
                framebuffer.xres(),
                framebuffer.yres(),
                &colors,
            );
            framebuffer.present(&frame)?;
            last_scene = Some(scene);
        }

        if platform::signals::requested() {
            break;
        }

        let timeout = if active { ACTIVE_TIMEOUT } else { IDLE_TIMEOUT };
        if let Some(action) = reader.next_action(app.session().base_octave(), timeout)? {
            app.handle(action);
        }

        if app.should_quit() {
            break;
        }
    }

    app.release_all();
    console.restore();
    app.shutdown();
    Ok(())
}

pub struct FbRenderer {
    text: TextRenderer,
}

impl FbRenderer {
    pub fn new() -> Result<Self, AppError> {
        let font = assets::ui_font().ok_or(AppError::Asset("fonte embutida inválida"))?;
        Ok(Self {
            text: TextRenderer::new(font),
        })
    }

    pub fn render(&mut self, app: &App, width: u32, height: u32) -> Pixmap {
        let scene = Scene::from_app(app);
        self.render_scene(
            &scene,
            app.session().keyup(),
            width,
            height,
            &Colors::new(app.color_mode()),
        )
    }

    pub fn render_scene(
        &mut self,
        scene: &Scene,
        keyup: KeyUpMode,
        width: u32,
        height: u32,
        colors: &Colors,
    ) -> Pixmap {
        let breakpoint = choose_breakpoint(width, height);
        let logical = self.render_logical_scene(scene, keyup, breakpoint, colors);
        compose(width, height, &logical, colors)
    }

    pub fn render_logical(&mut self, app: &App, breakpoint: Breakpoint, colors: &Colors) -> Pixmap {
        let scene = Scene::from_app(app);
        self.render_logical_scene(&scene, app.session().keyup(), breakpoint, colors)
    }

    fn render_logical_scene(
        &mut self,
        scene: &Scene,
        keyup: KeyUpMode,
        breakpoint: Breakpoint,
        colors: &Colors,
    ) -> Pixmap {
        let layout = FbLayout::new(breakpoint);
        let mut pixmap = Pixmap::new(layout.width, layout.height).expect("canvas de fb válido");
        pixmap.fill(colors.surface);

        self.draw_header(&mut pixmap, &layout, scene, colors);
        self.draw_floating(&mut pixmap, &layout, scene, colors);
        self.draw_piano(&mut pixmap, &layout, scene, colors);
        self.draw_status(&mut pixmap, &layout, scene, colors);
        self.draw_keyboard(&mut pixmap, &layout, scene, colors);
        self.draw_footer(&mut pixmap, &layout, scene, colors);
        if scene.help_open {
            self.draw_help(&mut pixmap, &layout, scene, keyup, colors);
        }

        pixmap
    }

    fn draw_header(
        &mut self,
        pixmap: &mut Pixmap,
        layout: &FbLayout,
        scene: &Scene,
        colors: &Colors,
    ) {
        let text = &mut self.text;
        let size = if layout.breakpoint == Breakpoint::Minimal {
            20.0
        } else {
            28.0
        };
        let baseline = baseline_in(text, &layout.header, size);
        let title = Key::AppTitle.text(scene.language);
        text.draw_text(
            pixmap,
            layout.header.x,
            baseline,
            title,
            size,
            colors.accent,
        );
        let title_width = text.measure(title, size);

        let hud_size = if layout.breakpoint == Breakpoint::Minimal {
            13.0
        } else {
            16.0
        };
        let hud = format!(
            "{}   {}   {}   {}",
            scene.hud.octave, scene.hud.volume, scene.hud.sound, scene.hud.language
        );
        let hud_width = text.measure(&hud, hud_size);
        let right = layout.header.x + layout.header.w;

        let subtitle_size = (size * 0.62).max(12.0);
        let subtitle = Key::AppSubtitle.text(scene.language);
        let subtitle_width = text.measure(subtitle, subtitle_size);
        if title_width + 16.0 + subtitle_width + 24.0 <= layout.header.w - hud_width {
            text.draw_text(
                pixmap,
                layout.header.x + title_width + 16.0,
                baseline,
                subtitle,
                subtitle_size,
                colors.text_secondary,
            );
        }

        let hud_baseline = baseline_in(text, &layout.header, hud_size);
        text.draw_right(
            pixmap,
            right,
            hud_baseline,
            &hud,
            hud_size,
            colors.text_secondary,
        );
        fill(
            pixmap,
            RectI::new(
                layout.header.x,
                layout.header.y + layout.header.h - 1.0,
                layout.header.w,
                1.0,
            ),
            colors.accent_soft,
        );
    }

    fn draw_floating(
        &mut self,
        pixmap: &mut Pixmap,
        layout: &FbLayout,
        scene: &Scene,
        colors: &Colors,
    ) {
        let Some(floating) = scene.floating else {
            return;
        };
        let text = &mut self.text;
        let size = if layout.breakpoint == Breakpoint::Minimal {
            18.0
        } else {
            22.0
        };
        let baseline = baseline_in(text, &layout.floating, size);
        let color = if floating.progress < 0.65 {
            colors.accent
        } else {
            colors.text_secondary
        };
        let name = floating.note.name();
        text.draw_centered(
            pixmap,
            layout.floating.x + layout.floating.w / 2.0,
            baseline,
            &name,
            size,
            color,
        );
    }

    fn draw_piano(
        &mut self,
        pixmap: &mut Pixmap,
        layout: &FbLayout,
        scene: &Scene,
        colors: &Colors,
    ) {
        let text = &mut self.text;
        let panel = layout.piano;
        fill(pixmap, panel, colors.panel);
        outline(pixmap, panel, colors.accent_soft, 2.0);

        let keys = RectI::new(panel.x + 6.0, panel.y + 6.0, panel.w - 12.0, panel.h - 12.0);
        let whites: Vec<&NoteView> = scene.white_keys().collect();
        if whites.is_empty() {
            return;
        }
        let key_width = keys.w / whites.len() as f32;
        let note_size: f32 = if layout.breakpoint == Breakpoint::Minimal {
            13.0
        } else {
            16.0
        };
        let key_size = (note_size * 0.85).max(10.0);

        for (index, view) in whites.iter().enumerate() {
            let x = keys.x + index as f32 * key_width;
            let rect = RectI::new(x, keys.y, (key_width - 2.0).max(1.0), keys.h);
            let fill_color = if view.held {
                colors.ivory_pressed
            } else {
                colors.ivory
            };
            fill(pixmap, rect, fill_color);
            fill(
                pixmap,
                RectI::new(rect.right() - 1.0, rect.y, 1.0, rect.h),
                colors.ivory_edge,
            );
            fill(
                pixmap,
                RectI::new(rect.x, rect.bottom() - 1.0, rect.w, 1.0),
                colors.ivory_edge,
            );

            let name = view.note.name();
            let name_baseline = rect.y + rect.h - 54.0;
            text.draw_centered(
                pixmap,
                rect.x + rect.w / 2.0,
                name_baseline,
                &name,
                note_size,
                colors.text_on_light,
            );
            let key = view.key.to_ascii_uppercase().to_string();
            let key_baseline = rect.y + rect.h - 24.0;
            text.draw_centered(
                pixmap,
                rect.x + rect.w / 2.0,
                key_baseline,
                &key,
                key_size,
                colors.text_on_light,
            );

            if scene.hud.sustain_active && view.active {
                fill(
                    pixmap,
                    RectI::new(rect.x + 4.0, rect.bottom() - 9.0, rect.w - 8.0, 5.0),
                    colors.marker_on_light,
                );
            } else if view.glow > 0.0 {
                fill(
                    pixmap,
                    RectI::new(rect.x + 4.0, rect.bottom() - 7.0, rect.w - 8.0, 3.0),
                    colors.accent,
                );
            }
        }

        let mut white_index_by_midi = [None; 128];
        for (index, view) in whites.iter().enumerate() {
            white_index_by_midi[usize::from(view.note.midi())] = Some(index);
        }

        let black_height = keys.h * 0.62;
        for view in scene.black_keys() {
            let lower = MidiNote::from_midi(view.note.midi().saturating_sub(1));
            let Some(lower) = lower else {
                continue;
            };
            let Some(index) = white_index_by_midi[usize::from(lower.midi())] else {
                continue;
            };
            let boundary = keys.x + (index + 1) as f32 * key_width;
            let black_width = (key_width * 0.62).max(16.0);
            let x = boundary - black_width / 2.0;
            let rect = RectI::new(x, keys.y, black_width, black_height);
            fill(
                pixmap,
                RectI::new(x + 3.0, keys.y + 4.0, black_width, black_height),
                colors.ebony_edge,
            );
            let fill_color = if view.held {
                colors.ebony_pressed
            } else {
                colors.ebony
            };
            fill(pixmap, rect, fill_color);
            if view.glow > 0.0 {
                outline(pixmap, rect, colors.accent, 1.5);
            }

            let badge_width = (black_width * 0.72).clamp(14.0, 38.0);
            let badge_height = 21.0;
            let badge = RectI::new(
                x + (black_width - badge_width) / 2.0,
                keys.y + black_height - badge_height - 8.0,
                badge_width,
                badge_height,
            );
            fill(
                pixmap,
                badge,
                if view.held {
                    colors.ivory_pressed
                } else {
                    colors.ivory
                },
            );
            let key = view.key.to_ascii_uppercase().to_string();
            let badge_baseline = baseline_in(text, &badge, 12.0);
            text.draw_centered(
                pixmap,
                badge.x + badge.w / 2.0,
                badge_baseline,
                &key,
                12.0,
                colors.text_on_light,
            );

            if scene.hud.sustain_active && view.active {
                fill(
                    pixmap,
                    RectI::new(rect.x + 3.0, rect.bottom() - 6.0, rect.w - 6.0, 4.0),
                    colors.marker_on_dark,
                );
            }
        }
    }

    fn draw_status(
        &mut self,
        pixmap: &mut Pixmap,
        layout: &FbLayout,
        scene: &Scene,
        colors: &Colors,
    ) {
        let text = &mut self.text;
        let size = 14.0;
        let baseline = baseline_in(text, &layout.status, size);
        let mut spans: Vec<(String, Color)> = vec![
            (
                scene.hud.sustain.clone(),
                if scene.hud.sustain_active {
                    colors.success
                } else {
                    colors.text_secondary
                },
            ),
            (scene.hud.keyup.clone(), colors.text_secondary),
        ];
        if scene.hud.audio_unavailable {
            spans.push((
                Key::HudAudioUnavailable.text(scene.language).to_string(),
                colors.danger,
            ));
        }
        if scene.hud.muted {
            spans.push((
                Key::HudMuted.text(scene.language).to_string(),
                colors.danger,
            ));
        }
        let mut x = layout.status.x;
        for (label, color) in spans {
            text.draw_text(pixmap, x, baseline, &label, size, color);
            x += text.measure(&label, size) + 18.0;
        }
    }

    fn draw_keyboard(
        &mut self,
        pixmap: &mut Pixmap,
        layout: &FbLayout,
        scene: &Scene,
        colors: &Colors,
    ) {
        let text = &mut self.text;
        let panel = layout.keyboard;
        fill(pixmap, panel, colors.panel);
        outline(pixmap, panel, colors.accent_soft, 1.0);

        let title_size = 11.0;
        let title_baseline = panel.y + 2.0 + text.ascent(title_size);
        text.draw_text(
            pixmap,
            panel.x + 8.0,
            title_baseline,
            Key::KeyboardTitle.text(scene.language),
            title_size,
            colors.text_secondary,
        );

        let inner = RectI::new(
            panel.x + 8.0,
            panel.y + title_size + 5.0,
            panel.w - 16.0,
            panel.h - title_size - 11.0,
        );
        let rows = keyboard_rows(layout.breakpoint.keyboard_style());
        if rows.is_empty() {
            return;
        }
        let gap = 6.0;
        let max_columns = rows.iter().map(Vec::len).max().unwrap_or(1) as f32;
        let key_width = (64.0_f32)
            .min((inner.w - (max_columns - 1.0) * gap) / max_columns)
            .max(16.0);
        let row_count = rows.len() as f32;
        let key_height = (22.0_f32)
            .min((inner.h - (row_count - 1.0) * gap) / row_count)
            .max(12.0);

        for (row_index, row) in rows.iter().enumerate() {
            let row_width = row.len() as f32 * key_width + (row.len() as f32 - 1.0) * gap;
            let start_x = inner.x + (inner.w - row_width) / 2.0;
            let y = inner.y + row_index as f32 * (key_height + gap);
            for (column, label) in row.iter().enumerate() {
                let x = start_x + column as f32 * (key_width + gap);
                let rect = RectI::new(x, y, key_width, key_height);
                let keycap = scene.keycap(*label);
                self.draw_keycap(pixmap, rect, *label, keycap, colors);
            }
        }
    }

    fn draw_keycap(
        &mut self,
        pixmap: &mut Pixmap,
        rect: RectI,
        label: char,
        keycap: Option<&Keycap>,
        colors: &Colors,
    ) {
        let text = &mut self.text;
        let held = keycap.is_some_and(|keycap| keycap.held);
        let note = keycap.and_then(|keycap| keycap.note);

        let (background, border, foreground) = if held {
            (colors.accent, colors.accent, colors.text_on_light)
        } else if note.is_some() {
            (colors.raised, colors.accent_soft, colors.accent)
        } else {
            (colors.panel, colors.raised, colors.text_secondary)
        };
        fill(pixmap, rect, background);
        outline(pixmap, rect, border, 1.0);

        let label_size = if rect.w < 42.0 { 10.0 } else { 12.0 };
        let label_baseline = baseline_in(text, &rect, label_size);
        let label = label.to_string();
        if let Some(note) = note {
            let name = note.name();
            let note_size = 8.5;
            let note_color = if held {
                colors.text_on_light
            } else {
                colors.text_secondary
            };
            let label_width = text.measure(&label, label_size);
            let note_width = text.measure(&name, note_size);
            let total = label_width + 3.0 + note_width;
            if total <= rect.w - 4.0 {
                let x = rect.x + (rect.w - total) / 2.0;
                text.draw_text(pixmap, x, label_baseline, &label, label_size, foreground);
                text.draw_text(
                    pixmap,
                    x + label_width + 3.0,
                    label_baseline,
                    &name,
                    note_size,
                    note_color,
                );
            } else {
                text.draw_centered(
                    pixmap,
                    rect.x + rect.w / 2.0,
                    label_baseline,
                    &label,
                    label_size,
                    foreground,
                );
            }
        } else {
            text.draw_centered(
                pixmap,
                rect.x + rect.w / 2.0,
                label_baseline,
                &label,
                label_size,
                foreground,
            );
        }
    }

    fn draw_footer(
        &mut self,
        pixmap: &mut Pixmap,
        layout: &FbLayout,
        scene: &Scene,
        colors: &Colors,
    ) {
        let text = &mut self.text;
        let size = 13.0;
        let labels = [
            Key::FooterHelp.text(scene.language),
            Key::FooterLanguage.text(scene.language),
            Key::FooterMute.text(scene.language),
            Key::FooterQuit.text(scene.language),
        ];
        let gap = 26.0;
        let total: f32 = labels
            .iter()
            .map(|label| text.measure(label, size))
            .sum::<f32>()
            + gap * (labels.len() as f32 - 1.0);
        let mut x = layout.footer.x + (layout.footer.w - total) / 2.0;
        let baseline = baseline_in(text, &layout.footer, size);
        for label in labels {
            text.draw_text(pixmap, x, baseline, label, size, colors.text_secondary);
            x += text.measure(label, size) + gap;
        }
    }

    fn draw_help(
        &mut self,
        pixmap: &mut Pixmap,
        layout: &FbLayout,
        scene: &Scene,
        keyup: KeyUpMode,
        colors: &Colors,
    ) {
        let text = &mut self.text;
        fill(
            pixmap,
            RectI::new(0.0, 0.0, layout.width as f32, layout.height as f32),
            Color::from_rgba8(0, 0, 0, 170),
        );

        let size = if layout.breakpoint == Breakpoint::Wide {
            18.0
        } else {
            16.0
        };
        let panel = RectI::new(
            (layout.width as f32 * 0.09).max(18.0),
            (layout.height as f32 * 0.09).max(18.0),
            layout.width as f32 * 0.82,
            layout.height as f32 * 0.82,
        );
        fill(pixmap, panel, colors.panel);
        outline(pixmap, panel, colors.accent, 2.0);

        let title_size = 26.0;
        text.draw_text(
            pixmap,
            panel.x + 26.0,
            panel.y + 18.0 + text.ascent(title_size),
            Key::HelpTitle.text(scene.language),
            title_size,
            colors.accent,
        );

        let content = RectI::new(
            panel.x + 26.0,
            panel.y + 64.0,
            panel.w - 52.0,
            panel.h - 82.0,
        );
        let line_height = text.line_height(size);
        let mut y = content.y;
        for (entry, kind) in help_entries(scene.language, keyup) {
            if entry.is_empty() {
                y += line_height * 0.5;
                continue;
            }
            for line in text.wrap(&entry, size, content.w) {
                if y + line_height > content.bottom() {
                    return;
                }
                let color = match kind {
                    HelpKind::Heading => colors.accent,
                    HelpKind::Secondary => colors.text_secondary,
                    HelpKind::Normal => colors.text_primary,
                };
                text.draw_text(pixmap, content.x, y + text.ascent(size), &line, size, color);
                y += line_height;
            }
        }
    }
}

fn keyboard_rows(style: KeyboardStyle) -> Vec<Vec<char>> {
    match style {
        KeyboardStyle::Full => KEYBOARD_ROWS
            .iter()
            .map(|row| row.chars().collect())
            .collect(),
        KeyboardStyle::BaseRows => KEYBOARD_ROWS
            .iter()
            .skip(2)
            .map(|row| row.chars().collect())
            .collect(),
        KeyboardStyle::CompactBase => vec![BASE_OCTAVE_KEYCAPS.to_vec()],
    }
}

#[derive(Debug, Clone, Copy)]
enum HelpKind {
    Heading,
    Normal,
    Secondary,
}

fn help_entries(language: Language, keyup: KeyUpMode) -> Vec<(String, HelpKind)> {
    let keyup_key = match keyup {
        KeyUpMode::Real => Key::HelpKeyupReal,
        KeyUpMode::Auto => Key::HelpKeyupFallback,
    };
    vec![
        (Key::HelpNotesTitle.text(language).into(), HelpKind::Heading),
        (Key::HelpNotesWhite.text(language).into(), HelpKind::Normal),
        (Key::HelpNotesBlack.text(language).into(), HelpKind::Normal),
        (Key::HelpSolfeges.text(language).into(), HelpKind::Normal),
        (String::new(), HelpKind::Normal),
        (
            Key::HelpControlsTitle.text(language).into(),
            HelpKind::Heading,
        ),
        (Key::HelpSustain.text(language).into(), HelpKind::Normal),
        (Key::HelpOctave.text(language).into(), HelpKind::Normal),
        (Key::HelpVolume.text(language).into(), HelpKind::Normal),
        (Key::HelpMute.text(language).into(), HelpKind::Normal),
        (Key::HelpLanguage.text(language).into(), HelpKind::Normal),
        (Key::HelpHelp.text(language).into(), HelpKind::Normal),
        (Key::HelpQuit.text(language).into(), HelpKind::Normal),
        (String::new(), HelpKind::Normal),
        (keyup_key.text(language).into(), HelpKind::Secondary),
        (
            Key::HelpNoPersistence.text(language).into(),
            HelpKind::Secondary,
        ),
        (String::new(), HelpKind::Normal),
        (Key::HelpClose.text(language).into(), HelpKind::Secondary),
    ]
}

fn baseline_in(text: &TextRenderer, rect: &RectI, size: f32) -> f32 {
    rect.y + (rect.h - text.line_height(size)) / 2.0 + text.ascent(size)
}

#[derive(Debug, Clone, Copy)]
pub struct RectI {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectI {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    fn right(self) -> f32 {
        self.x + self.w
    }

    fn bottom(self) -> f32 {
        self.y + self.h
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FbLayout {
    pub width: u32,
    pub height: u32,
    pub breakpoint: Breakpoint,
    pub header: RectI,
    pub floating: RectI,
    pub piano: RectI,
    pub status: RectI,
    pub keyboard: RectI,
    pub footer: RectI,
}

impl FbLayout {
    pub fn new(breakpoint: Breakpoint) -> Self {
        let (
            width,
            height,
            margin,
            top,
            bottom,
            header,
            floating,
            status,
            keyboard,
            footer,
            gap,
        ): (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32) = match breakpoint {
                Breakpoint::Wide => (
                    1344.0, 756.0, 48.0, 24.0, 12.0, 68.0, 46.0, 26.0, 128.0, 20.0, 4.0,
                ),
                Breakpoint::Compact => (
                    1120.0, 700.0, 36.0, 20.0, 14.0, 58.0, 38.0, 24.0, 92.0, 18.0, 4.0,
                ),
                Breakpoint::Minimal => (
                    960.0, 640.0, 28.0, 16.0, 12.0, 46.0, 30.0, 22.0, 44.0, 16.0, 3.0,
                ),
            };
        let fixed = top + bottom + header + floating + status + keyboard + footer + gap * 5.0;
        let piano_height: f32 = (height - fixed).max(80.0);
        let x = margin;
        let w = width - margin * 2.0;
        let header_rect = RectI::new(x, top, w, header);
        let floating_rect = RectI::new(x, header_rect.bottom() + gap, w, floating);
        let piano_rect = RectI::new(x, floating_rect.bottom() + gap, w, piano_height);
        let status_rect = RectI::new(x, piano_rect.bottom() + gap, w, status);
        let keyboard_rect = RectI::new(x, status_rect.bottom() + gap, w, keyboard);
        let footer_rect = RectI::new(x, keyboard_rect.bottom() + gap, w, footer);
        Self {
            width: width as u32,
            height: height as u32,
            breakpoint,
            header: header_rect,
            floating: floating_rect,
            piano: piano_rect,
            status: status_rect,
            keyboard: keyboard_rect,
            footer: footer_rect,
        }
    }
}

fn effects_active(scene: &Scene) -> bool {
    scene.floating.is_some()
        || scene
            .notes
            .iter()
            .any(|note| note.press > 0.0 || note.glow > 0.0)
}

pub fn choose_breakpoint(width: u32, height: u32) -> Breakpoint {
    if width >= 1280 && height >= 720 {
        Breakpoint::Wide
    } else if width >= 1024 && height >= 600 {
        Breakpoint::Compact
    } else {
        Breakpoint::Minimal
    }
}

fn compose(width: u32, height: u32, logical: &Pixmap, colors: &Colors) -> Pixmap {
    let mut output = Pixmap::new(width, height).expect("framebuffer de tamanho válido");
    output.fill(colors.surface);
    let scale = (width as f32 * CANVAS_USAGE / logical.width() as f32)
        .min(height as f32 * CANVAS_USAGE / logical.height() as f32)
        .max(0.01);
    let draw_width = logical.width() as f32 * scale;
    let draw_height = logical.height() as f32 * scale;
    let x = (width as f32 - draw_width) / 2.0;
    let y = (height as f32 - draw_height) / 2.0;
    let transform = Transform::from_translate(x, y).pre_scale(scale, scale);
    let paint = PixmapPaint {
        quality: FilterQuality::Bilinear,
        ..PixmapPaint::default()
    };
    output.draw_pixmap(0, 0, logical.as_ref(), &paint, transform, None);
    output
}

fn fill(pixmap: &mut Pixmap, rect: RectI, color: Color) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let Some(rect) = Rect::from_xywh(rect.x, rect.y, rect.w, rect.h) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = false;
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

fn outline(pixmap: &mut Pixmap, rect: RectI, color: Color, thickness: f32) {
    fill(pixmap, RectI::new(rect.x, rect.y, rect.w, thickness), color);
    fill(
        pixmap,
        RectI::new(rect.x, rect.bottom() - thickness, rect.w, thickness),
        color,
    );
    fill(pixmap, RectI::new(rect.x, rect.y, thickness, rect.h), color);
    fill(
        pixmap,
        RectI::new(rect.right() - thickness, rect.y, thickness, rect.h),
        color,
    );
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub surface: Color,
    pub panel: Color,
    pub raised: Color,
    pub ivory: Color,
    pub ivory_pressed: Color,
    pub ivory_edge: Color,
    pub ebony: Color,
    pub ebony_pressed: Color,
    pub ebony_edge: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_on_light: Color,
    pub danger: Color,
    pub success: Color,
    pub marker_on_light: Color,
    pub marker_on_dark: Color,
}

impl Colors {
    pub fn new(mode: ColorMode) -> Self {
        if mode.is_mono() {
            Self::mono()
        } else {
            Self::truecolor()
        }
    }

    fn truecolor() -> Self {
        Self {
            surface: themed(theme::SURFACE),
            panel: themed(theme::PANEL),
            raised: themed(theme::SURFACE_RAISED),
            ivory: themed(theme::IVORY),
            ivory_pressed: themed(theme::IVORY_PRESSED),
            ivory_edge: themed(theme::IVORY_EDGE),
            ebony: themed(theme::EBONY),
            ebony_pressed: themed(theme::EBONY_PRESSED),
            ebony_edge: themed(theme::EBONY_EDGE),
            accent: themed(theme::ACCENT),
            accent_soft: themed(theme::ACCENT_SOFT),
            text_primary: themed(theme::TEXT_PRIMARY),
            text_secondary: themed(theme::TEXT_SECONDARY),
            text_on_light: themed(theme::TEXT_ON_LIGHT),
            danger: themed(theme::DANGER),
            success: themed(theme::SUCCESS),
            marker_on_light: themed(theme::SUCCESS),
            marker_on_dark: themed(theme::SUCCESS),
        }
    }

    fn mono() -> Self {
        Self {
            surface: rgb(0x00, 0x00, 0x00),
            panel: rgb(0x14, 0x14, 0x14),
            raised: rgb(0x26, 0x26, 0x26),
            ivory: rgb(0xe8, 0xe8, 0xe8),
            ivory_pressed: rgb(0x9a, 0x9a, 0x9a),
            ivory_edge: rgb(0x70, 0x70, 0x70),
            ebony: rgb(0x08, 0x08, 0x08),
            ebony_pressed: rgb(0x66, 0x66, 0x66),
            ebony_edge: rgb(0x30, 0x30, 0x30),
            accent: rgb(0xf0, 0xf0, 0xf0),
            accent_soft: rgb(0x80, 0x80, 0x80),
            text_primary: rgb(0xff, 0xff, 0xff),
            text_secondary: rgb(0xb0, 0xb0, 0xb0),
            text_on_light: rgb(0x00, 0x00, 0x00),
            danger: rgb(0xff, 0xff, 0xff),
            success: rgb(0xff, 0xff, 0xff),
            marker_on_light: rgb(0x00, 0x00, 0x00),
            marker_on_dark: rgb(0xff, 0xff, 0xff),
        }
    }
}

fn themed(color: ratatui::style::Color) -> Color {
    match color {
        ratatui::style::Color::Rgb(red, green, blue) => rgb(red, green, blue),
        _ => rgb(0, 0, 0),
    }
}

fn rgb(red: u8, green: u8, blue: u8) -> Color {
    Color::from_rgba8(red, green, blue, 255)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use crate::state::Session;

    fn app() -> App {
        App::new(Session::new(3, 0.32, Language::Pt, false).unwrap())
    }

    #[test]
    fn chooses_documented_device_breakpoints() {
        assert_eq!(choose_breakpoint(1920, 1080), Breakpoint::Wide);
        assert_eq!(choose_breakpoint(1280, 720), Breakpoint::Wide);
        assert_eq!(choose_breakpoint(1279, 720), Breakpoint::Compact);
        assert_eq!(choose_breakpoint(1024, 600), Breakpoint::Compact);
        assert_eq!(choose_breakpoint(1023, 600), Breakpoint::Minimal);
    }

    #[test]
    fn logical_layouts_fit_inside_the_canvas() {
        for breakpoint in [Breakpoint::Wide, Breakpoint::Compact, Breakpoint::Minimal] {
            let layout = FbLayout::new(breakpoint);
            let canvas = RectI::new(0.0, 0.0, layout.width as f32, layout.height as f32);
            for rect in [
                layout.header,
                layout.floating,
                layout.piano,
                layout.status,
                layout.keyboard,
                layout.footer,
            ] {
                assert!(rect.h > 0.0, "{breakpoint:?}: altura não positiva");
                assert!(rect.x >= canvas.x, "{breakpoint:?}: x fora");
                assert!(
                    rect.right() <= canvas.right() + 0.5,
                    "{breakpoint:?}: direita fora"
                );
                assert!(
                    rect.bottom() <= canvas.bottom() + 0.5,
                    "{breakpoint:?}: base fora"
                );
            }
            assert!(layout.piano.h >= 80.0);
        }
    }

    #[test]
    fn renders_a_non_empty_deterministic_frame() {
        let mut renderer = FbRenderer::new().unwrap();
        let app = app();
        let colors = Colors::new(ColorMode::Truecolor);
        let first = renderer.render_logical(&app, Breakpoint::Wide, &colors);
        let second = renderer.render_logical(&app, Breakpoint::Wide, &colors);
        assert_eq!(first.width(), 1344);
        assert_eq!(first.height(), 756);
        assert_eq!(first.data(), second.data());
        let background = colors.surface.to_color_u8();
        let has_content = first.data().chunks(4).any(|pixel| {
            pixel[0] != background.red()
                || pixel[1] != background.green()
                || pixel[2] != background.blue()
        });
        assert!(has_content, "frame sem conteúdo");
    }

    #[test]
    fn composes_with_letterbox_and_centered_content() {
        let colors = Colors::new(ColorMode::Truecolor);
        let mut logical = Pixmap::new(100, 50).unwrap();
        logical.fill(rgb(255, 0, 0));
        let output = compose(200, 200, &logical, &colors);
        let corner = output.pixel(0, 0).unwrap();
        assert_eq!(corner.red(), 0x12);
        let center = output.pixel(100, 100).unwrap();
        assert!(center.red() > 100);
    }

    #[test]
    fn renders_help_and_mono_frames() {
        let mut renderer = FbRenderer::new().unwrap();
        let mut app = app();
        app.set_color_mode(ColorMode::Mono);
        app.handle(crate::input::Action::ToggleHelp);
        let colors = Colors::new(app.color_mode());
        let frame = renderer.render_logical(&app, Breakpoint::Compact, &colors);
        assert_eq!(frame.width(), 1120);
        assert_eq!(frame.height(), 700);
    }

    #[test]
    #[ignore = "benchmark manual de frame do fb"]
    fn frame_budget_benchmark() {
        use std::time::Instant;

        let mut renderer = FbRenderer::new().unwrap();
        let mut app = app();
        app.handle(crate::input::Action::NoteOn(
            crate::core::note::MidiNote::from_midi(48).unwrap(),
        ));
        for _ in 0..5 {
            let _ = renderer.render(&app, 1920, 1080);
        }
        let start = Instant::now();
        for _ in 0..60 {
            let _ = renderer.render(&app, 1920, 1080);
        }
        let active = start.elapsed().as_secs_f64() * 1000.0 / 60.0;
        println!("frame_fb_active_ms={active:.3}");
        assert!(active < 33.0, "frame fb acima de 33 ms: {active:.3}");
    }

    #[test]
    fn renders_every_breakpoint() {
        let mut renderer = FbRenderer::new().unwrap();
        let app = app();
        let colors = Colors::new(ColorMode::Truecolor);
        for breakpoint in [Breakpoint::Wide, Breakpoint::Compact, Breakpoint::Minimal] {
            let frame = renderer.render_logical(&app, breakpoint, &colors);
            assert!(frame.width() > 0);
            assert!(frame.height() > 0);
        }
    }
}
