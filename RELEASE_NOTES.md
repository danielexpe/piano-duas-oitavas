# Piano Duas Oitavas 0.1.0

Pré-release desktop (TUI) do Piano Duas Oitavas. O backend framebuffer e a
captura PNG entram na Fase 6; esta versão valida o fluxo de terminal, o áudio
sintetizado e o empacotamento.

## Destaques

- Piano de duas oitavas (24 notas) com teclas brancas e pretas.
- Mapeamento ABNT2 visível no piano e em uma representação do teclado físico.
- Efeitos visuais sincronizados nas duas representações.
- Síntese de piano polifônica (8 vozes) em runtime, com feature `audio`,
  inicialização lazy, mute, volume e shutdown com fade.
- Sustain, mudança de oitava, volume, mudo, ajuda e i18n pt-BR/en.
- Fonte Inter 4.1 embutida (OFL 1.1), verificada por manifesto e hash.
- Acessibilidade: `NO_COLOR`, variantes truecolor/256/16/mono, contraste
  mínimo e estados com marcadores além de cor.
- TUI offline, sem rede em runtime e sem assets externos.

## Validado

- Portões automatizados verdes com e sem a feature `audio`.
- 120 testes unitários + 5 CLI + 2 PTY + 15 snapshots.
- `cargo xtask assets --check` e `cargo xtask notices --check`.
- Medições locais: startup até a primeira tela ~1,2 ms; frame TUI ~0,23 ms
  ativo/ocioso; RSS ~3,5 MB; binário release ~1,5 MB.
- Pacote `tar.gz` + `.sha256`; smoke em `/tmp` sem a pasta `assets/`.
- Instalação testada com prefixo temporário e `--skip-group`.

## Pendente de validação manual

- Áudio audível em dispositivo ALSA real (com device, ausente e ocupado).
- TUI em xterm, foot, kitty, alacritty e console Linux; key-up real.
- Backend framebuffer, fonte no fb, 24/16 bpp e VT real (Fase 6).
- Captura determinística PNG (Fase 6).

## Limitações

- `--backend auto` usa TUI; o framebuffer é sempre explícito.
- Sem DRM/KMS nesta versão.
- Key-up real depende do terminal; sem ele, auto-release de 750 ms e sustain
  latch.
- Sem persistência de preferências, mouse, MIDI, gravação ou metrônomo.
- Sem vsync: tearing leve é aceitável.
- `lru` 0.12 foi eliminado ao atualizar `ratatui` para 0.30.2; o alerta
  Dependabot low correspondente não se aplica mais a esta árvore.

## Instalação

```bash
tar -xzf piano-duas-oitavas-0.1.0-x86_64-unknown-linux-gnu.tar.gz
cd piano-duas-oitavas-0.1.0-x86_64-unknown-linux-gnu
./install.sh --prefix "$HOME/.local"
```

Para testar sem alterar o grupo `video`:

```bash
./install.sh --prefix /tmp/piano-prefix --skip-group
```

> Não inclua o hash do próprio pacote nestas notas: ele muda quando as notas
> mudam. Registre o hash no `AGENTS.md` e na página da release.
