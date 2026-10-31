# 05 — Plano de testes

> Congelado na Fase 0. Atualizado na Fase 1 com os portões automatizados e o
> smoke em PTY. Atualizado na Fase 2 com snapshots de TUI, efeitos, breakpoints
> e teste PTY de restauração. Atualizado na Fase 3 com o ciclo de vozes,
> key-up real, invariantes e testes de tempo com relógio injetado; nenhuma
> validação em terminal/VT real foi executada ainda.

## 1. Pirâmide

1. **Unitários de domínio** — notas, mapeamento, vozes, envelope, sessão.
2. **Propriedades/invariantes** — unicidade do mapa, 24 notas, limites, somas.
3. **Snapshots de TUI** — telas/estados/idiomas/breakpoints.
4. **Integração/PTY** — entrada real, resize, key-up e fallback, saída limpa.
5. **Sem hardware** — formatos fb, ABI, conversão, geometria, blend, áudio.
6. **Manual em hardware** — TUI em terminais reais e fb no VT.

## 2. Testes por área

| Área | Casos |
| --- | --- |
| Notas | C3 = 48, B4 = 71, C1 = 24, B7 = 107; frequência A4 = 440 Hz; nome `C#3`; oitava calculada por `midi / 12 - 1` |
| Mapeamento | 24 teclas → 24 notas únicas; 14 brancas/10 pretas; oitava base e base+1 corretas; letras maiúsculas normalizadas; aliases `@ # % &`; `¨` documentado como não confiável; `Repeat` ignorado |
| Conflitos | `Q` e `M` não disparam sair/mudo; `F1/F2/F3`, setas, `+`, `-`, `Espaço`, `Esc`, `Ctrl+C` têm precedência |
| Oitava | deslocamento ±1; limites 1..=6; shift libera vozes/sustain e cancela timers |
| Vozes | 8 simultâneas; 9ª rouba a mais antiga; retrigger não duplica; release atinge a voz correta; roubo sem clique (fade) |
| Envelope | ataque/decay/sustain/release; release 180 ms; release de sustain 450 ms; pico ≤ 1,0; sem NaN/Inf |
| Auto-release | dispara em 750 ms; reataque reinicia o timer; timer cancelado ao sair/trocar de oitava |
| Sustain | modo key-up: segurar adia releases; fallback: latch alterna; desligar libera notas adiadas; HUD reflete o modo |
| Volume/mudo | passo 0,05; clamp 0..=1; 0% silencioso; mudo não para animações; sem ALSA = silêncio |
| Transições | piano ↔ ajuda; ajuda ignora notas; `Esc` fecha ajuda; `Esc` sem ajuda sai; sair libera vozes |
| RNG/seed | sem efeito no comportamento musical; captura determinística com o mesmo seed |
| CLI | defaults; todos os flags; `--octave` fora de 1..=6 e `--volume` fora de 0..=1 falham com código 2; `--backend fb` sem VT/permissão = código 3; `--capture-frame` não abre tty/fb/áudio |
| TUI | snapshots: Wide/Compact/Minimal × pt/en × idle/pressed/sustained/help/muted/audio-indisponível; resize preserva estado; 80 × 24 sem corte |
| Captura | 1344 × 756; determinística; não vazia; idêntica em execuções repetidas com mesmo estado |
| fb sem hardware | 32/24/16 bpp; ABI 160/80; stride; yoffset != 0; pan e fallback; RGB/BGR; escala bilinear |
| fb com hardware | 1920 × 1080/32; 1366 × 768; janela de VT; blank/DPMS; cursor; restauração |
| Áudio | síntese finita; pico ≤ 1,0; 8 vozes; sem device = silêncio; mute; shutdown < 250 ms; sem panic |
| Assets | fonte presente no manifesto; decode; cobertura de glifos; `--check` falha em divergência; execução em `/tmp` sem `assets/` |
| Entrada fb | setas/F1–F3/Esc; `Esc` isolado vs prefixo; aliases; sem key-up → auto-release/latch |
| Restauração | saída normal, erro, pânico, `Ctrl+C`, SIGTERM, SIGHUP; terminal em raw mode restaurado; `KD_TEXT` e cursor restaurados |

## 3. Portões

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

`cargo test` deve passar sem `/dev/fb0` e sem dispositivo de áudio.

Snapshots de TUI ficam em `tests/snapshots/` e usam `ratatui::backend::TestBackend`.
Para regenerar após uma mudança visual intencional:

```bash
UPDATE_SNAPSHOTS=1 cargo test --test tui_snapshots
```

Revise o diff antes de commitar. O teste PTY (`tests/pty.rs`) usa `script(1)`,
confirma a restauração do alternate screen, valida `Key-up: real` quando o
protocolo aprimorado é respondido e `Key-up: auto` com `TERM=linux`; se
`script` não existir, os testes são pulados.

## 4. Matriz manual

- [ ] TUI: xterm, foot, kitty, alacritty e console Linux; 80 × 24 e Wide.
- [ ] TUI com protocolo de teclado aprimorado: key-up real e sustain por
      segurar Espaço.
- [ ] TUI sem protocolo: auto-release em 750 ms e sustain latch.
- [ ] fb: 1920 × 1080, 32 bpp, driver genérico.
- [ ] fb: 24/16 bpp (ou registrar pendente).
- [ ] fb: 1366 × 768 e layout Compact.
- [ ] Fluxo completo em todos os modos: 24 notas, oitavas, volume, mudo.
- [ ] Idiomas, ajuda, pausa de entrada, saída.
- [ ] Restauração: menu (não existe), `Esc`, `Ctrl+C`, SIGTERM, pânico.
- [ ] VT real: `tty` = `/dev/ttyN`; `fb0/blank` chega a 0 sem `Ctrl+C`.
- [ ] Console previamente blanked/DPMS off.
- [x] Instalador com `PREFIX=/tmp/... SKIP_GROUP=1`.
- [ ] Áudio: dispositivo ALSA presente, ausente e ocupado.
- [ ] Sem ALSA (build `--no-default-features`) e com `--no-audio`.

**Não declare validado o que não foi executado.**

## 5. Validação registrada

| Data | Ambiente | O que foi validado | Pendências |
| --- | --- | --- | --- |
| 2026-10-31 | Linux, rustc 1.99.1, PTY via `script` | `fmt`, `clippy` e `test` com/sem `audio`; `xtask assets --check`; builds release; códigos de saída 1/2/3; nota `[C3]`, ajuda F1 e restauração do alternate screen | TUI em terminais reais, key-up real, resize, snapshots, fb, áudio, matriz manual completa |
| 2026-10-31 | Linux, `TestBackend` e PTY via `script` | Fase 2: 15 snapshots (80 × 24, 100 × 28, 120 × 36; pt/en; idle/pressed/glow/sustain/help/mudo/no-animations), teste PTY de desenho e restauração, protocolo aprimorado respondido (`Key-up: real`), portões completos com/sem `audio` | Revisão visual em terminais reais, key-up real por terminal, resize interativo, fb, áudio e matriz manual completa |
| 2026-10-31 | Linux, testes unitários e sequência determinística | Fase 3: release de 180/450 ms, fade de roubo de 5 ms, auto-release só no modo `auto`, sustain real adiando key-up, key-up processado com ajuda aberta, 20 mil operações invariantes sem voz duplicada/estouro; 97 unitários + integração; portões verdes com/sem `audio` | Key-up real por terminal e captura determinística (Fases 4/5) |
| 2026-10-31 | Linux, testes de síntese, xtask e snapshots | Fase 4: fonte Inter 4.1 vendorizada e hash verificado; 120 unitários + 5 CLI + 2 PTY + 15 snapshots; síntese aditiva sem NaN/pico ≤ 1, mute/fade, roubo limitado, lazy/shutdown; `NO_COLOR`/mono e contraste; cobertura de glifos; portões com/sem `audio` | Áudio audível em ALSA real, fonte no fb e matriz manual de terminais/VT seguem pendentes |
| 2026-10-31 | Linux, release TUI, PTY e prefixo temporário | Fase 5: portões com/sem `audio`; `xtask notices --check` (140 dependências) e `assets --check`; startup ~1,2 ms, frame TUI ~0,23 ms, RSS ~3,5 MB, binário ~1,5 MB; pacote `tar.gz` + `.sha256`; smoke em `/tmp` sem `assets/`; `install.sh --prefix /tmp/... --skip-group` | Áudio audível, terminais reais, key-up real, fb/VT, 24/16 bpp e captura PNG pendentes |
