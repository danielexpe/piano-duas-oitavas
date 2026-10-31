//! Domain and backends for Piano Duas Oitavas.
//!
//! The crate keeps a pure `core` (notes, keymap, voices, envelope), a
//! `state` session, an `app` orchestrator, shared `render` scene and a
//! `platform` boundary for terminal signals. The fbdev backend arrives in a
//! later phase.

pub mod app;
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
