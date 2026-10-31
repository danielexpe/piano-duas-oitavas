use std::path::Path;

use crate::app::App;
use crate::error::AppError;
use crate::render::fb::{Colors, FbRenderer};
use crate::render::layout::Breakpoint;

pub const WIDTH: u32 = 1344;
pub const HEIGHT: u32 = 756;

pub fn write_png(app: &App, path: &Path) -> Result<(), AppError> {
    let mut renderer = FbRenderer::new()?;
    let colors = Colors::new(app.color_mode());
    let pixmap = renderer.render_logical(app, Breakpoint::Wide, &colors);
    let data = pixmap
        .encode_png()
        .map_err(|error| AppError::Capture(error.to_string()))?;
    std::fs::write(path, data).map_err(|error| {
        AppError::Capture(format!(
            "não foi possível escrever {}: {error}",
            path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use crate::state::Session;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_path() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("piano-capture-{}-{id}.png", std::process::id()))
    }

    #[test]
    fn writes_a_deterministic_png() {
        let app = App::new(Session::new(3, 0.32, Language::Pt, true).unwrap());
        let first = temp_path();
        let second = temp_path();
        write_png(&app, &first).unwrap();
        write_png(&app, &second).unwrap();

        let first_bytes = std::fs::read(&first).unwrap();
        let second_bytes = std::fs::read(&second).unwrap();
        let _ = std::fs::remove_file(&first);
        let _ = std::fs::remove_file(&second);

        assert_eq!(&first_bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(first_bytes, second_bytes);
        let width = u32::from_be_bytes(first_bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(first_bytes[20..24].try_into().unwrap());
        assert_eq!((width, height), (WIDTH, HEIGHT));
    }
}
