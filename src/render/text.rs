use std::collections::HashMap;

use fontdue::{Font, Metrics};
use tiny_skia::{Color, Pixmap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    character: char,
    size: u32,
}

struct Glyph {
    metrics: Metrics,
    bitmap: Vec<u8>,
}

pub struct TextRenderer {
    font: Font,
    glyphs: HashMap<GlyphKey, Glyph>,
}

impl TextRenderer {
    pub fn new(font: Font) -> Self {
        Self {
            font,
            glyphs: HashMap::new(),
        }
    }

    pub fn measure(&mut self, text: &str, size: f32) -> f32 {
        text.chars()
            .map(|character| self.glyph(character, size).metrics.advance_width)
            .sum()
    }

    pub fn line_height(&self, size: f32) -> f32 {
        self.font
            .horizontal_line_metrics(size)
            .map(|metrics| metrics.new_line_size)
            .unwrap_or(size * 1.25)
    }

    pub fn ascent(&self, size: f32) -> f32 {
        self.font
            .horizontal_line_metrics(size)
            .map(|metrics| metrics.ascent)
            .unwrap_or(size * 0.8)
    }

    pub fn draw_text(
        &mut self,
        pixmap: &mut Pixmap,
        x: f32,
        baseline: f32,
        text: &str,
        size: f32,
        color: Color,
    ) {
        let mut pen_x = x;
        for character in text.chars() {
            let glyph = self.glyph(character, size);
            let glyph_x = pen_x + glyph.metrics.xmin as f32;
            let glyph_y = baseline - glyph.metrics.height as f32 - glyph.metrics.ymin as f32;
            blend_bitmap(
                pixmap,
                glyph_x.round() as i32,
                glyph_y.round() as i32,
                glyph.metrics.width,
                glyph.metrics.height,
                &glyph.bitmap,
                color,
            );
            pen_x += glyph.metrics.advance_width;
        }
    }

    pub fn draw_centered(
        &mut self,
        pixmap: &mut Pixmap,
        center_x: f32,
        baseline: f32,
        text: &str,
        size: f32,
        color: Color,
    ) {
        let width = self.measure(text, size);
        self.draw_text(pixmap, center_x - width / 2.0, baseline, text, size, color);
    }

    pub fn draw_right(
        &mut self,
        pixmap: &mut Pixmap,
        right: f32,
        baseline: f32,
        text: &str,
        size: f32,
        color: Color,
    ) {
        let width = self.measure(text, size);
        self.draw_text(pixmap, right - width, baseline, text, size, color);
    }

    pub fn wrap(&mut self, text: &str, size: f32, max_width: f32) -> Vec<String> {
        if text.is_empty() {
            return vec![String::new()];
        }
        let mut lines = Vec::new();
        let mut current = String::new();
        for word in text.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            if self.measure(&candidate, size) <= max_width {
                current = candidate;
                continue;
            }
            if !current.is_empty() {
                lines.push(current);
            }
            if self.measure(word, size) <= max_width {
                current = word.to_string();
                continue;
            }
            let mut chunk = String::new();
            for character in word.chars() {
                let mut candidate = chunk.clone();
                candidate.push(character);
                if chunk.is_empty() || self.measure(&candidate, size) <= max_width {
                    chunk = candidate;
                } else {
                    lines.push(chunk);
                    chunk = character.to_string();
                }
            }
            current = chunk;
        }
        if !current.is_empty() {
            lines.push(current);
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    fn glyph(&mut self, character: char, size: f32) -> &Glyph {
        let key = GlyphKey {
            character,
            size: (size * 4.0).round().max(0.0) as u32,
        };
        self.glyphs.entry(key).or_insert_with(|| {
            let (metrics, bitmap) = self.font.rasterize(character, size);
            Glyph { metrics, bitmap }
        })
    }
}

fn blend_bitmap(
    pixmap: &mut Pixmap,
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    bitmap: &[u8],
    color: Color,
) {
    let color = color.to_color_u8();
    let alpha = u16::from(color.alpha());
    if alpha == 0 || bitmap.is_empty() {
        return;
    }
    let pixmap_width = pixmap.width() as i32;
    let pixmap_height = pixmap.height() as i32;
    let stride = pixmap.width() as usize * 4;
    let data = pixmap.data_mut();

    for row in 0..height {
        let target_y = y + row as i32;
        if target_y < 0 || target_y >= pixmap_height {
            continue;
        }
        for column in 0..width {
            let coverage = bitmap[row * width + column];
            if coverage == 0 {
                continue;
            }
            let target_x = x + column as i32;
            if target_x < 0 || target_x >= pixmap_width {
                continue;
            }
            let source_alpha = (alpha * u16::from(coverage) + 127) / 255;
            if source_alpha == 0 {
                continue;
            }
            let inverse = 255 - source_alpha;
            let index = target_y as usize * stride + target_x as usize * 4;
            for (channel, source) in [color.red(), color.green(), color.blue()]
                .into_iter()
                .enumerate()
            {
                let premultiplied = (u16::from(source) * source_alpha + 127) / 255;
                let blended =
                    premultiplied + (u16::from(data[index + channel]) * inverse + 127) / 255;
                data[index + channel] = blended.min(255) as u8;
            }
            let blended_alpha = source_alpha + (u16::from(data[index + 3]) * inverse + 127) / 255;
            data[index + 3] = blended_alpha.min(255) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets;

    fn renderer() -> TextRenderer {
        TextRenderer::new(assets::ui_font().expect("embedded font"))
    }

    #[test]
    fn measures_and_draws_positive_text() {
        let mut text = renderer();
        let width = text.measure("C3", 16.0);
        assert!(width > 0.0);
        assert!(text.line_height(16.0) >= 16.0);

        let mut pixmap = Pixmap::new(64, 32).unwrap();
        pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
        text.draw_text(
            &mut pixmap,
            2.0,
            24.0,
            "C3",
            16.0,
            Color::from_rgba8(255, 255, 255, 255),
        );
        assert!(pixmap.data().chunks(4).any(|pixel| pixel[0] > 0));
    }

    #[test]
    fn clips_glyphs_outside_the_canvas() {
        let mut text = renderer();
        let mut pixmap = Pixmap::new(8, 8).unwrap();
        pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
        text.draw_text(
            &mut pixmap,
            -100.0,
            -100.0,
            "C3",
            16.0,
            Color::from_rgba8(255, 255, 255, 255),
        );
        assert!(pixmap.data().chunks(4).all(|pixel| pixel[0] == 0));
    }

    #[test]
    fn wraps_long_lines_by_word_and_character() {
        let mut text = renderer();
        let lines = text.wrap("Piano Duas Oitavas", 18.0, 60.0);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| text.measure(line, 18.0) <= 60.5));

        let lines = text.wrap("AAAAAAAAAAAAAAAAAAAAAAAA", 18.0, 30.0);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| text.measure(line, 18.0) <= 30.5));
    }

    #[test]
    fn empty_text_wraps_to_one_empty_line() {
        let mut text = renderer();
        assert_eq!(text.wrap("", 18.0, 100.0), vec![String::new()]);
    }
}
