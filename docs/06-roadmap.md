# 06 — Roadmap

| Fase | Objetivo | Critério de saída | Estado |
| --- | --- | --- | --- |
| 0 — Descoberta | escopo, regras, telas, hardware, licenças | specs aprovadas | concluída em 2026-10-31 |
| 1 — Fundação | crate, núcleo puro, i18n, guards, CLI | testes de domínio + TUI placeholder | concluída em 2026-10-31 |
| 2 — TUI funcional | piano completo, input, efeitos, ajuda | fluxo completo + snapshots | concluída em 2026-10-31; commit `c965c16` |
| 3 — Lógica rica | vozes, sustain, auto-release, key-up | invariantes e fallback testados | concluída em 2026-10-31; captura determinística adiada para as Fases 4/5 |
| 4 — Arte/som/i18n | fonte, áudio, acessibilidade | áudio + fonte + i18n validados | concluída em 2026-10-31; áudio sem validação em hardware |
| 5 — Release desktop | licenças, docs, medições, pacote | release validado | concluída em 2026-10-31; validação manual pendente |
| 6 — Framebuffer | fbdev, texto, scaling, sinais | console validado + pacote | pendente |
| 7 — Backlog | evdev, persistência, gravação, MIDI, temas | priorizado pelo usuário | pendente |

## Checklist por fase

### Fase 0

- [x] `docs/01`–`08` preenchidos e congelados
- [x] `AGENTS.md` com decisões congeladas
- [x] Produto, telas, mapeamento, áudio, idiomas e licenças definidos
- [ ] Hardware real confirmado (1920 × 1080/32 bpp é a referência; grupo
      `video`, VT e driver serão confirmados na Fase 6)

### Fase 1 — Fundação

- [x] Crate e módulos criados (`core`, `state`, `app`, `input`, `render`, `platform`, `util`)
- [x] `core/note.rs`, `core/keymap.rs`, `core/voice.rs`, `core/envelope.rs` puros
- [x] Testes de domínio: notas, 24 teclas únicas, oitavas, vozes, envelope
- [x] Catálogo i18n pt/en com todas as chaves
- [x] CLI completa e validação de faixas (`--octave`, `--volume`)
- [x] Guardas de terminal e restauração em pânico/sinais
- [x] Placeholder TUI com HUD e piano desenhado em texto
- [x] Portões verdes com e sem `audio`

> Fase 1 concluída em 2026-10-31. Validação manual em terminais reais ainda
> pendente; o smoke foi feito em PTY com `script`. Efeitos, snapshots,
> breakpoints completos e key-up real pertencem à Fase 2.

### Fase 2 — TUI funcional

- [x] Render do piano de duas oitavas e do teclado físico
- [x] Rótulos de nota e de tecla física em todas as teclas
- [x] Input completo, `Repeat` ignorado, conflitos resolvidos
- [x] Efeitos: pressed, glow, nota flutuante, sustain
- [x] Ajuda modal e alternância de idioma/mudo/volume/oitava
- [x] Breakpoints Wide/Compact/Minimal e resize
- [x] Snapshots de TUI e teste PTY de restauração
- [x] `--no-animations`

> Fase 2 concluída em 2026-10-31; commit `c965c16`. Validação automatizada verde;
> terminais reais e revisão visual em hardware continuam pendentes (matriz de
> `docs/05`).

### Fase 3 — Lógica rica

- [x] Alocação de 8 vozes, roubo, retrigger e fade
- [x] Auto-release de 750 ms e cancelamento correto
- [x] Key-up real (protocolo aprimorado) e fallback latch
- [x] Invariantes: sem notas presas, sem vozes duplicadas, sem estouro
- [x] Testes de tempo com relógio injetado
- [ ] `--seed` sem efeito musical; captura determinística
      (`--seed` continua aceito e sem efeito musical; a captura PNG chega nas
      Fases 4/5)

> Fase 3 concluída em 2026-10-31 para o critério de saída (invariantes e
> fallback testados). O ciclo de vozes agora tem release de 180 ms, release de
> sustain de 450 ms, fade de roubo de 5 ms e auto-release apenas no modo
> `auto`; no modo `real`, o sustain adia o release até soltar Espaço. A
> captura determinística segue pendente por depender do pipeline de assets.

### Fase 4 — Arte, som, i18n e acessibilidade

- [x] Fonte Inter vendorizada com OFL e aprovação de download
- [x] `assets/manifest.toml` e `xtask assets --check`
- [x] Síntese ALSA polifônica e testes sem hardware
- [x] Áudio lazy, mute, volume, shutdown limpo
- [x] Contraste, mono/`NO_COLOR`, estados sem depender de cor
- [x] Catálogo completo e teste de strings longas
- [x] Cobertura de glifos

> Fase 4 concluída em 2026-10-31. A fonte Inter 4.1 foi vendorizada com OFL e
> hash verificado; `xtask assets --check` falha em divergência. O áudio usa
> síntese aditiva de 8 vozes, ALSA dinâmico, thread própria, canal limitado,
> inicialização lazy e shutdown com fade de 120 ms. A validação foi feita por
> testes de síntese e portões sem hardware; ouvir em ALSA real continua
> pendente. Acessibilidade cobre `NO_COLOR`, 256/16/mono, contraste mínimo e
> estados com marcadores não-cromáticos.

### Fase 5 — Release desktop

- [x] Portões verdes com e sem features
- [x] Licenças, `CREDITS`, `THIRD_PARTY_NOTICES` sem drift
- [x] Medições de startup, frame, RSS e tamanho
- [x] Pacote `tar.gz` + `.sha256`
- [x] Smoke em `/tmp` sem `assets/`

> Fase 5 concluída em 2026-10-31 para o pacote desktop TUI. O manifesto de
> assets e a tabela de dependências têm portões (`xtask assets --check` e
> `xtask notices --check`). Medições locais: startup ~1,2 ms, frame TUI
> ~0,23 ms, RSS ~3,5 MB, binário ~1,5 MB. Pacote e instalação com prefixo
> temporário testados. Validação manual de áudio, terminais reais e fb
> continua pendente; fb e captura PNG pertencem à Fase 6.

### Fase 6 — Framebuffer

- [ ] fbdev aberto/validado (32/24/16 bpp)
- [ ] `KD_GRAPHICS` + `FBIOBLANK(FB_BLANK_UNBLANK)`
- [ ] Pan/`yoffset` e fallback
- [ ] Texto, scaling e layout Compact
- [ ] Entrada raw de `/dev/tty` com `Esc` isolado
- [ ] Sinais e panic hook
- [ ] Validação em VT real, console blanked e 1080p/32 bpp
- [ ] `testes_debug.sh` executado no VT e anexado

### Fase 7 — Backlog

- [ ] Key-up real no console via evdev (grupo `input`, descoberta)
- [ ] Persistência XDG de volume/idioma/oitava
- [ ] Gravação/reprodução de performance
- [ ] Metrônomo e transposição
- [ ] MIDI de entrada/saída
- [ ] Remapeamento de teclas pela UI
- [ ] Temas e DRM/KMS
