# Third-party notices

Piano Duas Oitavas includes third-party software and fonts. The full license
texts ship with the sources and with the packages listed in `Cargo.lock`.

## Rust crates

This table is generated from `Cargo.lock`. `cargo xtask notices --check` fails
when the lockfile and this list diverge.

<!-- dependencies:start -->
| Crate | Version | License |
| --- | --- | --- |
| `adler2` | `2.0.1` | 0BSD OR MIT OR Apache-2.0 |
| `allocator-api2` | `0.2.21` | MIT OR Apache-2.0 |
| `anstream` | `1.0.0` | MIT OR Apache-2.0 |
| `anstyle` | `1.0.14` | MIT OR Apache-2.0 |
| `anstyle-parse` | `1.0.0` | MIT OR Apache-2.0 |
| `anstyle-query` | `1.1.5` | MIT OR Apache-2.0 |
| `anstyle-wincon` | `3.0.11` | MIT OR Apache-2.0 |
| `approx` | `0.5.1` | Apache-2.0 |
| `arrayref` | `0.3.9` | BSD-2-Clause |
| `arrayvec` | `0.7.8` | MIT OR Apache-2.0 |
| `autocfg` | `1.5.1` | Apache-2.0 OR MIT |
| `bitflags` | `1.3.2` | MIT/Apache-2.0 |
| `bitflags` | `2.13.2` | MIT OR Apache-2.0 |
| `block-buffer` | `0.10.4` | MIT OR Apache-2.0 |
| `by_address` | `1.2.1` | MIT OR Apache-2.0 |
| `bytemuck` | `1.25.2` | Zlib OR Apache-2.0 OR MIT |
| `castaway` | `0.2.4` | MIT |
| `cfg-if` | `1.0.5` | MIT OR Apache-2.0 |
| `clap` | `4.6.7` | MIT OR Apache-2.0 |
| `clap_builder` | `4.6.7` | MIT OR Apache-2.0 |
| `clap_derive` | `4.6.7` | MIT OR Apache-2.0 |
| `clap_lex` | `1.1.1` | MIT OR Apache-2.0 |
| `colorchoice` | `1.0.5` | MIT OR Apache-2.0 |
| `compact_str` | `0.9.1` | MIT |
| `convert_case` | `0.10.0` | MIT |
| `core_maths` | `0.1.1` | MIT |
| `cpufeatures` | `0.2.17` | MIT OR Apache-2.0 |
| `crc32fast` | `1.5.3` | MIT OR Apache-2.0 |
| `crossterm` | `0.29.0` | MIT |
| `crossterm_winapi` | `0.9.1` | MIT |
| `crypto-common` | `0.1.7` | MIT OR Apache-2.0 |
| `darling` | `0.24.1` | MIT |
| `darling_core` | `0.24.1` | MIT |
| `darling_macro` | `0.24.1` | MIT |
| `deranged` | `0.5.9` | MIT OR Apache-2.0 |
| `derive_more` | `2.1.1` | MIT |
| `derive_more-impl` | `2.1.1` | MIT |
| `digest` | `0.10.7` | MIT OR Apache-2.0 |
| `document-features` | `0.2.12` | MIT OR Apache-2.0 |
| `either` | `1.19.0` | MIT OR Apache-2.0 |
| `equivalent` | `1.0.3` | Apache-2.0 OR MIT |
| `errno` | `0.3.14` | MIT OR Apache-2.0 |
| `fdeflate` | `0.3.7` | MIT OR Apache-2.0 |
| `flate2` | `1.1.10` | MIT OR Apache-2.0 |
| `foldhash` | `0.1.5` | Zlib |
| `foldhash` | `0.2.0` | Zlib |
| `fontdue` | `0.9.5` | MIT OR Apache-2.0 OR Zlib |
| `generic-array` | `0.14.7` | MIT |
| `hashbrown` | `0.15.5` | MIT OR Apache-2.0 |
| `hashbrown` | `0.17.1` | MIT OR Apache-2.0 |
| `heck` | `0.5.0` | MIT OR Apache-2.0 |
| `ident_case` | `1.0.1` | MIT/Apache-2.0 |
| `indexmap` | `2.14.2` | Apache-2.0 OR MIT |
| `indoc` | `2.0.8` | MIT OR Apache-2.0 |
| `instability` | `0.3.14` | MIT |
| `is_terminal_polyfill` | `1.70.2` | MIT OR Apache-2.0 |
| `itertools` | `0.14.0` | MIT OR Apache-2.0 |
| `itoa` | `1.0.18` | MIT OR Apache-2.0 |
| `kasuari` | `0.4.13` | MIT OR Apache-2.0 |
| `libc` | `0.2.191` | MIT OR Apache-2.0 |
| `libloading` | `0.8.9` | ISC |
| `libm` | `0.2.16` | MIT |
| `line-clipping` | `0.3.8` | MIT OR Apache-2.0 |
| `linux-raw-sys` | `0.12.1` | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `litrs` | `1.0.1` | MIT OR Apache-2.0 |
| `lock_api` | `0.4.14` | MIT OR Apache-2.0 |
| `log` | `0.4.35` | MIT OR Apache-2.0 |
| `lru` | `0.18.5` | MIT |
| `memchr` | `2.8.3` | Unlicense OR MIT |
| `miniz_oxide` | `0.8.9` | MIT OR Zlib OR Apache-2.0 |
| `miniz_oxide` | `0.9.1` | MIT OR Zlib OR Apache-2.0 |
| `mio` | `1.2.4` | MIT |
| `num-conv` | `0.2.2` | MIT OR Apache-2.0 |
| `num-traits` | `0.2.19` | MIT OR Apache-2.0 |
| `num_threads` | `0.1.8` | MIT OR Apache-2.0 |
| `once_cell_polyfill` | `1.70.2` | MIT OR Apache-2.0 |
| `palette` | `0.7.7` | MIT OR Apache-2.0 |
| `palette_derive` | `0.7.7` | MIT OR Apache-2.0 |
| `palette_math` | `0.7.7` | MIT OR Apache-2.0 |
| `parking_lot` | `0.12.5` | MIT OR Apache-2.0 |
| `parking_lot_core` | `0.9.12` | MIT OR Apache-2.0 |
| `png` | `0.17.16` | MIT OR Apache-2.0 |
| `portable-atomic` | `1.15.0` | Apache-2.0 OR MIT |
| `powerfmt` | `0.2.1` | MIT OR Apache-2.0 |
| `proc-macro2` | `1.0.107` | MIT OR Apache-2.0 |
| `quote` | `1.0.47` | MIT OR Apache-2.0 |
| `ratatui` | `0.30.2` | MIT |
| `ratatui-core` | `0.1.2` | MIT |
| `ratatui-crossterm` | `0.1.2` | MIT |
| `ratatui-widgets` | `0.3.2` | MIT |
| `redox_syscall` | `0.5.18` | MIT |
| `rustc_version` | `0.4.1` | MIT OR Apache-2.0 |
| `rustix` | `1.1.5` | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `rustversion` | `1.0.23` | MIT OR Apache-2.0 |
| `ryu` | `1.0.23` | Apache-2.0 OR BSL-1.0 |
| `scopeguard` | `1.2.0` | MIT OR Apache-2.0 |
| `semver` | `1.0.28` | MIT OR Apache-2.0 |
| `serde` | `1.0.229` | MIT OR Apache-2.0 |
| `serde_core` | `1.0.229` | MIT OR Apache-2.0 |
| `serde_derive` | `1.0.229` | MIT OR Apache-2.0 |
| `serde_spanned` | `0.6.9` | MIT OR Apache-2.0 |
| `sha2` | `0.10.9` | MIT OR Apache-2.0 |
| `signal-hook` | `0.3.18` | Apache-2.0/MIT |
| `signal-hook-mio` | `0.2.5` | MIT OR Apache-2.0 |
| `signal-hook-registry` | `1.4.8` | MIT OR Apache-2.0 |
| `simd-adler32` | `0.3.10` | MIT |
| `smallvec` | `1.16.4` | MIT OR Apache-2.0 |
| `static_assertions` | `1.1.0` | MIT OR Apache-2.0 |
| `strict-num` | `0.1.1` | MIT |
| `strsim` | `0.11.1` | MIT |
| `strum` | `0.28.0` | MIT |
| `strum_macros` | `0.28.0` | MIT |
| `syn` | `2.0.119` | MIT OR Apache-2.0 |
| `syn` | `3.0.9` | MIT OR Apache-2.0 |
| `thiserror` | `2.0.21` | MIT OR Apache-2.0 |
| `thiserror-impl` | `2.0.21` | MIT OR Apache-2.0 |
| `time` | `0.3.55` | MIT OR Apache-2.0 |
| `time-core` | `0.1.9` | MIT OR Apache-2.0 |
| `tiny-skia` | `0.11.4` | BSD-3-Clause |
| `tiny-skia-path` | `0.11.4` | BSD-3-Clause |
| `toml` | `0.8.23` | MIT OR Apache-2.0 |
| `toml_datetime` | `0.6.11` | MIT OR Apache-2.0 |
| `toml_edit` | `0.22.27` | MIT OR Apache-2.0 |
| `toml_write` | `0.1.2` | MIT OR Apache-2.0 |
| `ttf-parser` | `0.25.1` | MIT OR Apache-2.0 |
| `typenum` | `1.20.1` | MIT OR Apache-2.0 |
| `unicode-ident` | `1.0.27` | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `unicode-segmentation` | `1.13.3` | MIT OR Apache-2.0 |
| `unicode-truncate` | `2.0.1` | MIT OR Apache-2.0 |
| `unicode-width` | `0.2.0` | MIT OR Apache-2.0 |
| `utf8parse` | `0.2.2` | Apache-2.0 OR MIT |
| `version_check` | `0.9.5` | MIT/Apache-2.0 |
| `wasi` | `0.11.1+wasi-snapshot-preview1` | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `winapi` | `0.3.9` | MIT/Apache-2.0 |
| `winapi-i686-pc-windows-gnu` | `0.4.0` | MIT/Apache-2.0 |
| `winapi-x86_64-pc-windows-gnu` | `0.4.0` | MIT/Apache-2.0 |
| `windows-link` | `0.2.1` | MIT OR Apache-2.0 |
| `windows-sys` | `0.61.2` | MIT OR Apache-2.0 |
| `winnow` | `0.7.15` | MIT |
| `zlib-rs` | `0.6.8` | Zlib |
<!-- dependencies:end -->

## Fonts

| Font | License | License file |
| --- | --- | --- |
| Inter Regular 4.1 | OFL 1.1 | `assets/fonts/OFL-Inter.txt` |

## System libraries (not distributed)

| Library | License | Use |
| --- | --- | --- |
| ALSA `libasound.so.2` | LGPL-2.1-or-later | carregada dinamicamente em runtime; não é distribuída no pacote |

## Assets

Original artwork is licensed under CC BY 4.0; see `LICENSE-ART`. Third-party
assets must record their license and attribution here before being embedded.

O v1 não embute imagens raster. O piano e os keycaps são desenhados
vetorialmente pelo próprio código.
