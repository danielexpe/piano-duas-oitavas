# Piano Duas Oitavas

Piano de duas oitavas em TUI/fbdev com mapeamento do teclado alfanumérico,
síntese polifônica e efeitos visuais.

Aplicação Rust para console Linux com TUI (`ratatui` + `crossterm`) e backend
framebuffer fbdev (`/dev/fb0`, Fase 6). Roda offline, sem rede em runtime; a fonte Inter é embutida no binário e o
piano é desenhado vetorialmente.

> **Estado:** Fase 5 concluída (pacote desktop TUI com fonte embutida e áudio
> sintetizado). O backend framebuffer e a captura PNG ainda não foram
> implementados; este README descreve o produto alvo.
> Especificação: `docs/01`–`08`. Memória operacional: `AGENTS.md`.
> Arquitetura: `framework.md`.

## O que faz

- 24 notas (duas oitavas cromáticas), 14 teclas brancas e 10 pretas.
- Cada nota mostra a tecla física que a aciona; as teclas do teclado
  alfanumérico também aparecem na tela e acendem junto.
- Som de piano sintetizado em runtime, com 8 vozes simultâneas.
- Sustain, mudança de oitava, volume e mudo.
- Interface pt-BR/en, ajuda modal e efeitos visuais acessíveis.
- Funciona sem áudio (silencioso) e sem framebuffer (TUI).

## Requisitos

- Linux x86_64, Rust stable (MSRV 1.88), terminal local para a TUI.
- Áudio: ALSA (`libasound`) disponível em runtime; opcional, ausência não é
  erro.
- Para o console: VT de texto local (`tty` = `/dev/ttyN`), `/dev/fb0` e
  usuário no grupo `video`.

## Compilar e rodar

```bash
cargo build --release
./target/release/piano-duas-oitavas                  # TUI (backend auto)
./target/release/piano-duas-oitavas --backend fb     # console via framebuffer
./target/release/piano-duas-oitavas --lang en --octave 4
./target/release/piano-duas-oitavas --no-audio
```

Opções principais:

```text
--lang <pt|en>              idioma inicial (padrão: pt)
--backend <auto|tui|fb>     auto usa TUI; fb é sempre explícito
--fb-device <PATH>          padrão: /dev/fb0
--octave <1..=6>            oitava base inicial (padrão: 3)
--volume <0.0..=1.0>        volume inicial (padrão: 0.32)
--seed <u64>                reservado; sem aleatoriedade no v1
--color <auto|truecolor|256|16|never>
--no-animations
--no-audio
--capture-frame <PATH>      PNG determinístico e sai
```

## Instalação

```bash
./install.sh --prefix "$HOME/.local"
```

O instalador também adiciona o usuário ao grupo `video` (pode pedir `sudo`).
Depois de entrar no grupo, saia e entre na sessão ou use `newgrp video`.

## Controles

O app abre direto no piano. A oitava base padrão é 3 (C3–B4).

| Tecla | Nota / ação |
| --- | --- |
| `Z X C V B N M` | brancas da oitava base: C D E F G A B |
| `S D G H J` | pretas da oitava base: C# D# F# G# A# |
| `Q W E R T Y U` | brancas da oitava seguinte: C D E F G A B |
| `2 3 5 6 7` | pretas da oitava seguinte: C# D# F# G# A# |
| `Espaço` | sustain: segure; no fallback sem key-up, alterna (latch) |
| `←` / `→` | oitava base −1 / +1 (limites 1..=6) |
| `+` ou `=` | volume +5% |
| `-` ou `_` | volume −5% |
| `F1` | abre/fecha a ajuda |
| `F2` | alterna pt-BR/en |
| `F3` | liga/desliga o mudo |
| `Esc` | fecha a ajuda; se fechada, sai |
| `Ctrl+C` | sai |

`Q` e `M` são notas (Dó e Si), não atalhos de sair/mudo. `Repeat` do terminal
é ignorado. Sem key-up real, cada nota dura 750 ms; com suporte ao protocolo de
teclado aprimorado, soltar a tecla encerra a nota.

Preferências não são salvas entre execuções: volume, idioma, oitava e mudo
voltam ao padrão (ou ao valor das flags) a cada início.

## Diagnóstico de console

```bash
./testes_debug.sh
```

Rode **dentro do VT afetado** e envie o `testes_debug.txt`. O script não
instala nada e restaura o console ao final.

## Testes e portões

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test
cargo test --no-default-features
cargo xtask assets --check
cargo xtask notices --check
cargo build --release
cargo build --release --no-default-features
```

Snapshots de TUI: `UPDATE_SNAPSHOTS=1 cargo test --test tui_snapshots`
regenera `tests/snapshots/`; revise o diff. O teste PTY usa `script(1)` e
confirma a restauração do terminal, sendo pulado se `script` não existir.

## Limitações

- `--backend auto` usa TUI; o framebuffer é sempre explícito.
- Sem DRM/KMS na primeira versão; hardware sem `/dev/fb0` não é suportado.
- Sem mouse, MIDI, gravação/reprodução, metrônomo, transposição ou
  remapeamento no v1.
- Sem persistência de preferências.
- O key-up varia por terminal; o HUD mostra o modo (`real` ou `auto`).
- Sem vsync: tearing leve é aceitável.
- Troca de VT durante a execução pode deixar resíduos; reinicie o app.
- A validação em VT real, 24/16 bpp e telas menores está pendente até a
  Fase 6.

## Licença

Código MIT. Arte original CC BY 4.0. Fonte Inter 4.1 sob OFL 1.1
(vendorizada em `assets/fonts/`). Veja `THIRD_PARTY_NOTICES.md` e `CREDITS.md`.
