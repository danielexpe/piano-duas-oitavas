# AGENTS.md — Memória operacional do projeto

> **Este arquivo é a memória persistente entre sessões de IA.**
> Leia-o inteiro no início de cada sessão antes de qualquer alteração.
> Atualize-o ao final de cada tarefa relevante.
>
> **Última atualização:** 2026-10-31
> **Status atual:** Fase 2 (TUI funcional) concluída e commitada em 2026-10-31
> (`c965c16`). Piano e teclado físico completos, efeitos, breakpoints, snapshots
> e teste PTY implementados; áudio e fbdev ainda não.
> **Branch:** main (repositório raiz do toolkit)
> **Último commit do toolkit:** `3aec021`
> **Último commit do app:** `c965c16` (Fase 2, sem push)
> **Remote:** `origin` no repositório raiz; nenhum push do app

## 1. Retomada rápida

1. Leia este arquivo inteiro.
2. Rode `git status --short --branch` e `git log --oneline -5`.
3. Leia `README.md`, `framework.md` e os docs da fase atual.
4. Não sobrescreva trabalho existente; registre bloqueios em vez de inventar
   progresso.
5. Carregue a skill `rust-console-tui-framebuffer` antes de implementar.
6. Não implemente fora do escopo da fase; proponha ADR para mudanças
   estruturais.

## 2. Status atual

| Item | Estado |
| --- | --- |
| Fase 0 — Descoberta | concluída em 2026-10-31 |
| Fase 1 — Fundação | concluída em 2026-10-31 |
| Fase 2 — TUI funcional | concluída em 2026-10-31; commit `c965c16` |
| Especificações | `docs/01`–`08` preenchidos e congelados |
| Código | domínio puro, sessão, efeitos visuais, input, app, CLI, i18n, piano em células, teclado físico e TUI funcional |
| Testes | 88 unitários + 5 de CLI + 2 PTY + 15 snapshots; passam com e sem `audio` |
| Áudio | especificado; não implementado (feature compila) |
| Framebuffer | não iniciado; `--backend fb` retorna código 3 |
| Captura PNG | não iniciada; `--capture-frame` retorna código 1 |
| Release | não iniciado |
| Validação manual | PTY (`script`) com resposta ao protocolo aprimorado; terminais/VT reais ainda pendentes |
| Git | rastreado na `main` local; commits `0df048a` (Fase 1) e `c965c16` (Fase 2); sem push |

### Ponto de retomada (handoff 2026-10-31, pós-Fase 2)

- **Raiz do toolkit:** repositório que contém este diretório de app
- **Diretório do app:** `piano-duas-oitavas/`
- **Onde paramos:** Fase 2 concluída. A TUI desenha piano completo (14 brancas
  e 10 pretas sobrepostas), rótulos de nota e tecla física, teclado físico por
  breakpoint, nota flutuante, glow de release, barra de sustain, ajuda modal
  escurecida e HUD de key-up `real`/`auto`. `--no-animations` remove a nota
  flutuante e os efeitos animados.
- **Estado do Git:** Fase 1 commitada em `0df048a`; Fase 2 commitada em
  `c965c16` (`src/**`, `tests/**`, docs e este arquivo). Este registro de
  hash entra em um commit de documentação subsequente. Sem tag e sem push.
- **Validação já feita:** portões completos verdes (`fmt`, `clippy` com/sem
  features, `test` com/sem features, `xtask assets --check`, builds release).
  15 snapshots de TUI em 80 × 24, 100 × 28 e 120 × 36 cobrindo pt/en, idle,
  pressed, glow, sustain, ajuda, mudo e `--no-animations`. Teste PTY com
  `script(1)` valida desenho e restauração do alternate screen; o protocolo
  aprimorado é respondido no PTY e o HUD mostra `Key-up: real`.
- **Pendente de validação:** TUI em terminais reais (xterm, foot, kitty,
  alacritty e console), comportamento de key-up real por terminal, resize
  interativo e revisão visual das cores. Tudo manual, da matriz de `docs/05`.
- **Primeira ação ao voltar:** ler este arquivo, rodar
  `git status --short --branch`, conferir `docs/06-roadmap.md` e iniciar a
  Fase 3. Não reabrir decisões da seção 4.

### Próxima tarefa recomendada

Iniciar a **Fase 3 — Lógica rica**: alocação/roubo de 8 vozes com fade,
auto-release de 750 ms e cancelamento, key-up real e fallback latch,
invariantes (sem notas presas ou vozes duplicadas) e testes de tempo com
relógio injetado. Não implementar fbdev antes da lógica estar estável.

## 3. Resumo do produto

- **Nome:** Piano Duas Oitavas
- **Binário:** `piano-duas-oitavas`
- **Descrição:** piano de duas oitavas em TUI/fbdev com mapeamento do teclado
  alfanumérico, síntese polifônica e efeitos visuais nas duas representações
  de teclado.
- **Plataforma:** Linux console local; x86_64; MSRV 1.88.
- **Backends:** TUI (`ratatui`/`crossterm`) e fbdev (`/dev/fb0`).
- **Público:** usuários de terminal/console e músicos iniciantes.
- **Licença:** código MIT; arte original CC BY 4.0; fonte Inter OFL 1.1
  (a vendorizar na Fase 4).

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
| `framework.md` | Template arquitetural e de processo |

Ordem de precedência em conflito: este `AGENTS.md` para estado operacional;
`docs/` para especificação; skill/framework para forma.

## 6. Pendências e questões abertas

| ID | Pendência | Prazo | Observação |
| --- | --- | --- | --- |
| Q-001 | Vendorizar `Inter-Regular.ttf` + `OFL-Inter.txt`; download exige aprovação e registro de hash. | Fase 4 | bloqueia texto final no fb |
| Q-002 | Confirmar driver, VT, resolução, bpp e grupo `video` no hardware real. | Fase 6 | não declarar validado antes |
| Q-003 | Medir key-up por terminal (xterm, foot, kitty, alacritty, console) e registrar fallback observado. | Fases 2 e 6 | protocolo aprimorado implementado e validado em PTY; terminais reais ainda não medidos |
| Q-004 | Validar 24/16 bpp e resoluções menores; se indisponível, registrar pendência. | Fase 6 | alvo principal é 32 bpp |
| Q-005 | Testar aliases `@ # % &` e a tecla morta `¨` em ABNT2 real. | Fase 2 | ajustar só se necessário |

Não há questão aberta de produto. Hardware real, fonte e key-up são
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

## 11. Fases

| Fase | Estado | Próximo marco |
| --- | --- | --- |
| 0 — Planejamento | concluída em 2026-10-31 | — |
| 1 — Fundação | concluída em 2026-10-31 | Núcleo testado + TUI placeholder |
| 2 — TUI funcional | concluída em 2026-10-31; commit `c965c16` | Fluxo completo + snapshots |
| 3 — Lógica rica | pendente | Invariantes e fallback de key-up |
| 4 — Arte/som/i18n | pendente | Fonte + áudio + acessibilidade |
| 5 — Release desktop | pendente | Pacote validado |
| 6 — Framebuffer | pendente | Console validado |
| 7 — Backlog | pendente | Itens priorizados |

## 12. Notas para a próxima sessão

- Handoff detalhado na seção 2 ("Ponto de retomada").
- Comece pela Fase 3; não pule para fbdev.
- Reaproveite `core::*`, `state::Session`, `state::Effects`, `input::Action`,
  `app::App`, `render::scene::Scene`, `render::piano::PianoBoard` e
  `render::layout::Breakpoint`; não duplique regras nem geometria.
- `Q` e `M` são notas: não reintroduza atalhos `Q`/`M`.
- O protocolo aprimorado do crossterm é tentado após o primeiro frame; se
  suportado, o HUD mostra `real`. O fallback de 750 ms + latch continua ativo.
- A TUI já é funcional: piano em células, keycaps, efeitos e ajuda modal.
  `--color` e `--seed` continuam aceitos sem efeito no desenho; `--no-animations`
  já altera o desenho.
- Snapshots: rode `UPDATE_SNAPSHOTS=1 cargo test` após mudanças visuais
  intencionais e revise o diff antes de commitar.
- A fonte Inter ainda não existe no repositório; não invente o arquivo.
  A Fase 4 depende de aprovação de download (Q-001).
- Use `testes_debug.sh` no VT antes de alterar código de fbdev.
- A validação manual em terminais reais e VT continua pendente; não declare
  release validado sem rodar a matriz de `docs/05`.
- Não faça commit/push sem pedido explícito do usuário.
