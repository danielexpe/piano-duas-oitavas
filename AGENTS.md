# AGENTS.md — Memória operacional do projeto

> **Este arquivo é a memória persistente entre sessões de IA.**
> Leia-o inteiro no início de cada sessão antes de qualquer alteração.
> Atualize-o ao final de cada tarefa relevante.
>
> **Última atualização:** 2026-10-31
> **Status atual:** Fase 4 (arte, som, i18n e acessibilidade) concluída em
> 2026-10-31. A fonte Inter 4.1 está vendorizada com OFL e hash; o áudio ALSA
> é sintetizado, lazy e com shutdown com fade; acessibilidade cobre
> `NO_COLOR`, 256/16/mono e contraste. O app é um repositório próprio,
> independente do toolkit; o toolkit é apenas ferramenta de desenvolvimento e
> não é dependência de runtime. Captura determinística e fbdev continuam
> pendentes (Fases 5/6).
> **Branch:** main (repositório próprio)
> **Último commit de código:** `4494fbc` (Fase 4)
> **Alterações da Fase 4:** commitadas e enviadas em `4494fbc`
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
| Fase 4 — Arte/som/i18n | concluída em 2026-10-31; áudio sem validação audível em hardware |
| Especificações | `docs/01`–`08` preenchidos e congelados |
| Código | domínio puro, sessão, efeitos, input, app, CLI, i18n, piano em células, TUI, assets embutidos, síntese aditiva, ALSA dinâmico, paleta mono/256/16 |
| Testes | 120 unitários + 5 CLI + 2 PTY + 15 snapshots com `audio`; 119 unitários sem |
| Fonte | Inter 4.1 vendorizada em `assets/fonts/`; OFL 1.1; hash no manifesto |
| Assets | `assets/manifest.toml` + `cargo xtask assets --check` real |
| Áudio | implementado: 8 vozes, 48 kHz, ALSA via `libloading`, thread, canal 64, lazy, mute/volume, shutdown 120 ms; sem device = silêncio |
| Acessibilidade | `NO_COLOR`, `--color truecolor/256/16/never`, mono, contraste testado, estados com marcadores |
| Framebuffer | não iniciado; `--backend fb` retorna código 3 |
| Captura PNG | não iniciada; `--capture-frame` retorna código 1 |
| Release | não iniciado |
| Validação manual | PTY (`script`) e testes sem hardware; terminais/VT reais e áudio audível ainda pendentes |
| Git | repositório próprio; `origin` = `danielexpe/piano-duas-oitavas`; último commit de código `4494fbc` |

### Ponto de retomada (handoff 2026-10-31, Fase 4)

- **Repositório do app:** este diretório é um repositório Git próprio
  (`origin` = `github.com/danielexpe/piano-duas-oitavas`); o toolkit externo
  não faz parte do app.
- **Onde paramos:** Fase 4 concluída. A fonte Inter 4.1
  (`assets/fonts/Inter-Regular.ttf`, SHA-256
  `40d692fce188e4471e2b3cba937be967878f631ad3ebbbdcd587687c7ebe0c82`) e a
  licença OFL foram vendorizadas com aprovação do usuário. `xtask assets
  --check` valida manifesto, arquivo e licença. A síntese aditiva usa 8 vozes,
  fundamental + 4 harmônicos, envelope existente, fade de roubo/mute de 5 ms e
  limiter `tanh`; o backend ALSA é carregado dinamicamente em thread própria,
  com inicialização lazy na primeira nota e shutdown com fade de 120 ms. A
  paleta agora tem variantes truecolor/256/16/mono, `NO_COLOR` tem prioridade
  e os estados usam marcadores além de cor.
- **Estado do Git:** commit `4494fbc` (Fase 4) enviado para `origin/main`;
  este registro de docs vem em commit separado.
- **Validação já feita:** portões completos verdes (`fmt`, `clippy` com/sem
  features, `test` com/sem features, `xtask assets --check`, builds release).
  120 testes unitários + 5 CLI + 2 PTY + 15 snapshots (snapshots exigem a
  feature `audio`); sem a feature são 119 unitários. Síntese testada sem hardware; hashes de asset verificados;
  cobertura de glifos e contraste cobertos por teste.
- **Pendente de validação:** ouvir áudio em ALSA real, key-up real em
  terminais reais, TUI em xterm/foot/kitty/alacritty/console, resize
  interativo, revisão visual das cores, fonte no fb (Fase 6), captura
  determinística (Fase 5) e matriz manual de `docs/05`.
- **Primeira ação ao voltar:** ler este arquivo, rodar
  `git status --short --branch`, conferir `docs/06-roadmap.md` e iniciar a
  Fase 5 (release desktop).

### Próxima tarefa recomendada

Iniciar a **Fase 5 — Release desktop**: conferir licenças/`CREDITS`/
`THIRD_PARTY_NOTICES` sem drift, medir startup, frame, RSS e tamanho do
binário, gerar pacote `tar.gz` + `.sha256` e fazer smoke em `/tmp` sem a pasta
`assets/`. Não iniciar fbdev antes do pacote desktop estar validado.

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
cargo test
UPDATE_SNAPSHOTS=1 cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --no-default-features
cargo build --release
cargo build --release --no-default-features
cargo xtask assets --check
```

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
| 2026-10-31 | Commit e push da Fase 4 | Commit `4494fbc` (`feat: conclui Fase 4 com fonte, áudio e acessibilidade`) enviado para `origin/main`; GitHub reporta 1 alerta Dependabot low para triagem | `AGENTS.md` |

## 11. Fases

| Fase | Estado | Próximo marco |
| --- | --- | --- |
| 0 — Planejamento | concluída em 2026-10-31 | — |
| 1 — Fundação | concluída em 2026-10-31 | Núcleo testado + TUI placeholder |
| 2 — TUI funcional | concluída em 2026-10-31; commit `c965c16` | Fluxo completo + snapshots |
| 3 — Lógica rica | concluída em 2026-10-31 | Invariantes e fallback de key-up |
| 4 — Arte/som/i18n | concluída em 2026-10-31 | Fonte + áudio + acessibilidade |
| 5 — Release desktop | pendente | Pacote validado |
| 6 — Framebuffer | pendente | Console validado |
| 7 — Backlog | pendente | Itens priorizados |

## 12. Notas para a próxima sessão

- Handoff detalhado na seção 2 ("Ponto de retomada").
- Comece pela Fase 5; não pule para fbdev.
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
- A validação manual em terminais reais e VT continua pendente; não declare
  release validado sem rodar a matriz de `docs/05`.
- Não faça commit/push sem pedido explícito do usuário.
