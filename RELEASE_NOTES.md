# Piano Duas Oitavas 0.2.0

Segunda release, com o backend framebuffer para console Linux, captura PNG
determinística e áudio ALSA mais robusto no VT.

## Destaques

- Backend fbdev (`--backend fb`): `/dev/fb0` em 32/24/16 bpp, `KD_GRAPHICS`,
  `FBIOBLANK`, `yoffset` real, double buffer/pan com fallback, scaling
  bilinear e layout Wide/Compact/Minimal.
- Texto no framebuffer com a fonte Inter 4.1 embutida (`fontdue`), piano e
  teclado físico vetoriais (`tiny-skia`).
- Entrada raw de `/dev/tty` com setas, F1–F3, `Esc` isolado e `Ctrl+C`.
- Restauração de `KD_TEXT`, cursor e terminal em saída, erro, pânico,
  SIGINT/SIGTERM/SIGHUP.
- Captura PNG determinística em 1344 × 756 com `--capture-frame`.
- Áudio: preparo de `XDG_RUNTIME_DIR`/PipeWire/DBus no VT, seleção com
  `--audio-device`, fallback `default` → `pipewire`/`pulse` → hardware
  (analógico/USB antes de HDMI), recuperação de xrun/short write e diagnóstico
  com `PIANO_AUDIO_DEBUG=1`.
- TUI e recursos anteriores preservados: 24 notas, efeitos, sustain, oitavas,
  volume, mudo, ajuda, i18n pt-BR/en, acessibilidade e áudio sintetizado.

## Validado

- Portões automatizados verdes com e sem a feature `audio`.
- 150 testes unitários + 5 CLI + 2 PTY + 15 snapshots com `audio`; 144
  unitários sem `audio`; 2 benchmarks ignorados.
- `cargo fmt`, `clippy -D warnings` (com/sem features), `xtask assets --check`,
  `xtask notices --check` e builds release.
- Capturas visuais em 1344 × 756 e 1920 × 1080, incluindo mono e inglês.
- Frame fb ativo ~17 ms em 1920 × 1080; o loop não redesenha quando a cena
  está estável.
- Fluxo principal no VT real (interface e áudio) aprovado pelo usuário após a
  correção de áudio.
- Pacote `tar.gz` + `.sha256`; smoke em `/tmp` sem a pasta `assets/`;
  instalação testada com prefixo temporário e `--skip-group`.

## Pendente de validação manual

- 24/16 bpp em hardware real.
- Console previamente blanked/DPMS off e retomada após blank por inatividade.
- Pan real em driver com `ypanstep > 0` (o alvo observado anuncia `pan: 0,0`).
- Key-up real por terminal e a matriz completa de `docs/05`.
- Áudio em headset e analógico, com dispositivo ocupado e ausente.
- Telas menores (Compact/Minimal) em hardware real.

## Limitações

- `--backend auto` usa TUI; o framebuffer é sempre explícito.
- Sem DRM/KMS nesta versão.
- Key-up real depende do terminal; no fbdev, o fallback usa auto-release de
  750 ms e sustain latch.
- Sem persistência de preferências, mouse, MIDI, gravação ou metrônomo.
- Sem vsync: tearing leve é aceitável.
- O fallback automático de áudio pode escolher o analógico quando o PipeWire
  não estiver acessível; use `--audio-device` para forçar um PCM.

## Instalação

```bash
tar -xzf piano-duas-oitavas-0.2.0-x86_64-unknown-linux-gnu.tar.gz
cd piano-duas-oitavas-0.2.0-x86_64-unknown-linux-gnu
./install.sh --prefix "$HOME/.local"
```

Para testar sem alterar o grupo `video`:

```bash
./install.sh --prefix /tmp/piano-prefix --skip-group
```

> Não inclua o hash do próprio pacote nestas notas: ele muda quando as notas
> mudam. Registre o hash no `AGENTS.md` e na página da release.
