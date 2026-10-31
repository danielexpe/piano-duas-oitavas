# Third-party notices

Piano Duas Oitavas includes third-party software and fonts. The full license texts ship
with the sources and with the packages listed in `Cargo.lock`.

## Rust crates

| Crate | License |
| --- | --- |
| `clap` | MIT OR Apache-2.0 |
| `crossterm` | MIT |
| `fontdue` | MIT OR Apache-2.0 OR Zlib |
| `libc` | MIT OR Apache-2.0 |
| `libloading` | ISC |
| `ratatui` | MIT |
| `thiserror` | MIT OR Apache-2.0 |
| `tiny-skia` | BSD-3-Clause |

Build-time crates in `xtask`:

| Crate | License |
| --- | --- |
| `serde` | MIT OR Apache-2.0 |
| `sha2` | MIT OR Apache-2.0 |
| `toml` | MIT OR Apache-2.0 |

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
