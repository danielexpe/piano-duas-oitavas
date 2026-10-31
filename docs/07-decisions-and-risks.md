# 07 — Decisões, riscos e questões

## ADRs

### D-001 — Dois backends com um único estado

- **Status:** aceita.
- **Contexto:** o app precisa rodar em terminal e em console local.
- **Decisão:** um `App` compartilhado e dois backends (`tui`, `fb`) que
  consomem a mesma cena lógica.
- **Consequências:** `core` e `state` ficam puros; `render/scene` é
  compartilhado; cada backend tem seu próprio ciclo de vida e restauração.

### D-002 — `--backend auto` prefere TUI

- **Status:** aceita.
- **Contexto:** detectar fbdev automaticamente é frágil (SSH, X/Wayland,
  containers, `/dev/fb0` sem VT ativo).
- **Decisão:** `auto` nunca escolhe fb; fb é explícito.
- **Consequências:** o usuário do console precisa passar `--backend fb`.

### D-003 — Duas oitavas, base 3, deslocamento 1..=6

- **Status:** aceita.
- **Contexto:** o produto pede exatamente duas oitavas visíveis.
- **Decisão:** 24 notas cromáticas, padrão C3–B4 (MIDI 48–71); `←`/`→`
  deslocam a oitava base entre 1 e 6.
- **Consequências:** 14 brancas e 10 pretas; o mapa cresce em múltiplos de 12
  sem mudar a geometria; trocar de oitava libera vozes para evitar notas presas.

### D-004 — Mapeamento ABNT2 em duas fileiras

- **Status:** aceita.
- **Contexto:** o usuário quer teclas físicas visíveis e associadas às notas.
- **Decisão:** brancas em `Z X C V B N M` (base) e `Q W E R T Y U` (base+1);
  pretas em `S D G H J` e `2 3 5 6 7`; aliases deslocados `@ # % &`.
- **Consequências:** 24 teclas únicas; letras normalizadas; `¨` do Shift+6 não
  é confiável; o layout não é remapeável no v1.

### D-005 — Tela única com ajuda modal, sem menu

- **Status:** aceita.
- **Contexto:** o app deve abrir tocando, com o mínimo de cliques.
- **Decisão:** abrir direto no `Piano`; `F1` abre `Help` sobreposto; `Esc`
  fecha a ajuda e, se já fechada, sai.
- **Consequências:** não há tela de configurações nem menu; a ajuda concentra
  o mapa completo e os atalhos.

### D-006 — Áudio sintetizado polifônico, sem samples

- **Status:** aceita.
- **Contexto:** som imediato, offline, binário pequeno e sem licenças de
  amostras.
- **Decisão:** síntese aditiva em runtime, 8 vozes, ALSA dinâmico, feature
  `audio` desligável; ausência de dispositivo = silêncio.
- **Consequências:** timbre aproximado de piano, não idêntico a um sample;
  sem arquivos de áudio; testes de síntese sem hardware.

### D-007 — Sem persistência no v1

- **Status:** aceita.
- **Contexto:** evitar IO e corrupção de config no console.
- **Decisão:** volume, idioma, oitava e mudo vivem só em memória; flags de CLI
  definem o inicial.
- **Consequências:** o usuário reajusta preferências a cada execução; a Fase 7
  pode adicionar XDG.

### D-008 — Idiomas pt-BR/en e licença MIT

- **Status:** aceita.
- **Contexto:** público brasileiro, com opção de inglês.
- **Decisão:** catálogo i18n completo, padrão pt-BR, `F2` alterna; código MIT;
  arte original CC BY 4.0; fonte OFL 1.1.
- **Consequências:** toda string visível passa pelo catálogo; não há texto
  hardcoded na UI.

### D-009 — Sem imagens raster; piano vetorial

- **Status:** aceita.
- **Contexto:** nitidez em qualquer resolução e menos assets/licenças.
- **Decisão:** desenhar piano e keycaps com `tiny-skia` no fb e com células de
  texto na TUI; nenhum protocolo de imagem no terminal.
- **Consequências:** sem tiers Kitty/Sixel; o único asset binário é a fonte.

### D-010 — Key-up de melhor esforço com fallback

- **Status:** aceita.
- **Contexto:** muitos terminais e o console em modo texto não entregam
  eventos de soltura.
- **Decisão:** usar key-up real quando o protocolo de teclado aprimorado
  estiver disponível; senão, auto-release em 750 ms. Espaço segura o sustain
  com key-up real e alterna latch no fallback.
- **Consequências:** comportamento ligeiramente diferente entre terminais;
  o HUD indica o modo; evdev fica para a Fase 7.

### D-011 — `F3` para mudo; sair apenas com `Esc`/`Ctrl+C`

- **Status:** aceita.
- **Contexto:** `M` e `Q` são notas (Si e Dó) no mapeamento escolhido.
- **Decisão:** mudo em `F3`; saída em `Esc` (com ajuda fechada) e `Ctrl+C`.
- **Consequências:** não há atalho de uma letra para mudo/sair; a ajuda e o
  rodapé deixam isso explícito.

### D-012 — Hardware alvo e layout adaptativo

- **Status:** aceita.
- **Contexto:** console de referência 1920 × 1080/32 bpp, mas há telas menores.
- **Decisão:** cena Wide em ≥ 1280 × 720; Compact em ≥ 1024 × 600; Minimal
  abaixo disso. O canvas lógico é escalado com bilinear para até 96% do
  dispositivo e centralizado; suportar 32/24/16 bpp.
- **Consequências:** a implementação foi entregue na Fase 6; a validação real
  em VT, 24/16 bpp e resoluções menores segue pendente.

### D-013 — `--seed` reservado, sem aleatoriedade musical

- **Status:** aceita.
- **Contexto:** o framework prevê determinismo, mas o piano não tem RNG.
- **Decisão:** aceitar `--seed` por compatibilidade e reservá-lo; nenhuma
  decisão musical ou visual depende dele no v1.
- **Consequências:** capturas são determinísticas sem seed; o flag não muda o
  comportamento.

### D-014 — Seleção e fallback de dispositivo ALSA

- **Status:** aceita.
- **Contexto:** no console/VT o PCM `default` pode falhar porque o shell não
  herdou `XDG_RUNTIME_DIR`/DBus do serviço PipeWire/PulseAudio, ou porque o
  dispositivo preferido está ocupado (o headset USB pode retornar `EBUSY`
  quando o servidor de áudio o detém).
- **Decisão:** preparar as variáveis de sessão ausentes a partir de
  `/run/user/<uid>`; tentar `default`, `pipewire`, `pulse` e, por último,
  dispositivos de hardware, com preferência por analógico/USB sobre HDMI.
  Expor `--audio-device <NAME>` (padrão `default`) e `auto`; um nome explícito
  não sofre fallback, exceto `hw:` → `plughw:`. `PIANO_AUDIO_DEBUG=1` loga a
  escolha e as falhas no stderr.
- **Consequências:** o áudio funciona no VT mesmo sem ambiente gráfico; se o
  PipeWire não estiver acessível, o som pode sair pelo analógico/HDMI. A
  escolha automática pode não corresponder ao dispositivo que o usuário
  ouve, por isso o seletor explícito é documentado.

## Premissas

- Linux x86_64 local; Rust stable, edição 2021, MSRV 1.88.
- `/dev/fb0` disponível no console alvo; usuário no grupo `video`.
- Sem rede em runtime.
- Fonte e arte embutidas; arte vetorial gerada pelo próprio código.
- O usuário aprovou a aquisição da fonte Inter em 2026-10-31 (Q-001 resolvida).

## Questões abertas

| ID | Questão | Prazo | Estado |
| --- | --- | --- | --- |
| Q-001 | Vendorizar `Inter-Regular.ttf` + `OFL-Inter.txt` exige download; aprovar origem e hash. | Fase 4 | resolvida em 2026-10-31: Inter 4.1 de `rsms/inter`, hashes em `assets/manifest.toml` |
| Q-002 | Confirmar em hardware real: driver, VT, resolução, bpp e grupo `video`. | Fase 6 | aberta |
| Q-003 | Medir key-up real por terminal (xterm, foot, kitty, alacritty, console) e registrar o fallback observado. | Fases 2 e 6 | aberta |
| Q-004 | Validar 24/16 bpp e resoluções menores em hardware; se indisponível, registrar pendente. | Fase 6 | aberta |
| Q-005 | Testar aliases `@ # % &` e a tecla morta `¨` em ABNT2 real; ajustar somente se necessário. | Fase 2 | aberta |

Não há questão aberta de produto: escopo, mapeamento, telas, áudio, idiomas,
persistência e licenças estão congelados em `docs/01`.

## Riscos

| Risco | Impacto | Mitigação |
| --- | --- | --- |
| Terminal não entrega key-up | notas presas ou curtas | auto-release 750 ms + latch; HUD indica o modo |
| `Shift+6` (`¨`) é tecla morta em ABNT2 | preta superior não toca com Shift | documentar uso de `6`; alias só quando o caractere chega |
| Auto-repeat do terminal | reataques indesejados | ignorar `KeyEventKind::Repeat` |
| Console blanked/DPMS | tela invisível | `FBIOBLANK(FB_BLANK_UNBLANK)` após `KD_GRAPHICS` |
| Driver anuncia ypan e não honra | frame não aparece | fallback para buffer visível |
| Sem `/dev/fb0` ou sem permissão | backend indisponível | erro acionável, código 3 |
| `tty` = `/dev/pts/N` | diagnóstico falso negativo | validar VT antes de testar |
| Sem ALSA ou device ocupado | sem som | silêncio + HUD; thread isolada; sem panic |
| Latência/glitch de áudio | experiência ruim | buffer ~21 ms, canal limitado, fade e teste de síntese |
| Fonte sem glifos | texto quebrado no fb | teste de cobertura na Fase 4 e fallback ASCII |
| Roubo de voz audível | clique | fade de 5 ms na voz roubada |
| 60 FPS em 1080p | CPU alta | cache de fundo/fonte, frame só com animação ativa |
| Baixa resolução no fb | texto ilegível | breakpoint Minimal e escala mínima de 11 px |
| Troca de VT durante execução | resíduos no console | detectar perda de foco/erro, restaurar e sair |
| Fonte Inter sem glifos | texto quebrado no fb | Q-001 resolvida; teste de cobertura em `src/assets.rs` |
| Sem validação de hardware | release frágil | registrar pendência; não declarar validado |
