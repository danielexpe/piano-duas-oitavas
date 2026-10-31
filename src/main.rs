use std::process::ExitCode;

use clap::Parser;
use piano_duas_oitavas::app::App;
use piano_duas_oitavas::capture;
use piano_duas_oitavas::cli::{Backend, Cli};
use piano_duas_oitavas::error::AppError;
use piano_duas_oitavas::platform;
use piano_duas_oitavas::render::{fb, tui};
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
    platform::prepare_audio_environment();
    let session = Session::new(cli.octave, cli.volume, cli.lang.into(), cli.no_audio)
        .ok_or(AppError::InvalidOptions("oitava ou volume fora da faixa"))?;
    let mut app = App::with_options_and_audio(session, cli.no_animations, cli.audio_device.clone());
    app.set_color_mode(ColorMode::resolve_from_env(cli.color));

    if let Some(path) = cli.capture_frame {
        return capture::write_png(&app, &path);
    }

    match cli.backend {
        Backend::Auto | Backend::Tui => tui::run(app),
        Backend::Fb => fb::run(app, cli.fb_device),
    }
}
