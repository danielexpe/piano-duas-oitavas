use std::process::ExitCode;

use clap::Parser;
use piano_duas_oitavas::app::App;
use piano_duas_oitavas::cli::{Backend, Cli};
use piano_duas_oitavas::error::AppError;
use piano_duas_oitavas::render::tui;
use piano_duas_oitavas::state::Session;
use piano_duas_oitavas::theme::ColorMode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("piano-duas-oitavas: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    if cli.capture_frame.is_some() {
        return Err(AppError::NotImplemented("--capture-frame (Fases 4/5)"));
    }

    let session = Session::new(cli.octave, cli.volume, cli.lang.into(), cli.no_audio)
        .ok_or(AppError::InvalidOptions("oitava ou volume fora da faixa"))?;
    let mut app = App::with_options(session, cli.no_animations);
    app.set_color_mode(ColorMode::resolve_from_env(cli.color));

    match cli.backend {
        Backend::Auto | Backend::Tui => tui::run(app),
        Backend::Fb => Err(AppError::BackendUnavailable("--backend fb chega na Fase 6")),
    }
}
