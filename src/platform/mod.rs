#[cfg(feature = "audio")]
pub mod alsa;
pub mod fb;
pub mod input;
pub mod signals;

pub fn prepare_audio_environment() {
    #[cfg(feature = "audio")]
    alsa::prepare_environment();
}
