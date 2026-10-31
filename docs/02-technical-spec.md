# 02 — Especificação técnica

> Congelada na Fase 0 (2026-10-31), derivada de `docs/01-product.md`.

## 1. Plataforma e toolchain

- Linux x86_64; Rust stable, edição 2021, MSRV 1.88.
- `cargo fmt`, `cargo clippy`, `cargo test`, `cargo xtask`.
- Sem rede em runtime; áudio via ALSA carregado dinamicamente.
- Alvo de console: `/dev/fb0`, 1920 × 1080 a 32 bpp como referência,
  driver genérico (efifb, simpledrm, amdgpu, i915), VT de texto local.
- Formatos de framebuffer suportados: 32, 24 e 16 bpp; validação em hardware
  real fica pendente até a Fase 6.

## 2. Arquitetura em camadas

```text
core/ (puro) <- state/ <- app/ -> render/{scene,tui,fb} -> platform/fb
                         \-> audio/, assets/, i18n/, capture/
```

Regras:

- `core/` não conhece UI, IO, tempo, RNG global, áudio ou assets.
- `state/` modela a sessão e as transições; depende de `core/`.
- `app/` orquestra backend, áudio, entrada e relógio (`Instant`).
- `render/scene.rs` descreve a cena lógica uma vez; `tui.rs` e `fb.rs` a
  consomem sem mutar estado.
- `platform/` isola `unsafe`, ioctls, `mmap`, `termios` e sinais, com
  `// SAFETY:` em todo bloco.
- Nenhum módulo de domínio imprime logs na tela.

## 3. Módulos previstos

| Módulo | Responsabilidade |
| --- | --- |
| `cli.rs` | flags e enums de opções |
| `input.rs` | `Action`, `NoteKey`, normalização e `map_key` |
| `app.rs` | estado, transições, timers, loop TUI/fb |
| `core/note.rs` | `PitchClass`, `Octave`, `MidiNote`, nomes e frequência |
| `core/keymap.rs` | tabela ABNT2 ↔ nota, aliases, validação de unicidade |
| `core/voice.rs` | alocação de 8 vozes, roubo de voz, eventos de nota |
| `core/envelope.rs` | ADSR puro, release, normalização de pico |
| `state/session.rs` | oitava, volume, mudo, idioma, sustain, ajuda, vozes ativas |
| `state/effects.rs` | envelopes visuais (press, glow, ripple, nota flutuante) |
| `render/layout.rs` | breakpoints Wide/Compact/Minimal e geometria |
| `render/scene.rs` | cena lógica compartilhada (piano, teclado, HUD) |
| `render/tui.rs` | desenho `ratatui` |
| `render/fb.rs` | canvas, escala, blend, desenho vetorial |
| `render/text.rs` | texto no fb (`fontdue`) |
| `platform/fb.rs` | device, console, ioctls, mmap, sinais |
| `platform/input.rs` | leitura de `/dev/tty`, sequências de teclas |
| `audio.rs` | síntese, thread, ALSA dinâmico, feature `audio` |
| `assets.rs` | `include_bytes!`, decode/cache da fonte |
| `capture.rs` | PNG determinístico de QA |
| `i18n/` | catálogo de strings |
| `theme.rs` | tokens de cor e variantes |
| `util/rng.rs` | reservado; sem uso no v1 |
| `util/tty.rs` | guardas de terminal |

## 4. Modelo de entrada

### 4.1 Normalização

- Letras ASCII → minúsculas (`CapsLock` e Shift não mudam a nota).
- Fileira numérica: `2 3 5 6 7` e aliases `@ # % &` (ver `docs/01`).
- `KeyEventKind::Repeat` é ignorado para notas.
- `Ctrl`/`Alt` em tecla de nota é ignorado; `Ctrl+C` é `Quit`.
- Atalhos globais têm precedência sobre notas: `F1`, `F2`, `F3`, `Espaço`,
  `←`, `→`, `+`, `=`, `-`, `_`, `Esc`, `Ctrl+C`.
- `Q` e `M` são notas; não são atalhos.

### 4.2 Key-up e fallback

- **TUI:** tentar habilitar o protocolo de teclado aprimorado do crossterm
  (`REPORT_EVENT_TYPES | DISAMBIGUATE_ESCAPE_CODES`). Se suportado, usar
  `Release` para encerrar a nota e a soltura de Espaço para o sustain; ao sair,
  remover as flags com guarda RAII.
- **Fallback (TUI sem protocolo e fbdev com tty em modo texto):** não há
  key-up observável. Cada nota recebe auto-release em 750 ms; `Espaço` alterna
  sustain (latch). O HUD indica `Sustain: latch`.
- **fbdev:** v1 lê caracteres de `/dev/tty` em raw mode (sem key-up). A
  leitura real de press/release via evdev é backlog da Fase 7.
- O app mantém um único caminho de domínio: `note_on`, `note_off` e
  `expire(now)`. A origem do key-up é detalhe do backend.

### 4.3 Sequências no fb

- Setas: `ESC [ D` / `ESC [ C`.
- `F1`: `ESC O P` ou `ESC [ 11~`; `F2`: `ESC O Q` ou `ESC [ 12~`;
  `F3`: `ESC O R` ou `ESC [ 13~`.
- `Esc` isolado: reconhecido após timeout de ~20 ms para não confundir com
  prefixo de sequência.
- Teclas de nota: bytes ASCII; aliases deslocados conforme `docs/01`.

## 5. Máquina de estados e loop

```text
Screen = Piano | Help
Action = NoteOn(MidiNote) | NoteOff(MidiNote) | SustainPress | SustainRelease
       | OctaveDown | OctaveUp | VolumeDown | VolumeUp | ToggleMute
       | ToggleLanguage | ToggleHelp | CloseOrQuit | Quit | Tick
```

- `App::handle(Action)` centraliza o roteamento; ações globais antes da tela.
- `App::update(now)` vence auto-release, animações e nota flutuante.
- Não há `sleep` no domínio; timers usam `Instant` no `App`.
- Loop ocioso: timeout 33 ms. Com animação ativa: 16 ms (~60 FPS).
- `--no-animations` mantém highlight estático e usa timeout de 33 ms.
- Nunca bloquear > ~100 ms sem tratar sinais/saída.
- Pausa não existe no v1; a ajuda modal não pausa o áudio já emitido, apenas
  ignora novas notas.

## 6. Renderização compartilhada

- Canvas lógico: **1344 × 756** (70% de 1920 × 1080).
- Fb: fundo ocupa a tela inteira; composição escalada e centralizada.
- TUI: breakpoints Wide / Compact / Minimal; mínimo 80 × 24.
- `render/scene.rs` produz uma lista de teclas com:
  - nota MIDI, branca/preta, posição lógica;
  - tecla física e rótulo;
  - estado `Idle | Pressed | Sustained | Releasing`;
  - fase do efeito (0..=1) e intensidade;
  - linhas do teclado alfanumérico com keycaps mapeadas.
- `tui.rs` desenha com `ratatui`; `fb.rs` com `tiny-skia` e `fontdue`.
- `tiny-skia` usa RGBA **pré-multiplicado**; respeitar no blend.
- Cores em tokens semânticos, com variantes truecolor/256/16/mono.
- Cache: fonte decodificada uma vez; fundo estático por tamanho; nunca
  rasterizar glifos ou decodificar fonte a cada frame.
- TUI e fb compartilham o mesmo estado; nenhum renderer altera o `App`.

### 6.1 Layout lógico

| Faixa | Y | Conteúdo |
| --- | --- | --- |
| Cabeçalho | 24–92 | título, oitava, volume, mudo, idioma |
| Nota flutuante | 100–146 | nome da última nota (600 ms) |
| Piano | 150–570 | 14 brancas, 10 pretas sobrepostas |
| Status | 580–606 | sustain, áudio, key-up/latch |
| Teclado físico | 610–720 | 4 fileiras de keycaps (dígitos, QWERTY, home, base) |
| Rodapé | 724–744 | dicas F1/F2/F3/Esc |

- Brancas: ~89 px de largura; pretas: ~56 px de largura por ~260 px de altura,
  posicionadas nas fronteiras C-D, D-E, F-G, G-A, A-B de cada oitava.
- Teclado físico: keycaps de ~64 × 22 px; teclas não mapeadas em tom apagado.
- O foco visual é o piano; o teclado físico é uma legenda interativa.

## 7. CLI

```text
piano-duas-oitavas [OPTIONS]

  --lang <pt|en>              idioma inicial (padrão: pt)
  --backend <auto|tui|fb>     auto usa TUI; fb é sempre explícito
  --fb-device <PATH>          padrão: /dev/fb0
  --octave <1..=6>            oitava base inicial (padrão: 3)
  --volume <0.0..=1.0>        volume inicial (padrão: 0.32)
  --seed <u64>                reservado; sem aleatoriedade no v1
  --color <auto|truecolor|256|16|never>
  --no-animations
  --no-audio                  inicia mudo
  --capture-frame <PATH>      PNG determinístico e sai
  -h, --help
  -V, --version
```

- `--backend auto` nunca escolhe fbdev.
- `--capture-frame` não abre áudio, framebuffer nem terminal interativo.
- `NO_COLOR` tem prioridade; `--color never` força monocromático.
- Flags de imagem (`--image-protocol`, `--no-images`) não existem no v1: não
  há imagens raster.

Códigos de saída: `0` sucesso; `1` runtime/IO/captura; `2` CLI inválida;
`3` backend indisponível (fb sem permissão/VT, TUI sem TTY). Mensagens de erro
informam caminho, causa e ação.

## 8. Áudio

- Feature `audio` (padrão ligada); `--no-default-features` compila e testa sem
  ALSA.
- ALSA via `libloading`; biblioteca/dispositivo ausente = silêncio, sem erro
  fatal. Thread de áudio separada com canal limitado (64 mensagens).
- Formato interno: mono `f32`, 48 kHz; ALSA converte para o dispositivo.
- 8 vozes; onda aditiva com fundamental + 4 harmônicos, amplitudes
  `[1.0, 0.42, 0.22, 0.12, 0.07]` e leve inarmonicidade por harmônico;
  normalização de pico ≤ 1.0 e master padrão 0.32.
- Envelope: ataque 8 ms, decay 350 ms, sustain 0.25, release 180 ms
  (key-up) e 450 ms (sustain solto). Fade de 5 ms no roubo de voz e no mute.
- Inicialização lazy na primeira nota (mantém o startup rápido). A thread é
  encerrada com `AllNotesOff`, fade de ~120 ms e timeout de 250 ms; a
  restauração do terminal/console nunca depende do áudio.
- Mensagens: `NoteOn(midi, velocity)`, `NoteOff(midi)`, `AllNotesOff`,
  `SetVolume`, `SetMuted`, `Shutdown`. Canal cheio descarta a mensagem mais
  nova, sem bloquear o loop.
- Testes de síntese não usam hardware: amostras finitas, pico ≤ 1.0, duração
  do envelope e ausência de NaN.

## 9. Desempenho

| Métrica | Alvo |
| --- | --- |
| Startup até primeira tela | < 400 ms |
| Frame de animação (fb 1080p) | ≤ 16 ms |
| Frame ocioso | ≤ 33 ms |
| RSS | < 120 MB |
| Binário release | < 15 MB |
| Latência de nota (press → som) | < 30 ms em ALSA local |

## 10. Persistência e determinismo

- Nenhuma preferência persiste no v1; sem leitura/escrita de config.
- Não há RNG no comportamento musical; `--seed` é aceito e reservado.
- `--capture-frame` é determinístico para a mesma build, idioma e estado
  inicial (cena ociosa, pt-BR, oitava 3, volume 0.32, áudio indicado como
  disponível).

## 11. Backends e restauração

- Contrato: entrar com guarda RAII, desenhar de `&App`, apresentar, ler eventos
  com timeout, restaurar em saída normal, erro, pânico e sinais.
- TUI: raw mode + alternate screen + keyboard enhancement (se disponível);
  restaurar cursor e flags ao sair.
- Fb: `KD_GRAPHICS`, `FBIOBLANK(FB_BLANK_UNBLANK)`, validar `line_length`,
  `smem_len`, `yres_virtual`, `ypanstep` e usar o `yoffset` real; `KD_TEXT` e
  cursor restaurados ao sair.
- Sinais: SIGINT/SIGTERM/SIGHUP acionam flag atômica; `Ctrl+C` em raw mode é
  tratado como `Action::Quit`.
- Após troca de VT ou perda do device, restaurar e sair com erro acionável.
