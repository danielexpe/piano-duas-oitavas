# 03 — Design visual

> Congelado na Fase 0. Tema "conservatório noturno": madeira escura, marfim e
> âmbar, sem neon nem estética de CRT.

## 1. Direção

- **Tema:** conservatório noturno; palco escuro, teclas de marfim e ébano,
  luz âmbar quente.
- **Sensação:** acústico, sóbrio, artesanal, legível a distância no console.
- **Referências:** salas de concerto, pianos verticais, partituras impressas,
  madeira e latão envelhecidos.
- **Evitar:** neon, gradientes saturados, estética CRT/arcade, emojis,
  partículas coloridas, ícones exóticos e excesso de texto piscando.

## 2. Canvas e layout

- Canvas lógico: **1344 × 756** (70% de 1920 × 1080).
- Fb: fundo ocupa a tela inteira; composição escalada e centralizada com
  bilinear; nunca nearest para texto.
- TUI: breakpoints Wide / Compact / Minimal; mínimo 80 × 24.
- Resize preserva todo o estado.

| Breakpoint | Condição | Layout |
| --- | --- | --- |
| Wide | ≥ 110 × 30 | piano completo + teclado físico completo + HUD + rodapé |
| Compact | 90 × 26 a 109 × 29 | piano + teclado físico abreviado + HUD + rodapé curto |
| Minimal | 80 × 24 a 89 × 25 | piano com rótulos de tecla + uma fileira compacta de keycaps + HUD mínimo |

No fb, a cena Wide é usada quando a tela tem ≥ 1280 × 720; Compact em
≥ 1024 × 600; abaixo disso, Minimal. O canvas lógico (1344 × 756 em Wide) é
escalado com bilinear para até 96% do dispositivo e centralizado; o fundo
preenche a tela inteira. O caminho rápido é usado quando a escala fica próxima
de 1. O piano nunca é cortado; o teclado físico pode ser abreviado.

### 2.1 Geometria lógica (Wide)

| Faixa | Y | Conteúdo |
| --- | --- | --- |
| Cabeçalho | 24–92 | título, oitava, volume, mudo, idioma |
| Nota flutuante | 100–146 | nome da última nota por 600 ms |
| Piano | 150–570 | 14 brancas (~89 px), 10 pretas (~56 × 260 px) |
| Status | 580–606 | sustain, áudio, modo de key-up |
| Teclado físico | 610–720 | 4 fileiras de keycaps (~64 × 22 px) |
| Rodapé | 724–744 | dicas de atalhos |

O piano ocupa a largura útil (1248 px, margens de 48 px). As pretas ficam nas
fronteiras C-D, D-E, F-G, G-A e A-B de cada oitava, com sombra projetada e
altura 62% da branca. Cada branca mostra a letra da nota e a tecla física na
frente; cada preta mostra a tecla física em uma etiqueta clara na base.

### 2.2 Teclado físico na tela

Quatro fileiras com contexto suficiente para o usuário localizar a tecla:

```text
1 2 3 4 5 6 7 8 9 0
Q W E R T Y U I O P
A S D F G H J K L
Z X C V B N M
```

- Teclas mapeadas: borda âmbar, rótulo da nota em tamanho pequeno, brilho
  normal.
- Teclas não mapeadas: 35% de opacidade, sem rótulo de nota.
- Ao pressionar, o keycap afunda 2 px, ganha preenchimento âmbar e projeta um
  halo curto; a tecla do piano correspondente faz o mesmo.
- No breakpoint Minimal, mostrar apenas a fileira `Z X C V B N M` + as pretas
  `S D G H J` em uma faixa compacta acima do piano; a tabela completa fica na
  ajuda.

## 3. Paleta (tokens semânticos)

| Token | Truecolor | Uso |
| --- | --- | --- |
| `surface` | `#12100e` | fundo da tela |
| `panel` | `#1c1916` | painéis, HUD, ajuda |
| `surface_raised` | `#26211c` | keycaps não mapeadas |
| `ivory` | `#f4ead8` | teclas brancas |
| `ivory_pressed` | `#ffd79a` | branca pressionada |
| `ivory_edge` | `#c9b99f` | borda/sombra da branca |
| `ebony` | `#171412` | teclas pretas |
| `ebony_pressed` | `#b8792a` | preta pressionada |
| `ebony_edge` | `#3a3028` | borda da preta |
| `accent` | `#e0a24a` | destaque, halo, foco |
| `accent_soft` | `#8a5a24` | bordas e estados secundários |
| `text_primary` | `#f4ead8` | texto principal |
| `text_secondary` | `#b8a88f` | texto secundário |
| `text_on_light` | `#1c1916` | texto sobre marfim/âmbar |
| `danger` | `#d45a4a` | erro, áudio indisponível |
| `success` | `#7fb069` | áudio ativo, sustain ligado |

- Variantes: truecolor, 256, 16 e mono. Em mono, pressed = invertido +
  deslocamento; sustain = sublinhado; mudo = `[M]` textual.
- Contraste mínimo: texto principal ≥ 7:1; secundário ≥ 4,5:1; rótulo sobre
  marfim usa `text_on_light`.
- **Não depender só de cor:** estado pressionado combina cor + deslocamento +
  marcador interno; sustain combina cor + barra inferior; mudo combina cor +
  texto `MUDO`/`MUTED`.

## 4. Tipografia

- Fonte: **Inter Regular 4.1**, TTF, OFL 1.1, vendorizada em
  `assets/fonts/Inter-Regular.ttf` na Fase 4, com hash verificado pelo
  manifesto.
- TUI: fonte do terminal; usar apenas ASCII, acentos do catálogo e `#`, `+`,
  `-`, `[`, `]`.
- Tamanhos no fb: título 28 px; HUD 18 px; rótulo de nota 16 px; tecla física
  14 px; nota flutuante 22 px; ajuda 18 px com `line_height` 1,25.
- Medir com `fontdue::layout` antes de centralizar; quebra de linha por palavra
  na ajuda.
- Teste de cobertura de glifos para `á é í ó ú ã õ ç â ê ô` e para `# + -`.
  Evitar setas e símbolos musicais; usar "Esq./Dir." e "Left/Right".

## 5. Imagens e animação

- **Sem imagens raster.** Piano, teclas e keycaps são vetoriais
  (`tiny-skia`); a TUI usa células de texto. Não há tiers Kitty/Sixel/iTerm2.
- `--no-animations` remove ripple, fade e nota flutuante; mantém highlight
  estático e o deslocamento do keycap.
- Animações:
  - ataque do highlight: 90 ms, ease-out cúbico;
  - halo/ripple: 220 ms, dois anéis concêntricos, opacidade decrescente;
  - fade do glow ao soltar: 260 ms;
  - nota flutuante: sobe 24 px e desaparece em 600 ms;
  - sustain: anel fino pulsando a 1,2 s (desligado em `--no-animations`);
  - roubo de voz: sem animação própria, apenas release visual.
- Frame ativo 16 ms; ocioso 33 ms. Nada pisca acima de 3 Hz.
- Em `--no-animations`, o estado pressionado persiste enquanto a nota está
  ativa (ou 750 ms no fallback).

## 6. Estados

| Estado | Piano | Teclado físico | HUD |
| --- | --- | --- | --- |
| Normal | marfim/ébano | mapeadas com borda âmbar | oitava, volume, som |
| Pressionado | cor pressed + inset + halo | keycap afundado + âmbar | nota flutuante |
| Sustain ativo | barra inferior em todas as notas ativas | keycaps mantêm brilho | `Sustain: on/latch` |
| Mudo | sem alteração visual | sem alteração | `MUDO` + token danger |
| Áudio indisponível | sem alteração | sem alteração | `Sem áudio` + danger |
| Ajuda | escurecido atrás do painel | escurecido | painel modal |
| Sem foco/desabilitado | não se aplica (sem mouse) | teclas não mapeadas a 35% | — |

Estados de erro de runtime (fb sem permissão, TUI sem TTY) não são telas:
vão para stderr com código de saída acionável.
