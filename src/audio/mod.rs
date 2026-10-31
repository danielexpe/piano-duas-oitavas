pub mod synth;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

const STATE_STARTING: u8 = 0;
#[cfg(feature = "audio")]
const STATE_AVAILABLE: u8 = 1;
const STATE_UNAVAILABLE: u8 = 2;

pub struct Audio {
    volume: f32,
    muted: bool,
    state: Arc<AtomicU8>,
    #[cfg(feature = "audio")]
    device: String,
    #[cfg(feature = "audio")]
    engine: Option<Engine>,
}

impl std::fmt::Debug for Audio {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Audio")
            .field("volume", &self.volume)
            .field("muted", &self.muted)
            .field("state", &self.state.load(Ordering::Relaxed))
            .finish()
    }
}

impl Audio {
    pub fn new(volume: f32, muted: bool) -> Self {
        Self::new_with_device(volume, muted, "default")
    }

    pub fn new_with_device(volume: f32, muted: bool, device: impl Into<String>) -> Self {
        #[cfg(not(feature = "audio"))]
        let _ = device;
        Self {
            volume: volume.clamp(0.0, 1.0),
            muted,
            state: Arc::new(AtomicU8::new(STATE_STARTING)),
            #[cfg(feature = "audio")]
            device: device.into(),
            #[cfg(feature = "audio")]
            engine: None,
        }
    }

    pub fn unavailable(&self) -> bool {
        cfg!(not(feature = "audio")) || self.state.load(Ordering::Relaxed) == STATE_UNAVAILABLE
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        #[cfg(feature = "audio")]
        {
            self.ensure_started();
            if let Some(engine) = &self.engine {
                engine.send(Message::NoteOn { note, velocity });
            }
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = (note, velocity);
        }
    }

    pub fn note_off(&mut self, note: u8) {
        #[cfg(feature = "audio")]
        if let Some(engine) = &self.engine {
            engine.send(Message::NoteOff { note });
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = note;
        }
    }

    pub fn release_all(&mut self) {
        #[cfg(feature = "audio")]
        if let Some(engine) = &self.engine {
            engine.send(Message::AllNotesOff);
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        #[cfg(feature = "audio")]
        if let Some(engine) = &self.engine {
            engine.send(Message::SetVolume(self.volume));
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        #[cfg(feature = "audio")]
        if let Some(engine) = &self.engine {
            engine.send(Message::SetMuted(muted));
        }
    }

    pub fn shutdown(&mut self) {
        #[cfg(feature = "audio")]
        if let Some(mut engine) = self.engine.take() {
            engine.shutdown();
        }
    }

    #[cfg(feature = "audio")]
    fn ensure_started(&mut self) {
        if self.engine.is_some() || self.muted || cfg!(test) {
            return;
        }

        let (sender, receiver) = std::sync::mpsc::sync_channel(64);
        let state = Arc::clone(&self.state);
        let volume = self.volume;
        let muted = self.muted;
        let device = self.device.clone();
        match std::thread::Builder::new()
            .name("piano-audio".to_string())
            .spawn(move || run(receiver, state, volume, muted, device))
        {
            Ok(handle) => {
                self.engine = Some(Engine {
                    sender,
                    handle: Some(handle),
                });
            }
            Err(_) => self.state.store(STATE_UNAVAILABLE, Ordering::Relaxed),
        }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(feature = "audio")]
#[derive(Debug, Clone, Copy)]
enum Message {
    NoteOn { note: u8, velocity: f32 },
    NoteOff { note: u8 },
    AllNotesOff,
    SetVolume(f32),
    SetMuted(bool),
    Shutdown,
}

#[cfg(feature = "audio")]
struct Engine {
    sender: std::sync::mpsc::SyncSender<Message>,
    handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(feature = "audio")]
impl Engine {
    fn send(&self, message: Message) {
        let _ = self.sender.try_send(message);
    }

    fn shutdown(&mut self) {
        let _ = self.sender.send(Message::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(feature = "audio")]
fn run(
    receiver: std::sync::mpsc::Receiver<Message>,
    state: Arc<AtomicU8>,
    volume: f32,
    muted: bool,
    device_name: String,
) {
    use synth::Synth;

    const BUFFER_FRAMES: usize = 1024;
    const SHUTDOWN_BUFFERS: usize = 6;

    let mut device = match crate::platform::alsa::open_device(&device_name) {
        Ok(device) => device,
        Err(error) => {
            crate::platform::alsa::debug(&format!("áudio indisponível: {error}"));
            state.store(STATE_UNAVAILABLE, Ordering::Relaxed);
            return;
        }
    };
    state.store(STATE_AVAILABLE, Ordering::Relaxed);

    let mut synth = Synth::new(volume, muted);
    let mut buffer = vec![0.0f32; BUFFER_FRAMES];

    loop {
        let mut shutdown = false;
        while let Ok(message) = receiver.try_recv() {
            match message {
                Message::NoteOn { note, velocity } => synth.note_on(note, velocity),
                Message::NoteOff { note } => synth.note_off(note),
                Message::AllNotesOff => synth.all_notes_off(),
                Message::SetVolume(volume) => synth.set_volume(volume),
                Message::SetMuted(muted) => synth.set_muted(muted),
                Message::Shutdown => {
                    shutdown = true;
                    break;
                }
            }
        }

        if shutdown {
            synth.shutdown();
            for _ in 0..SHUTDOWN_BUFFERS {
                synth.render(&mut buffer);
                if device.write(&buffer).is_err() {
                    break;
                }
            }
            break;
        }

        synth.render(&mut buffer);
        if device.write(&buffer).is_err() {
            crate::platform::alsa::debug("falha de escrita no PCM; áudio desativado");
            state.store(STATE_UNAVAILABLE, Ordering::Relaxed);
            break;
        }
    }
}

#[cfg(all(test, feature = "audio"))]
mod tests {
    use super::*;

    #[test]
    fn starts_optimistic_and_reports_state() {
        let audio = Audio::new(0.32, false);
        assert!(!audio.unavailable());
        assert_eq!(audio.state.load(Ordering::Relaxed), STATE_STARTING);
    }

    #[test]
    fn clamps_volume() {
        let mut audio = Audio::new(0.32, false);
        audio.set_volume(2.0);
        assert!((audio.volume - 1.0).abs() < f32::EPSILON);
        audio.set_volume(-1.0);
        assert!(audio.volume.abs() < f32::EPSILON);
    }
}

#[cfg(all(test, not(feature = "audio")))]
mod silent_tests {
    use super::*;

    #[test]
    fn reports_unavailable_without_the_feature() {
        let audio = Audio::new(0.32, true);
        assert!(audio.unavailable());
    }
}
