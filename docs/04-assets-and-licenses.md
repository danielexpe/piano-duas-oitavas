# 04 — Assets e licenças

> Congelado na Fase 0. O v1 não usa imagens raster; o único asset binário
> planejado é a fonte de UI.

## 1. Pipeline

```text
assets/src/        # originais editáveis (nenhum no v1)
assets/generated/  # derivados versionados (nenhum no v1)
assets/fonts/      # Inter-Regular.ttf + OFL-Inter.txt
assets/manifest.toml
```

- `cargo xtask assets` gera/valida derivados e manifesto.
- `cargo xtask assets --check` é portão: falha se o manifesto ou a fonte
  divergirem do esperado.
- Nunca editar derivado gerado à mão.
- Embutir com `include_bytes!`; testar decode e execução copiada para `/tmp`
  sem a pasta de assets.
- Não há pipeline de imagens (SVG → PNG) no v1. O piano, as teclas e os
  keycaps são desenhados vetorialmente em runtime.

## 2. Inventário

| Asset | Origem | Derivado | Uso | Licença |
| --- | --- | --- | --- | --- |
| fonte de UI | `Inter-4.1.zip` (`extras/ttf/Inter-Regular.ttf`) | `assets/fonts/Inter-Regular.ttf` | texto no fb, HUD, ajuda | OFL 1.1 |
| piano vetorial | código (`render/fb.rs`) | — | teclas brancas/pretas | CC BY 4.0 (arte original do projeto) |
| teclado físico vetorial | código (`render/fb.rs`) | — | keycaps e rótulos | CC BY 4.0 (arte original do projeto) |
| paleta/tokens | código (`theme.rs`) | — | cores e estados | MIT (código) |

> A fonte Inter 4.1 está vendorizada em `assets/fonts/Inter-Regular.ttf`
> (SHA-256 `40d692fce188e4471e2b3cba937be967878f631ad3ebbbdcd587687c7ebe0c82`),
> obtida de `https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip`
> com aprovação do usuário em 2026-10-31. Origem, versão e hashes ficam em
> `assets/manifest.toml` e são verificados por `cargo xtask assets --check`.
> Q-001 está resolvida em `docs/07`.

## 3. Fontes

| Fonte | Arquivo | Licença |
| --- | --- | --- |
| Inter Regular | `assets/fonts/Inter-Regular.ttf` | OFL 1.1 |
| Inter (texto da licença) | `assets/fonts/OFL-Inter.txt` | OFL 1.1 |

- Incluir o texto da licença no pacote.
- Cobertura dos glifos do catálogo (`docs/08`) verificada por teste em
  `src/assets.rs` na Fase 4.
- A TUI não usa a fonte embutida; usa a fonte do terminal.

## 4. Licenças do projeto

- Código: MIT (`LICENSE`).
- Arte original (piano/keycaps vetoriais): CC BY 4.0 (`LICENSE-ART`), com
  crédito em `CREDITS.md`.
- Fonte: OFL 1.1, com texto completo em `assets/fonts/`.
- `THIRD_PARTY_NOTICES.md` lista todas as crates e a fonte.
- ALSA (`libasound`) é carregada dinamicamente do sistema e não é distribuída
  no pacote; registrar em `THIRD_PARTY_NOTICES.md`.

## 5. Regras

- Não embutir asset de terceiro sem licença compatível e crédito.
- Toda dependência nova entra em `THIRD_PARTY_NOTICES.md` e no orçamento de
  tamanho.
- Releases públicas levam licenças e créditos.
- Sem rede em runtime; qualquer download de asset é feito em desenvolvimento,
  com aprovação e registro no manifesto.
