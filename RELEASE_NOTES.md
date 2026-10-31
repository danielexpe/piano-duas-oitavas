# Piano Duas Oitavas 0.1.0

Primeira versão de desenvolvimento do Piano Duas Oitavas. Este arquivo descreve
o alvo do release; as fases iniciais ainda não implementam nem validam o
produto completo.

## Destaques

- Piano de duas oitavas (24 notas) com teclas brancas e pretas.
- Mapeamento ABNT2 visível no piano e em uma representação do teclado físico.
- Efeitos visuais sincronizados nas duas representações.
- Síntese de piano polifônica (8 vozes) em runtime, com feature `audio`.
- Sustain, mudança de oitava, volume, mudo, ajuda e i18n pt-BR/en.
- TUI e backend fbdev (`/dev/fb0`), offline e sem assets de imagem.

## Validado

- Portões automatizados verdes com e sem a feature `audio` (Fase 1).
- Smoke da TUI em PTY com `script`: nota, ajuda e restauração do terminal.
- TUI em terminais reais: pendente.
- Console em VT real, resolução e bpp: pendente.
- Áudio em hardware ALSA: pendente.

## Limitações

- `--backend auto` usa TUI; o framebuffer é sempre explícito.
- Sem DRM/KMS nesta versão.
- 24/16 bpp podem estar pendentes de validação em hardware.
- Key-up real depende do terminal; sem ele, auto-release de 750 ms e sustain
  latch.
- Sem persistência de preferências, mouse, MIDI, gravação ou metrônomo.
- Sem vsync: tearing leve é aceitável.

## Instalação

```bash
tar -xzf piano-duas-oitavas-0.1.0-x86_64-unknown-linux-gnu.tar.gz
cd piano-duas-oitavas-0.1.0-x86_64-unknown-linux-gnu
./install.sh --prefix "$HOME/.local"
```

> Não inclua o hash do próprio pacote nestas notas: ele muda quando as notas
> mudam. Registre o hash no `AGENTS.md` e na página da release.
