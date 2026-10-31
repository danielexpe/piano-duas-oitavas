//! Domain and backends for Piano Duas Oitavas.
//!
//! The crate keeps a pure `core` (notes, keymap, voices, envelope), a
//! `state` session, an `app` orchestrator, shared `render` scene and a
//! `platform` boundary for framebuffer, input and terminal signals, plus the
//! fbdev backend loop and deterministic PNG capture.

pub mod app;
pub mod assets;
pub mod audio;
pub mod capture;
pub mod cli;
pub mod core;
pub mod error;
pub mod i18n;
pub mod input;
pub mod platform;
pub mod render;
pub mod state;
pub mod theme;
pub mod util;
