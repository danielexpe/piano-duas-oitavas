# Piano Duas Oitavas

Piano de duas oitavas em TUI/fbdev com mapeamento do teclado alfanumérico,
síntese polifônica e efeitos visuais.

Aplicação Rust para console Linux com TUI (`ratatui` + `crossterm`) e backend
framebuffer fbdev (`/dev/fb0`, Fase 6). Roda offline, sem rede em runtime; a fonte Inter é embutida no binário e o
piano é desenhado vetorialmente.

> **Estado:** Fases 0–5 concluídas e Fase 6 implementada (backend fbdev,
> texto vetorial, scaling, entrada raw e captura PNG determinística). A
> validação manual em VT real, console blanked e 24/16 bpp ainda está pendente.
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
./target/release/piano-duas-oitavas --capture-frame /tmp/piano.png
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
--no-audio                  inicia mudo
--audio-device <NAME>       PCM ALSA (padrão: default; auto tenta fallbacks)
--capture-frame <PATH>      PNG determinístico e sai
```

## Framebuffer e captura

O backend `fb` é sempre explícito (`--backend fb`) e exige um VT de texto
local com `/dev/fb0` acessível (grupo `video`). Ele entra em `KD_GRAPHICS`,
desbloqueia o console com `FBIOBLANK`, usa o `yoffset` real, tenta double
buffer/pan quando o driver oferece e cai para o buffer visível se o pan falhar.
Sinais, pânico e saída normal restauram `KD_TEXT` e o cursor. Em X/Wayland/SSH o
app recusa com código 3 e uma mensagem acionável.

`--capture-frame` renderiza a cena lógica Wide em 1344 × 756 e grava um PNG
determinístico sem abrir VT, framebuffer ou áudio. É útil para revisão visual e
QA.

## Áudio no console/VT

O app abre o PCM `default` (normalmente PipeWire/PulseAudio). Se o shell do VT
não tiver `XDG_RUNTIME_DIR`/DBus, o app preenche essas variáveis a partir de
`/run/user/<uid>` antes de abrir o áudio. Se ainda assim `default` falhar
(sessão de áudio ausente ou dispositivo ocupado), ele tenta `pipewire`,
`pulse` e, por último, dispositivos de hardware, preferindo analógico/USB a
HDMI.

Para escolher ou forçar um dispositivo:

```bash
./target/release/piano-duas-oitavas --backend fb --audio-device auto
./target/release/piano-duas-oitavas --backend fb --audio-device sysdefault:CARD=Generic_1
./target/release/piano-duas-oitavas --backend fb --audio-device plughw:2,0
```

`--audio-device default` (padrão) usa o PCM padrão e só cai para alternativas
se ele falhar; um nome explícito como `plughw:2,0` é usado sem fallback. Para
diagnosticar, rode com `PIANO_AUDIO_DEBUG=1` e veja no stderr qual dispositivo
foi escolhido e por que os outros falharam.

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

> **Atenção:** o último comando de build dos portões usa
> `--no-default-features`, então `target/release/piano-duas-oitavas` fica
> **sem áudio**. Antes de testar som, rode `cargo build --release` novamente.

Testes sem hardware cobrem ABI das structs de fbdev, formatos 32/24/16 bpp,
pan/fallback, geometria, parser de sequências, layout, texto e captura PNG. A
matriz manual em VT real está em `docs/05`.

## Limitações

- `--backend auto` usa TUI; o framebuffer é sempre explícito.
- Sem DRM/KMS na primeira versão; hardware sem `/dev/fb0` não é suportado.
- Sem mouse, MIDI, gravação/reprodução, metrônomo, transposição ou
  remapeamento no v1.
- Sem persistência de preferências.
- O key-up varia por terminal; o HUD mostra o modo (`real` ou `auto`).
- Sem vsync: tearing leve é aceitável.
- Troca de VT durante a execução pode deixar resíduos; reinicie o app.
- O backend fb está implementado, mas a validação em VT real, 24/16 bpp e
  telas menores segue pendente.

## Licença

Código MIT. Arte original CC BY 4.0. Fonte Inter 4.1 sob OFL 1.1
(vendorizada em `assets/fonts/`). Veja `THIRD_PARTY_NOTICES.md` e `CREDITS.md`.
