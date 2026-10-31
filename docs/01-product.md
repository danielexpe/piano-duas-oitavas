# 01 — Produto: Piano Duas Oitavas

> Documento congelado na Fase 0 (2026-10-31). É a autoridade funcional.

## 1. Visão

- **Nome / marca:** Piano Duas Oitavas
- **Subtítulo:** Duas oitavas, 24 teclas, um teclado.
- **Propósito / gênero:** instrumento musical de prática e exploração, sem
  partitura e sem mouse, para tocar, ouvir e visualizar notas diretamente no
  terminal ou no console Linux.
- **Público:** pessoas que usam Linux em terminal/console, músicos iniciantes e
  curiosos que querem um piano imediato, offline e sem instalação pesada.
- **Diferenciais:**
  - duas oitavas completas (24 notas) visíveis ao mesmo tempo;
  - cada tecla do piano mostra a tecla física que a aciona e vice-versa;
  - som de piano sintetizado em runtime, polifônico (8 vozes), sem samples;
  - efeito visual sincronizado nas duas representações de teclado;
  - um único binário com TUI e backend fbdev (`/dev/fb0`);
  - interface pt-BR/en, atalhos globais consistentes e fallback silencioso
    quando não há áudio.

## 2. Escopo

### Premissas

- Linux x86_64 local; Rust stable, edição 2021, MSRV 1.88.
- TUI em terminal local; SSH é melhor esforço.
- Console alvo com `/dev/fb0` (efifb, simpledrm, amdgpu, i915 etc.),
  1920 × 1080 a 32 bpp como referência, com layout adaptativo para outras
  resoluções e formatos.
- Offline em runtime; entrada apenas por teclado; fontes embutidas.
- Áudio ALSA opcional, sintetizado em runtime e desligável na compilação
  (feature `audio`) ou em runtime (`--no-audio`, `F3`).
- Sem persistência: volume, idioma, oitava e mudo voltam ao padrão a cada
  execução, salvo flags de CLI.

### Fora de escopo (v1)

- Windows, macOS, Wayland/X11 nativos, DRM/KMS.
- Mouse, gamepad, toque, MIDI de entrada/saída, gravação/reprodução.
- Metrônomo, transposição, afinador, sintetizador configurável.
- Remapeamento de teclas pela UI, múltiplos perfis, persistência em disco.
- Samples de piano, imagens raster, temas claros/escuros alternáveis.
- Acessibilidade por leitor de tela; acessibilidade visual e por teclado é
  obrigatória.

## 3. Fluxos e telas

| Tela | Objetivo | Entradas | Saídas |
| --- | --- | --- | --- |
| `Piano` | tela única; tocar, ver mapeamento e status | 24 teclas de nota, Espaço, ←/→, `+`/`=`, `-`/`_`, `F1`, `F2`, `F3`, `Esc`, `Ctrl+C` | notas e efeitos; abre `Help`; alterna idioma/mudo; sai |
| `Help` | ajuda modal sobreposta, sem perder o estado | `F1`, `Esc`, `F2`, `F3` | fecha e volta ao piano; `Esc` com ajuda fechada sai |

Não há menu inicial. O app abre direto no piano. A ajuda é uma sobreposição
modal: teclas de nota são ignoradas enquanto ela está aberta; `F2` e `F3`
continuam ativos.

## 4. Regras do domínio

### 4.1 Notas e oitavas

- Duas oitavas cromáticas, 24 notas: 14 brancas e 10 pretas.
- Oitava base padrão: **3**; exibe C3–B4 (MIDI 48–71).
- `←`/`→` deslocam a oitava base em ±1, com limites **1..=6**
  (C1–B2 até C6–B7, MIDI 24–107).
- Nome canônico da nota: letra + acidente + oitava, em ASCII
  (`C3`, `C#3`, `D3`, …, `B4`). O nome exibido não muda com o idioma; a ajuda
  pt-BR explica o solfejo (Dó, Ré, Mi, Fá, Sol, Lá, Si).
- Fórmula MIDI: `midi = 12 * (oitava + 1) + semitom`, com
  `C=0, C#=1, …, B=11`.

### 4.2 Mapeamento do teclado alfanumérico (ABNT2)

Cada nota tem exatamente uma tecla física principal. As teclas não listadas
não tocam nada. Letras são normalizadas para minúsculas; `CapsLock` não altera
o mapeamento.

| Oitava | Brancas | Pretas |
| --- | --- | --- |
| base (3) | `Z`=C, `X`=D, `C`=E, `V`=F, `B`=G, `N`=A, `M`=B | `S`=C#, `D`=D#, `G`=F#, `H`=G#, `J`=A# |
| base+1 (4) | `Q`=C, `W`=D, `E`=E, `R`=F, `T`=G, `Y`=A, `U`=B | `2`=C#, `3`=D#, `5`=F#, `6`=G#, `7`=A# |

Aliases aceitos para a fileira numérica quando o terminal entrega o caractere
deslocado: `@`→2, `#`→3, `%`→5, `&`→7. O caractere `¨` (Shift+6 em ABNT2) é
tecla morta em muitos terminais e não é confiável: use `6` sem Shift.
`Shift+6` pode não mapear. Teclas de nota com `Ctrl` ou `Alt` são ignoradas.

### 4.3 Execução das notas

- `Press` inicia a nota; `Repeat` (auto-repeat do terminal) é ignorado;
  `Release` encerra a nota **quando o backend entrega key-up**.
- Fallback sem key-up: a nota entra em release automaticamente **750 ms**
  após o ataque. Pressionar a mesma tecla de novo reataque e reinicia o timer.
- Sustentar (`Espaço`):
  - com key-up real, segurar Espaço adia o release de todas as notas até
    soltar Espaço;
  - no fallback, `Espaço` alterna o sustain (latch), porque a soltura não é
    observável. O estado aparece no HUD.
- Limite de **8 vozes simultâneas**. A nona nota toma a voz mais antiga que
  não esteja em release; se todas estiverem, toma a mais antiga. Roubo de voz
  aplica fade curto para evitar clique.
- Reatacar a mesma nota não cria voz duplicada: reinicia o envelope da voz
  existente.
- A mudança de oitava libera todas as vozes e o sustain, e cancela timers de
  auto-release, para não deixar notas presas em alturas antigas.
- Volume: padrão `0.32`, passo `0.05`, faixa `0.00..=1.00`.
  `+`/`=` aumenta; `-`/`_` diminui. O valor é exibido como porcentagem.
- Mudo (`F3`): silencia a saída imediatamente, sem parar as animações, o
  sustain ou o estado das vozes. `--no-audio` inicia mudo. Sem dispositivo
  ALSA, o app permanece funcional e sinaliza áudio indisponível no HUD.
- Ao sair, todas as vozes recebem release, o áudio é silenciado e o
  terminal/console é restaurado.

### 4.4 Estados e limites

- Ajuda aberta: notas ignoradas; `Esc`/`F1` fecham; `Esc` sem ajuda sai.
- Idioma (`F2`) e mudo (`F3`) são globais e não interrompem notas.
- Não há pontuação, fases, IA, empates ou aleatoriedade de jogo.
- O app nunca bloqueia o loop por mais de ~100 ms.

## 5. Controles

| Tecla | Ação |
| --- | --- |
| `Z X C V B N M` | brancas da oitava base |
| `S D G H J` | pretas da oitava base |
| `Q W E R T Y U` | brancas da oitava seguinte |
| `2 3 5 6 7` | pretas da oitava seguinte |
| `Espaço` | sustain (segurar, ou latch no fallback) |
| `←` / `→` | oitava base −1 / +1 |
| `+` ou `=` | volume +5% |
| `-` ou `_` | volume −5% |
| `F1` | abrir/fechar ajuda |
| `F2` | alternar pt-BR/en |
| `F3` | alternar mudo |
| `Esc` | fechar ajuda; se fechada, sair |
| `Ctrl+C` | sair |

`Q` e `M` são notas (Dó e Si); por isso **não** são atalhos de sair/mudo.
Não há atalho `Q` para sair nem `M` para mudo.

## 6. Critérios de aceite

- [ ] As 24 teclas de nota produzem a altura MIDI correta, com ataque visual
      simultâneo na tecla do piano e na tecla física correspondente.
- [ ] As duas oitavas são visíveis e rotuladas; a mudança de oitava respeita
      1..=6 e libera notas antigas.
- [ ] Sustain, volume e mudo se comportam como especificado, inclusive no
      fallback sem key-up.
- [ ] Ajuda modal, idiomas pt-BR/en e todos os atalhos funcionam sem conflito.
- [ ] A TUI funciona em 80 × 24 e em layout Wide; o fb funciona a
      1920 × 1080/32 bpp e degrada para resoluções menores.
- [ ] Sem ALSA ou com `--no-audio`, o app permanece utilizável e silencioso.
- [ ] Saída normal, erro, `Esc`, `Ctrl+C`, SIGTERM/SIGHUP e pânico restauram
      terminal/console.
- [ ] Nenhuma preferência persiste entre execuções.
- [ ] Portões automatizados verdes com e sem a feature `audio`.
