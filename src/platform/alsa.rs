#![cfg(feature = "audio")]

use std::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void, CString};
use std::ptr;

use libloading::Library;

const SND_PCM_STREAM_PLAYBACK: c_int = 0;
const SND_PCM_FORMAT_FLOAT_LE: c_int = 14;
const SND_PCM_ACCESS_RW_INTERLEAVED: c_int = 3;
const LATENCY_NS: c_uint = 21_000;

type Pcm = c_void;
type OpenFn = unsafe extern "C" fn(*mut *mut Pcm, *const c_char, c_int, c_int) -> c_int;
type SetParamsFn =
    unsafe extern "C" fn(*mut Pcm, c_int, c_int, c_uint, c_uint, c_int, c_uint) -> c_int;
type WriteFn = unsafe extern "C" fn(*mut Pcm, *const c_void, c_ulong) -> c_long;
type DrainFn = unsafe extern "C" fn(*mut Pcm) -> c_int;
type CloseFn = unsafe extern "C" fn(*mut Pcm) -> c_int;
type RecoverFn = unsafe extern "C" fn(*mut Pcm, c_int, c_int) -> c_int;

pub fn prepare_environment() {
    // SAFETY: `getuid` has no preconditions and only reads process credentials.
    let uid = unsafe { libc::getuid() };
    let runtime = std::path::PathBuf::from(format!("/run/user/{uid}"));
    if std::env::var_os("XDG_RUNTIME_DIR").is_none_or(|value| value.is_empty()) && runtime.is_dir()
    {
        std::env::set_var("XDG_RUNTIME_DIR", &runtime);
    }
    if std::env::var_os("PIPEWIRE_RUNTIME_DIR").is_none_or(|value| value.is_empty()) {
        if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            std::env::set_var("PIPEWIRE_RUNTIME_DIR", dir);
        }
    }
    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none_or(|value| value.is_empty()) {
        if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            let bus = std::path::Path::new(&dir).join("bus");
            if bus.exists() {
                std::env::set_var(
                    "DBUS_SESSION_BUS_ADDRESS",
                    format!("unix:path={}", bus.display()),
                );
            }
        }
    }
    if std::env::var_os("PULSE_SERVER").is_none_or(|value| value.is_empty()) {
        if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            let pulse = std::path::Path::new(&dir).join("pulse/native");
            if pulse.exists() {
                std::env::set_var("PULSE_SERVER", format!("unix:{}", pulse.display()));
            }
        }
    }
}

pub fn open_device(requested: &str) -> Result<Device, String> {
    let candidates = candidate_devices(requested);
    let mut errors = Vec::new();
    for name in &candidates {
        match Device::open(name) {
            Ok(device) => {
                debug(&format!("usando dispositivo ALSA '{name}'"));
                return Ok(device);
            }
            Err(error) => {
                debug(&format!("dispositivo ALSA '{name}' falhou: {error}"));
                errors.push(format!("{name}: {error}"));
            }
        }
    }
    Err(format!(
        "nenhum dispositivo ALSA disponível ({})",
        errors.join("; ")
    ))
}

pub fn debug(message: &str) {
    if std::env::var_os("PIANO_AUDIO_DEBUG").is_some() {
        eprintln!("piano-duas-oitavas: audio: {message}");
    }
}

pub fn candidate_devices(requested: &str) -> Vec<String> {
    let cards = std::fs::read_to_string("/proc/asound/cards").unwrap_or_default();
    let pcms = std::fs::read_to_string("/proc/asound/pcm").unwrap_or_default();
    candidates_from(requested, &cards, &pcms)
}

fn candidates_from(requested: &str, cards_text: &str, pcms_text: &str) -> Vec<String> {
    let requested = requested.trim();
    let mut candidates = Vec::new();

    if !requested.is_empty() && requested != "auto" && requested != "default" {
        push_unique(&mut candidates, requested);
        if let Some(rest) = requested.strip_prefix("hw:") {
            push_unique(&mut candidates, &format!("plughw:{rest}"));
        }
        return candidates;
    }

    push_unique(&mut candidates, "default");
    push_unique(&mut candidates, "pipewire");
    push_unique(&mut candidates, "pulse");

    let cards = parse_cards(cards_text);
    let pcms = parse_pcms(pcms_text);
    let mut hardware = Vec::new();
    for (card, device, name) in &pcms {
        let score = hardware_score(name);
        if let Some((_, id)) = cards.iter().find(|(index, _)| index == card) {
            hardware.push((score, format!("sysdefault:CARD={id}")));
            hardware.push((score, format!("plughw:CARD={id},DEV={device}")));
        }
        hardware.push((score, format!("plughw:{card},{device}")));
    }
    hardware.sort_by_key(|(score, _)| *score);
    for (_, name) in hardware {
        push_unique(&mut candidates, &name);
    }

    if candidates.len() <= 3 {
        for name in ["sysdefault", "plughw:0", "plughw:1", "plughw:2"] {
            push_unique(&mut candidates, name);
        }
    }

    candidates
}

fn push_unique(candidates: &mut Vec<String>, name: &str) {
    if !candidates.iter().any(|existing| existing == name) {
        candidates.push(name.to_string());
    }
}

fn parse_cards(text: &str) -> Vec<(u32, String)> {
    text.lines()
        .filter_map(|line| {
            let index = line.split_whitespace().next()?.parse::<u32>().ok()?;
            let start = line.find('[')? + 1;
            let end = line[start..].find(']')? + start;
            let id = line[start..end].trim();
            if id.is_empty() {
                None
            } else {
                Some((index, id.to_string()))
            }
        })
        .collect()
}

fn parse_pcms(text: &str) -> Vec<(u32, u32, String)> {
    text.lines()
        .filter(|line| line.contains("playback"))
        .filter_map(|line| {
            let (prefix, rest) = line.split_once(':')?;
            let (card, device) = prefix.trim().split_once('-')?;
            let card = card.parse::<u32>().ok()?;
            let device = device.parse::<u32>().ok()?;
            let name = rest.split(':').next()?.trim().to_string();
            Some((card, device, name))
        })
        .collect()
}

fn hardware_score(name: &str) -> u8 {
    let name = name.to_ascii_lowercase();
    if name.contains("hdmi") {
        3
    } else if name.contains("analog")
        || name.contains("usb")
        || name.contains("headset")
        || name.contains("speaker")
    {
        0
    } else {
        1
    }
}

pub struct Device {
    pcm: *mut Pcm,
    write: WriteFn,
    drain: DrainFn,
    close: CloseFn,
    recover: RecoverFn,
    _library: Library,
}

impl Device {
    pub fn open(name: &str) -> Result<Self, String> {
        // SAFETY: loading a shared library and resolving symbols is unsafe;
        // the signatures above mirror the documented ALSA C API.
        unsafe {
            let library = load_library()?;
            let open = *library
                .get::<OpenFn>(b"snd_pcm_open\0")
                .map_err(|error| format!("snd_pcm_open: {error}"))?;
            let set_params = *library
                .get::<SetParamsFn>(b"snd_pcm_set_params\0")
                .map_err(|error| format!("snd_pcm_set_params: {error}"))?;
            let write = *library
                .get::<WriteFn>(b"snd_pcm_writei\0")
                .map_err(|error| format!("snd_pcm_writei: {error}"))?;
            let drain = *library
                .get::<DrainFn>(b"snd_pcm_drain\0")
                .map_err(|error| format!("snd_pcm_drain: {error}"))?;
            let close = *library
                .get::<CloseFn>(b"snd_pcm_close\0")
                .map_err(|error| format!("snd_pcm_close: {error}"))?;
            let recover = *library
                .get::<RecoverFn>(b"snd_pcm_recover\0")
                .map_err(|error| format!("snd_pcm_recover: {error}"))?;

            let name = CString::new(name).map_err(|_| "nome ALSA inválido".to_string())?;
            let mut pcm = ptr::null_mut();
            let result = open(&mut pcm, name.as_ptr(), SND_PCM_STREAM_PLAYBACK, 0);
            if result < 0 || pcm.is_null() {
                return Err(format!("snd_pcm_open falhou: {result}"));
            }

            let result = set_params(
                pcm,
                SND_PCM_FORMAT_FLOAT_LE,
                SND_PCM_ACCESS_RW_INTERLEAVED,
                1,
                crate::audio::synth::SAMPLE_RATE,
                1,
                LATENCY_NS,
            );
            if result < 0 {
                close(pcm);
                return Err(format!("snd_pcm_set_params falhou: {result}"));
            }

            Ok(Self {
                pcm,
                write,
                drain,
                close,
                recover,
                _library: library,
            })
        }
    }

    pub fn write(&mut self, samples: &[f32]) -> Result<(), &'static str> {
        if samples.is_empty() {
            return Ok(());
        }
        let mut offset = 0;
        let mut recovered = false;
        while offset < samples.len() {
            // SAFETY: `pcm` was opened by `open` and stays valid until `Drop`;
            // the slice starts at `offset` and has `len - offset` floats.
            let frames = unsafe {
                (self.write)(
                    self.pcm,
                    samples[offset..].as_ptr().cast::<c_void>(),
                    (samples.len() - offset) as c_ulong,
                )
            };
            if frames < 0 {
                if recovered {
                    return Err("falha ao escrever no PCM");
                }
                recovered = true;
                // SAFETY: the same valid `pcm` handle is passed to ALSA's
                // recovery helper, which can restore the stream after xrun.
                let result = unsafe { (self.recover)(self.pcm, frames as c_int, 1) };
                if result < 0 {
                    return Err("falha ao recuperar o PCM");
                }
                continue;
            }
            if frames == 0 {
                return Err("PCM não aceitou amostras");
            }
            offset += frames as usize;
        }
        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // SAFETY: the handle is valid and owned by this value; drain and close
        // are called once, before the library handle is released.
        unsafe {
            (self.drain)(self.pcm);
            (self.close)(self.pcm);
        }
    }
}

fn load_library() -> Result<Library, String> {
    let mut errors = Vec::new();
    for name in ["libasound.so.2", "libasound.so"] {
        // SAFETY: loading a system shared library is unsafe; the handle is
        // kept alive for the lifetime of `Device`.
        match unsafe { Library::new(name) } {
            Ok(library) => return Ok(library),
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
    Err(format!("libasound não encontrada ({})", errors.join("; ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARDS: &str = "\
 0 [Generic        ]: HDA-Intel - HD-Audio Generic
                      HD-Audio Generic at 0xdfa88000 irq 79
 1 [LX3000         ]: USB-Audio - Microsoft LifeChat LX-3000
                      C-Media Electronics Inc. Microsoft LifeChat LX-3000 at usb-0000:0a:00.4-2
 2 [Generic_1      ]: HDA-Intel - HD-Audio Generic
                      HD-Audio Generic at 0xdfa80000 irq 80
";

    const PCMS: &str = "\
00-03: HDMI 0 : HDMI 0 : playback 1
00-07: HDMI 1 : HDMI 1 : playback 1
01-00: USB Audio : USB Audio : playback 1 : capture 1
02-00: ALC897 Analog : ALC897 Analog : playback 1 : capture 1
";

    fn position(candidates: &[String], name: &str) -> Option<usize> {
        candidates.iter().position(|candidate| candidate == name)
    }

    #[test]
    fn default_tries_session_then_non_hdmi_hardware() {
        let candidates = candidates_from("default", CARDS, PCMS);
        assert_eq!(&candidates[..3], ["default", "pipewire", "pulse"]);
        assert_eq!(
            candidates.iter().filter(|name| *name == "default").count(),
            1
        );

        let usb = position(&candidates, "plughw:CARD=LX3000,DEV=0").unwrap();
        let analog = position(&candidates, "plughw:CARD=Generic_1,DEV=0").unwrap();
        let hdmi = position(&candidates, "plughw:CARD=Generic,DEV=3").unwrap();
        assert!(usb < hdmi, "USB deveria vir antes do HDMI");
        assert!(analog < hdmi, "analógico deveria vir antes do HDMI");
    }

    #[test]
    fn auto_starts_with_default_and_includes_hardware() {
        let candidates = candidates_from("auto", CARDS, PCMS);
        assert_eq!(candidates.first().map(String::as_str), Some("default"));
        assert!(candidates.contains(&"plughw:CARD=Generic_1,DEV=0".to_string()));
        assert!(candidates.contains(&"plughw:2,0".to_string()));
    }

    #[test]
    fn specific_device_does_not_expand_to_unrelated_hardware() {
        assert_eq!(
            candidates_from("plughw:2,0", CARDS, PCMS),
            vec!["plughw:2,0".to_string()]
        );
        assert_eq!(
            candidates_from("hw:CARD=Generic_1,DEV=0", CARDS, PCMS),
            vec![
                "hw:CARD=Generic_1,DEV=0".to_string(),
                "plughw:CARD=Generic_1,DEV=0".to_string()
            ]
        );
    }

    #[test]
    fn missing_proc_files_use_numeric_fallbacks() {
        let candidates = candidates_from("default", "", "");
        for name in ["sysdefault", "plughw:0", "plughw:1", "plughw:2"] {
            assert!(
                candidates.contains(&name.to_string()),
                "faltou {name}: {candidates:?}"
            );
        }
    }

    #[test]
    fn parses_cards_and_playback_pcms() {
        assert_eq!(
            parse_cards(CARDS),
            vec![
                (0, "Generic".to_string()),
                (1, "LX3000".to_string()),
                (2, "Generic_1".to_string())
            ]
        );
        assert_eq!(
            parse_pcms(PCMS),
            vec![
                (0, 3, "HDMI 0".to_string()),
                (0, 7, "HDMI 1".to_string()),
                (1, 0, "USB Audio".to_string()),
                (2, 0, "ALC897 Analog".to_string())
            ]
        );
    }
}
