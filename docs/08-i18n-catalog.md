# 08 — Catálogo i18n

## 1. Regras

- Todo texto visível vem do catálogo; nada hardcoded na UI.
- Idiomas: `pt` (padrão) e `en`.
- `F2` alterna o idioma em todas as telas, inclusive com a ajuda aberta.
- A marca "Piano Duas Oitavas" permanece igual nos dois idiomas.
- Nomes de nota são canônicos em ASCII (`C3`, `C#3`, `B4`) e **não** são
  traduzidos; a ajuda pt-BR explica o solfejo.
- Teclas físicas e rótulos de keycap (`Z`, `Q`, `2`) não são traduzidos.
- Testar strings longas no layout Compact/Minimal; linhas de ajuda ≤ 72
  caracteres.
- Placeholders usam `{base}`, `{next}`, `{pct}`, `{lang}`.

## 2. Chaves

| Chave | pt-BR | en | Limite |
| --- | --- | --- | --- |
| `app.title` | Piano Duas Oitavas | Piano Duas Oitavas | 24 |
| `app.subtitle` | Duas oitavas, 24 teclas, um teclado. | Two octaves, 24 keys, one keyboard. | 48 |
| `hud.octave` | Oitava {base} (C{base}–B{next}) | Octave {base} (C{base}–B{next}) | 28 |
| `hud.volume` | Vol {pct}% | Vol {pct}% | 12 |
| `hud.sound` | Som: {state} | Sound: {state} | 20 |
| `hud.muted` | MUDO | MUTED | 8 |
| `hud.audio_unavailable` | Sem áudio | No audio | 16 |
| `hud.sustain` | Sustain: {state} | Sustain: {state} | 20 |
| `hud.language` | Idioma: {lang} | Language: {lang} | 20 |
| `hud.keyup` | Key-up: {state} | Key-up: {state} | 20 |
| `state.on` | ligado | on | 8 |
| `state.off` | desligado | off | 10 |
| `state.latch` | latch | latch | 8 |
| `state.real` | real | real | 8 |
| `state.auto` | auto | auto | 8 |
| `lang.pt` | PT-BR | PT-BR | 8 |
| `lang.en` | EN | EN | 8 |
| `keyboard.title` | Teclado físico | Computer keyboard | 20 |
| `help.title` | Ajuda | Help | 8 |
| `help.close` | F1 ou Esc para fechar | F1 or Esc to close | 32 |
| `help.notes_title` | Notas e teclas | Notes and keys | 20 |
| `help.notes_white` | Brancas: Z X C V B N M / Q W E R T Y U | White: Z X C V B N M / Q W E R T Y U | 56 |
| `help.notes_black` | Pretas: S D G H J / 2 3 5 6 7 | Black: S D G H J / 2 3 5 6 7 | 40 |
| `help.solfeges` | Solfejo: Dó Ré Mi Fá Sol Lá Si | Letters: C D E F G A B | 40 |
| `help.controls_title` | Controles | Controls | 12 |
| `help.sustain` | Espaço: sustain (segurar; no fallback, alterna) | Space: sustain (hold; toggles in fallback) | 60 |
| `help.octave` | Esq./Dir.: muda a oitava base | Left/Right: shift base octave | 48 |
| `help.volume` | + ou -: volume em passos de 5% | + or -: volume in 5% steps | 48 |
| `help.mute` | F3: liga/desliga o som | F3: toggle sound | 32 |
| `help.language` | F2: português/inglês | F2: Portuguese/English | 32 |
| `help.help` | F1: abre/fecha esta ajuda | F1: open/close this help | 36 |
| `help.quit` | Esc: fecha a ajuda ou sai | Esc: close help or quit | 36 |
| `help.keyup_real` | Key-up real: soltar a tecla encerra a nota. | Real key-up: releasing a key ends the note. | 72 |
| `help.keyup_fallback` | Sem key-up: cada nota dura 750 ms; Espaço alterna o sustain. | No key-up: each note lasts 750 ms; Space toggles sustain. | 72 |
| `help.no_persistence` | Preferências não são salvas. | Preferences are not saved. | 40 |
| `footer.help` | F1 ajuda | F1 help | 12 |
| `footer.language` | F2 idioma | F2 language | 12 |
| `footer.mute` | F3 mudo | F3 mute | 12 |
| `footer.quit` | Esc sair | Esc quit | 12 |
| `error.no_tty` | Terminal interativo indisponível; use um terminal local ou --backend fb. | Interactive terminal unavailable; use a local terminal or --backend fb. | 80 |
| `error.fb_permission` | Sem permissão para {path}; entre no grupo video ou ajuste as permissões. | No permission for {path}; join the video group or fix permissions. | 80 |
| `error.fb_vt` | O backend fb exige um VT de texto local; tty atual: {tty}. | The fb backend requires a local text VT; current tty: {tty}. | 80 |
| `error.audio_unavailable` | Áudio indisponível; o piano continua sem som. | Audio unavailable; the piano continues silently. | 72 |

## 3. Cobertura de glifos

- Caracteres usados: ASCII imprimível, acentos do português
  `á é í ó ú ã õ ç â ê ô`, mais `#`, `+`, `-`, `/`, `%`, `(`, `)`, `[`, `]`.
- Não usar setas tipográficas, `♯`, emojis ou símbolos musicais no v1.
- Verificar a cobertura da Inter na Fase 4; se faltar um glifo, substituir por
  equivalente ASCII no catálogo, nunca no renderer.
- O hífen `–` (en dash) usado em `hud.octave` é o único sinal tipográfico
  especial; se a fonte não o tiver, trocar por `-` antes do release.
- A TUI depende da fonte do terminal; validar os acentos nos terminais da
  matriz manual.
