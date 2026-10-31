# AGENTS.md — Memória operacional do projeto

> **Este arquivo é a memória persistente entre sessões de IA.**
> Leia-o inteiro no início de cada sessão antes de qualquer alteração.
> Atualize-o ao final de cada tarefa relevante.
>
> **Última atualização:** 2026-10-31
> **Status atual:** Fases 0–5 concluídas; Fase 6 implementada em 2026-10-31
> (fbdev, renderer vetorial, scaling, entrada raw, captura PNG e áudio no VT
> com fallback de dispositivo). A validação manual em VT real, console blanked,
> 24/16 bpp e áudio audível no hardware continua pendente. O app é um repositório próprio, independente do toolkit.
> **Branch:** main (repositório próprio)
> **Último commit de código:** `2ab35d8` (Fase 6)
> **Alterações da Fase 6:** commitadas em `2ab35d8`; release `v0.2.0` publicada
> **Pacote desktop:** `dist/piano-duas-oitavas-0.2.0-x86_64-unknown-linux-gnu.tar.gz`
> (SHA-256 `6a82c05fb96a2bccc093ed8215391f783db34cfd9cce14958a58b686fd1b8f86`)
> **Remote:** `origin` = `github.com/danielexpe/piano-duas-oitavas`

## 1. Retomada rápida

1. Leia este arquivo inteiro.
2. Rode `git status --short --branch` e `git log --oneline -5`.
3. Leia `README.md` e os docs da fase atual. `framework.md` é referência de
   processo copiada do toolkit, não dependência de runtime do app.
4. Não sobrescreva trabalho existente; registre bloqueios em vez de inventar
   progresso.
5. Use a skill `rust-console-tui-framebuffer` do toolkit apenas como ferramenta
   de desenvolvimento; o app não importa nem executa nada do toolkit.
6. Não implemente fora do escopo da fase; proponha ADR para mudanças
   estruturais.

## 2. Status atual

| Item | Estado |
| --- | --- |
| Fase 0 — Descoberta | concluída em 2026-10-31 |
| Fase 1 — Fundação | concluída em 2026-10-31 |
| Fase 2 — TUI funcional | concluída em 2026-10-31; commit `c965c16` |
| Fase 3 — Lógica rica | concluída em 2026-10-31; commit `f6274a5` |
| Fase 4 — Arte/som/i18n | concluída em 2026-10-31; commit `4494fbc` |
| Fase 5 — Release desktop | concluída em 2026-10-31 para o pacote TUI; validação manual pendente |
| Fase 6 — Framebuffer | implementada em 2026-10-31; fluxo principal validado pelo usuário no VT; matriz manual completa pendente |
| Especificações | `docs/01`–`08` preenchidos e congelados |
| Código | TUI completa, domínio puro, assets embutidos, síntese aditiva, ALSA dinâmico, paleta mono/256/16, backend fbdev e captura PNG |
| Testes | 150 unitários (+2 benchmarks ignorados) + 5 CLI + 2 PTY + 15 snapshots com `audio`; 144 unitários sem `audio` |
| Dependências | `ratatui` 0.30.2 (remove `lru` 0.12 vulnerável); 140 dependências conferidas por `xtask notices --check` |
| Áudio | implementado com preparo de ambiente no VT, fallback `default`→hardware e `--audio-device`; validação audível no VT pendente |
| Assets | Inter 4.1 + OFL; `xtask assets --check` real |
| Pacote | `dist/...0.2.0...tar.gz` (1,1 MB) + `.sha256`; binário 2,4 MB; smoke em `/tmp` sem assets |
| Instalação | testada a partir do tarball e do diretório extraído com `--skip-group`; trap de cleanup corrigido para preservar o status |
| Medições | startup ~1,2 ms; frame TUI ~0,23 ms; RSS ~3,5 MB |
| Framebuffer | implementado: fbdev, `KD_GRAPHICS`, unblank, pan/fallback, texto, scaling, input raw e restauração; validação em VT pendente |
| Captura PNG | implementada: 1344 × 756 determinística, sem abrir VT/fb/áudio |
| Áudio no VT | `default`/PipeWire recuperado por `XDG_RUNTIME_DIR`/DBus; fallback para analógico/USB; debug em `PIANO_AUDIO_DEBUG=1` |
| Frame fb | ~17 ms ativo em 1920 × 1080 no release; ocioso estático não redesenha |
| Validação manual | fluxo principal no VT real (interface e áudio) aprovado pelo usuário; 24/16 bpp, console blanked, pan real, key-up real e matriz completa pendentes |
| Release | `v0.2.0` publicada em 2026-10-31: https://github.com/danielexpe/piano-duas-oitavas/releases/tag/v0.2.0 (pacote `6a82c05...b8f86`) |
| Git | `origin` = `danielexpe/piano-duas-oitavas`; último commit de código `2ab35d8`; tag `v0.2.0` |

### Ponto de retomada (handoff 2026-10-31, Fase 6)

- **Onde paramos:** Fase 6 implementada e commitada em `2ab35d8`; release `v0.2.0` publicada. O backend
  `--backend fb` abre e valida `/dev/fb0`, entra em `KD_GRAPHICS`, desbloqueia
  o console, honra `yoffset`/`ypanstep`, tenta pan e cai para o buffer visível,
  desenha a cena com `tiny-skia` + Inter embutida, lê `/dev/tty` em raw mode e
  restaura `KD_TEXT`/cursor em saída, erro, pânico e sinais. A captura
  `--capture-frame` grava PNG 1344 × 756 determinístico.
- **Testes:** 150 unitários com `audio` (144 sem), 5 CLI, 2 PTY e 15 snapshots;
  ABI 160/80, formatos 32/24/16 bpp, pan/fallback, parser de teclas, layout,
  texto, captura e candidatos/fallback de áudio. Portões `fmt`, `clippy`
  (com/sem features), testes e builds release verdes.
- **Hardware observado:** `/dev/fb0` = `amdgpu` (`amdgpu` DRM), 1920 × 1080,
  32 bpp, stride 7680, `pan: 0,0` (sem ypan), blank 0; usuário no grupo
  `video`. A sessão atual é Wayland/VT tty2; o app recusa `--backend fb` fora
  de VT com código 3.
- **Áudio no VT (2026-10-31):** o usuário relatou `Sem áudio` no VT real. A
  causa provável é o PCM `default` (PipeWire) falhar quando o shell do VT não
  herda `XDG_RUNTIME_DIR`/DBus, ou o headset USB retornar `EBUSY` (-16). O app
  agora preenche as variáveis ausentes a partir de `/run/user/<uid>` e, se
  necessário, cai para `sysdefault:CARD=Generic_1`/`plughw:2,0` (analógico);
  `--audio-device` permite escolher o PCM e `PIANO_AUDIO_DEBUG=1` loga a
  escolha/falhas. Simulação sem `XDG_RUNTIME_DIR` confirmou `default` após o
  preparo e o fallback analógico sem ele.
- **Medições:** frame fb ativo ~17 ms em 1920 × 1080 (alvo ≤16 ms); o loop
  compara a cena e não redesenha quando ela está estável, reduzindo o custo
  ocioso. A medição é de software, não de hardware real.
- **Release v0.2.0 (2026-10-31):** tag anotada `v0.2.0` no commit `2ab35d8`,
  assets `piano-duas-oitavas-0.2.0-x86_64-unknown-linux-gnu.tar.gz` (1,1 MB) e
  `.sha256` (`6a82c05fb96a2bccc093ed8215391f783db34cfd9cce14958a58b686fd1b8f86`).
  URL: https://github.com/danielexpe/piano-duas-oitavas/releases/tag/v0.2.0.
- **Validado em 2026-10-31:** o usuário aprovou o fluxo principal no VT real
  (interface gráfica e áudio) após a correção. O pacote 0.2.0 foi verificado
  com SHA-256, smoke em `/tmp`, captura PNG, erro de fb com código 3 e
  instalação em prefixo temporário a partir do tarball e do diretório extraído.
- **Pendente de validação:** 24/16 bpp, console previamente blanked/DPMS, pan
  real, key-up real, áudio em headset/analógico com device ocupado e ausente,
  e a matriz manual completa de `docs/05`. Anexar `testes_debug.txt` do
  `testes_debug.sh`.
- **Primeira ação ao voltar:** entrar em um VT de texto, rodar
  `./testes_debug.sh` e depois `./target/release/piano-duas-oitavas --backend
  fb`; registrar o que foi validado e o que permanece pendente.

### Próxima tarefa recomendada

Validar a **Fase 6 em VT real**: rodar `./testes_debug.sh` e o app com
`--backend fb` em `/dev/ttyN` (1920 × 1080/32 bpp), confirmar `KD_GRAPHICS`,
desbloqueio de console blanked, pan/fallback, texto, entrada e restauração.
O fluxo principal já foi aprovado pelo usuário; ainda faltam 24/16 bpp,
console blanked, pan real, key-up real, áudio com device ocupado/ausente e a
matriz completa de `docs/05`. Depois, propor a Fase 7 (evdev, persistência,
gravação, MIDI, temas).

## 3. Resumo do produto

- **Nome:** Piano Duas Oitavas
- **Binário:** `piano-duas-oitavas`
- **Descrição:** piano de duas oitavas em TUI/fbdev com mapeamento do teclado
  alfanumérico, síntese polifônica e efeitos visuais nas duas representações
  de teclado.
- **Plataforma:** Linux console local; x86_64; MSRV 1.88.
- **Backends:** TUI (`ratatui`/`crossterm`) e fbdev (`/dev/fb0`).
- **Público:** usuários de terminal/console e músicos iniciantes.
- **Licença:** código MIT; arte original CC BY 4.0; fonte Inter 4.1 OFL 1.1
  vendorizada.

## 4. Decisões congeladas (não reabrir sem o usuário)

- **Escopo:** duas oitavas cromáticas, 24 notas; uma tela `Piano` + ajuda
  modal. Sem menu, gravação, MIDI, metrônomo, transposição, remapeamento ou
  mouse no v1.
- **Oitavas:** base padrão 3 (C3–B4, MIDI 48–71); `←`/`→` deslocam entre
  1 e 6; mudar de oitava libera todas as vozes e o sustain.
- **Mapeamento ABNT2:** brancas `Z X C V B N M` (base) e `Q W E R T Y U`
  (base+1); pretas `S D G H J` (base) e `2 3 5 6 7` (base+1). Aliases
  `@ # % &`; `Shift+6`/`¨` não é confiável. Letras normalizadas; `Repeat`
  ignorado.
- **Atalhos:** `Espaço` sustain, `+`/`=` volume +5%, `-`/`_` volume −5%,
  `F1` ajuda, `F2` idioma, `F3` mudo, `Esc` fecha ajuda/sai, `Ctrl+C` sai.
  **`Q` e `M` são notas** e nunca atalhos de sair/mudo.
- **Áudio:** síntese aditiva em runtime, 8 vozes, ALSA dinâmico, feature
  `audio`, sem samples; ausência de device = silêncio; volume padrão 0,32;
  inicialização lazy; shutdown com fade ≤ 250 ms.
- **Key-up:** melhor esforço com protocolo de teclado aprimorado; fallback
  com auto-release de 750 ms e sustain latch; HUD indica `real`/`auto`.
  evdev no console fica para a Fase 7.
- **Telas/idiomas:** abertura direta no piano; ajuda modal; pt-BR padrão e en;
  toda string visível no catálogo.
- **Persistência:** nenhuma no v1; sem leitura/escrita de config.
- **Visual:** tema conservatório noturno, sem imagens raster; piano e keycaps
  vetoriais no fb e células na TUI; estados não dependem só de cor.
- **Hardware:** referência `/dev/fb0` 1920 × 1080/32 bpp, driver genérico;
  suportar 32/24/16 bpp e layout adaptativo; validação real pendente.
- **Determinismo:** sem RNG no comportamento; `--seed` reservado.
- **Licenças:** código MIT; arte original CC BY 4.0; fonte OFL 1.1.

## 5. Documentos e fontes de verdade

| Documento | Autoridade |
| --- | --- |
| `README.md` | Guia do usuário |
| `docs/01-product.md` | Regras, fluxos, aceite funcional |
| `docs/02-technical-spec.md` | Arquitetura, crates, CLI, desempenho |
| `docs/03-visual-design.md` | Paleta, tipografia, layout |
| `docs/04-assets-and-licenses.md` | Pipeline, manifesto, licenças |
| `docs/05-test-plan.md` | Testes e matriz manual |
| `docs/06-roadmap.md` | Fases e critérios de saída |
| `docs/07-decisions-and-risks.md` | ADRs, premissas, questões |
| `docs/08-i18n-catalog.md` | Strings e limites |
| `framework.md` | Referência de processo copiada do toolkit; não é dependência de runtime |

Ordem de precedência em conflito: este `AGENTS.md` para estado operacional;
`docs/` para especificação; skill/framework para forma.

## 6. Pendências e questões abertas

| ID | Pendência | Prazo | Observação |
| --- | --- | --- | --- |
| Q-001 | Vendorizar `Inter-Regular.ttf` + `OFL-Inter.txt`; download exige aprovação e registro de hash. | Fase 4 | resolvida em 2026-10-31: Inter 4.1 de `rsms/inter`, hashes no manifesto |
| Q-002 | Confirmar driver, VT, resolução, bpp e grupo `video` no hardware real. | Fase 6 | não declarar validado antes |
| Q-003 | Medir key-up por terminal (xterm, foot, kitty, alacritty, console) e registrar fallback observado. | Fases 2 e 6 | lógica real/auto e sustain adiado testada em PTY; terminais reais ainda não medidos |
| Q-004 | Validar 24/16 bpp e resoluções menores; se indisponível, registrar pendência. | Fase 6 | alvo principal é 32 bpp |
| Q-005 | Testar aliases `@ # % &` e a tecla morta `¨` em ABNT2 real. | Fase 2 | ajustar só se necessário |

Não há questão aberta de produto. Hardware real, áudio audível e key-up são
pendências de execução/validação, não de escopo.

## 7. Comandos e portões

```bash
cargo run
cargo run -- --backend fb
cargo run -- --backend fb --audio-device auto
cargo run -- --backend fb --audio-device sysdefault:CARD=Generic_1
PIANO_AUDIO_DEBUG=1 cargo run -- --backend fb
cargo run -- --capture-frame /tmp/piano.png
cargo test
UPDATE_SNAPSHOTS=1 cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --no-default-features
cargo build --release
cargo build --release --no-default-features
cargo xtask assets --check
cargo xtask notices --check
cargo test --release --lib render::fb::tests::frame_budget_benchmark -- --ignored --nocapture
```

> **Atenção:** o último build dos portões é `--no-default-features`, que
> sobrescreve `target/release/piano-duas-oitavas` **sem áudio**. Antes de
> testar som/fb, rode `cargo build --release` para religar a feature `audio`.

Diagnóstico de console (executar no VT afetado):

```bash
./testes_debug.sh
```

## 8. Regras de trabalho para o agente

1. Leia este arquivo antes de agir.
2. Não use rede em runtime; downloads exigem aprovação explícita.
3. Não faça commit, push, amend ou PR sem pedido explícito.
4. Não configure remote sem pedido.
5. Não armazene segredos neste arquivo nem no repositório.
6. Não reabra decisões congeladas; proponha um ADR.
7. Mantenha o escopo da fase.
8. Preserve terminal/console em saída, erro, pânico e sinais.
9. Mantenha `core/` puro e `unsafe` isolado em `platform/` com `// SAFETY:`.
10. Código/identificadores em inglês; UI nos idiomas do produto.
11. Sem comentários desnecessários; siga o estilo existente.
12. Teste antes de declarar concluído; rode os portões com e sem features.
13. Não invente validação manual; registre o que não foi executado.
14. Não altere o mapeamento ABNT2 nem os atalhos `F3`/`Esc` sem o usuário.
15. Atualize este arquivo ao final de cada tarefa.

## 9. Protocolo de atualização deste arquivo

Ao final de cada tarefa, atualize: data no topo, status, próxima tarefa,
registro de atividades, pendências e decisões. Seja conciso; use checkboxes;
registre hashes de commits e releases; não apague entradas antigas. Marque
claramente o que foi validado e o que permanece pendente.

## 10. Registro de atividades

| Data | Atividade | Resultado | Arquivos |
| --- | --- | --- | --- |
| 2026-10-31 | Projeto criado a partir do toolkit | Esqueleto e docs iniciais | `README.md`, `AGENTS.md`, `docs/`, `src/`, `Cargo.toml` |
| 2026-10-31 | Fase 0: escopo, mapeamento, áudio, telas, i18n, licenças e hardware congelados | `docs/01`–`08`, `README.md`, `AGENTS.md` preenchidos; nenhum código de produção | `docs/01`–`08`, `README.md`, `AGENTS.md`, `CREDITS.md`, `RELEASE_NOTES.md`, `THIRD_PARTY_NOTICES.md` |
| 2026-10-31 | Sessão encerrada para desligamento; handoff registrado | Ponto de retomada na seção 2; próximo passo é a Fase 1; nada commitado | `AGENTS.md` |
| 2026-10-31 | Fase 1: domínio puro, sessão, input, app, i18n, CLI, guardas e TUI placeholder | 65 testes unitários + 5 de integração; portões verdes com/sem `audio`; smoke em PTY; commit `0df048a` | `src/**`, `tests/cli.rs`, `AGENTS.md`, `docs/06-roadmap.md`, `docs/05-test-plan.md` |
| 2026-10-31 | Sessão encerrada para desligamento após a Fase 1 | Handoff atualizado com commits `0df048a`/`829ce8c`; commit de encerramento local; sem push | `AGENTS.md` |
| 2026-10-31 | Fase 2: TUI funcional, piano em células, efeitos, breakpoints, snapshots e PTY | 88 testes unitários + 5 CLI + 2 PTY + 15 snapshots; portões verdes com/sem `audio`; commit `c965c16` | `src/**`, `tests/tui_snapshots.rs`, `tests/pty.rs`, `tests/snapshots/**`, `docs/05`, `docs/06`, `README.md`, `AGENTS.md` |
| 2026-10-31 | Extração do app para repositório próprio | `git init`, commit `9071c6d`, remoto público `danielexpe/piano-duas-oitavas`; toolkit deixou de rastrear a pasta | `.git/`, `AGENTS.md` |
| 2026-10-31 | Fase 3: ciclo de vozes, key-up real, sustain adiado, invariantes e testes de tempo | 97 unitários; release/fade, auto-release só em `auto`, sequência determinística de 20 mil operações; portões completos verdes; commit `f6274a5` e push | `src/core/voice.rs`, `src/state/session.rs`, `src/app.rs`, `docs/05`, `docs/06`, `AGENTS.md` |
| 2026-10-31 | Fase 4: fonte Inter, manifesto/xtask, síntese ALSA, acessibilidade e glifos | 120 unitários + 5 CLI + 2 PTY + 15 snapshots; áudio lazy/mute/volume/shutdown; `NO_COLOR`/mono/contraste; hash de asset verificado; sem validação audível em hardware | `assets/**`, `xtask/**`, `src/assets.rs`, `src/audio/**`, `src/platform/alsa.rs`, `src/theme.rs`, `src/render/**`, `src/app.rs`, `src/i18n`, `docs/04`–`07`, `README.md`, `THIRD_PARTY_NOTICES.md`, `CREDITS.md`, `AGENTS.md` |
| 2026-10-31 | Fase 5: release desktop, dependências/licenças, medições, pacote e smoke | Pacote `tar.gz` (SHA-256 `6b67350a...39733d25`) + binário sem assets; `xtask notices --check` (140 deps); `ratatui` 0.30.2 remove `lru` vulnerável; startup ~1,2 ms, frame ~0,23 ms, RSS ~3,5 MB; instalação em prefixo temporário | `Cargo.toml`, `Cargo.lock`, `THIRD_PARTY_NOTICES.md`, `xtask/src/main.rs`, `install.sh`, `RELEASE_NOTES.md`, `README.md`, `docs/05`, `docs/06`, `AGENTS.md` |
| 2026-10-31 | Commit e push da Fase 5 | Commit `bf77ae9` (`feat: conclui Fase 5 com pacote desktop e auditoria de dependências`) enviado para `origin/main`; pacote em `dist/` não versionado | `AGENTS.md` |
| 2026-10-31 | Commit e push da Fase 4 | Commit `4494fbc` (`feat: conclui Fase 4 com fonte, áudio e acessibilidade`) enviado para `origin/main`; GitHub reporta 1 alerta Dependabot low para triagem | `AGENTS.md` |
| 2026-10-31 | Fase 6: fbdev, renderer vetorial, scaling, input raw, sinais e captura PNG | `platform/fb.rs`, `platform/input.rs`, `render/fb.rs`, `render/text.rs` e `capture.rs`; ABI 160/80, 32/24/16 bpp, pan/fallback, parser `Esc`/setas/F1–F3, layout Wide/Compact/Minimal, PNG 1344×756 determinístico; portões com/sem `audio` verdes; frame fb ~17 ms em 1920×1080; validação em VT pendente | `src/platform/**`, `src/render/**`, `src/capture.rs`, `src/main.rs`, `src/error.rs`, `tests/cli.rs`, `README.md`, `docs/02`, `docs/03`, `docs/05`, `docs/06`, `AGENTS.md` |
| 2026-10-31 | Correção de áudio no VT: ambiente PipeWire/Pulse e fallback ALSA | `prepare_environment` preenche `XDG_RUNTIME_DIR`/`PIPEWIRE_RUNTIME_DIR`/DBus/`PULSE_SERVER`; candidatos `default`→`pipewire`→`pulse`→hardware (não-HDMI primeiro); `--audio-device` e `PIANO_AUDIO_DEBUG=1`; escrita com recuperação de xrun/short writes; 5 testes de candidatos | `src/platform/alsa.rs`, `src/audio/mod.rs`, `src/app.rs`, `src/cli.rs`, `src/main.rs`, `README.md`, `docs/02`, `docs/05`, `docs/06`, `docs/07`, `AGENTS.md` |
| 2026-10-31 | Release 0.2.0: versão, notas, pacote e instalador | `version = 0.2.0`, `RELEASE_NOTES.md` reescrito, tarball 1,1 MB + `.sha256` (`6a82c05...b8f86`), binário 2,4 MB; smoke, captura, erro fb e instalação em `/tmp`; `install.sh` agora retorna o status correto ao instalar de diretório extraído | `Cargo.toml`, `Cargo.lock`, `install.sh`, `RELEASE_NOTES.md`, `README.md`, `AGENTS.md`, `docs/05`, `docs/06` |
| 2026-10-31 | Commit e push da Fase 6 | Commit `2ab35d8` (`feat: conclui Fase 6 com fbdev, captura PNG e áudio no VT`) enviado para `origin/main`; 28 arquivos, 3035 inserções; pacote em `dist/` não versionado | `AGENTS.md` |
| 2026-10-31 | Release `v0.2.0` | Tag anotada no commit `2ab35d8`; GitHub Release com título `Piano Duas Oitavas 0.2.0`, tarball 1,1 MB e `.sha256`; URL https://github.com/danielexpe/piano-duas-oitavas/releases/tag/v0.2.0 | `AGENTS.md` |

## 11. Fases

| Fase | Estado | Próximo marco |
| --- | --- | --- |
| 0 — Planejamento | concluída em 2026-10-31 | — |
| 1 — Fundação | concluída em 2026-10-31 | Núcleo testado + TUI placeholder |
| 2 — TUI funcional | concluída em 2026-10-31; commit `c965c16` | Fluxo completo + snapshots |
| 3 — Lógica rica | concluída em 2026-10-31 | Invariantes e fallback de key-up |
| 4 — Arte/som/i18n | concluída em 2026-10-31 | Fonte + áudio + acessibilidade |
| 5 — Release desktop | concluída em 2026-10-31; validação manual pendente | Pacote e smoke validados |
| 6 — Framebuffer | implementada em 2026-10-31; release v0.2.0 publicada; matriz manual completa pendente | 24/16 bpp, console blanked, pan real e `testes_debug.sh` |
| 7 — Backlog | pendente | Itens priorizados |

## 12. Notas para a próxima sessão

- Handoff detalhado na seção 2 ("Ponto de retomada").
- Valide a Fase 6 em VT real; não pule a validação manual pendente da Fase 5.
- Reaproveite `core::*`, `state::Session`, `state::Effects`, `input::Action`,
  `app::App`, `render::scene::Scene`, `render::piano::PianoBoard` e
  `render::layout::Breakpoint`; não duplique regras nem geometria.
- `Q` e `M` são notas: não reintroduza atalhos `Q`/`M`.
- O protocolo aprimorado do crossterm é tentado após o primeiro frame; se
  suportado, o HUD mostra `real`. No modo `auto`, o fallback de 750 ms + latch
  continua ativo; no modo `real`, `NoteOff` encerra e o sustain adia releases
  até soltar Espaço.
- A TUI já é funcional: piano em células, keycaps, efeitos e ajuda modal.
  `--color` agora controla truecolor/256/16/never e `NO_COLOR` tem prioridade;
  `--seed` continua reservado e `--no-animations` já altera o desenho.
- Snapshots: rode `UPDATE_SNAPSHOTS=1 cargo test` após mudanças visuais
  intencionais e revise o diff antes de commitar.
- A fonte Inter 4.1 está vendorizada e verificada por
  `cargo xtask assets --check`; não altere o TTF nem o manifesto sem
  atualizar hashes, licença e `CREDITS`.
- O áudio só abre ALSA na primeira nota e nunca deve bloquear o loop; sem
  device o app continua silencioso e o HUD sinaliza `Sem áudio`.
- Use `testes_debug.sh` no VT antes de alterar código de fbdev.
- O pacote desktop fica em `dist/` (ignorado pelo Git) e deve ser regenerado
  após qualquer mudança de código ou docs; o hash muda e não deve entrar nas
  notas de release. O hash atual em `AGENTS.md` é
  `6a82c05fb96a2bccc093ed8215391f783db34cfd9cce14958a58b686fd1b8f86`.
- A validação manual em terminais reais e VT continua pendente; não declare
  release validado sem rodar a matriz de `docs/05`.
- O backend fb usa o canvas lógico de 1344 × 756 e o escala com bilinear para
  até 96% do dispositivo; o fundo preenche a tela. Quando a cena não muda, o
  loop não redesenha. A captura PNG sempre usa a cena Wide em 1344 × 756.
- Fora de um VT de texto, `--backend fb` retorna código 3 com mensagem
  acionável; isso é esperado em X/Wayland/SSH e nos testes automatizados.
- `platform/fb.rs` isola todo o `unsafe` de fbdev/console com `// SAFETY:`;
  mantenha ioctls, mmap e restauração ali, não em `render/` ou `app/`.
- O áudio abre `default` e, se falhar, tenta PipeWire/Pulse e hardware
  (analógico/USB antes de HDMI). Use `--audio-device` para forçar um PCM e
  `PIANO_AUDIO_DEBUG=1` para ver a escolha/falhas no stderr. O headset USB pode
  retornar `EBUSY` quando o PipeWire o detém; o fallback analógico é esperado.
- Não faça commit/push sem pedido explícito do usuário.
