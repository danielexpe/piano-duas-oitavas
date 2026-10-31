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
        // SAFETY: `pcm` was opened by `open` and stays valid until `Drop`;
        // `samples` is a live buffer of `len` float frames.
        let frames = unsafe {
            (self.write)(
                self.pcm,
                samples.as_ptr().cast::<c_void>(),
                samples.len() as c_ulong,
            )
        };
        if frames < 0 {
            // SAFETY: the same valid `pcm` handle is passed to ALSA's recovery
            // helper, which can restore the stream after an underrun/xrun.
            unsafe {
                (self.recover)(self.pcm, frames as c_int, 1);
            }
            return Err("falha ao escrever no PCM");
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
