# Sistema de design — General Bots Desktop (edição gbgov)

Padrão de design obrigatório para toda a UI da General Bots (suite, cloud, login, assistant, splash e aplicativos de desktop). Descreve o que está **no código hoje** (árvore de trabalho, 2026-11-26). Nada aqui é proposta; cada valor vem de um arquivo, citado como `arquivo:linha`. A fonte canônica de todos os tokens `--id-*` é `suite/css/identity-tokens.css` — o arquivo `suite/identity/tokens.css` é apenas um shim de três linhas que o importa.

Legenda de caminhos (todos sob `botui/ui/`):

| Abreviação | Caminho |
|---|---|
| `identity-tokens.css` | `suite/css/identity-tokens.css` (shim: `suite/identity/tokens.css` → `@import`) |
| `accent.js`, `theme.js`, `textsize.js`, `fonts.css` | `suite/identity/` |
| `default.css` | `suite/public/themes/` |
| `assistant*.css`, `titlebar.css`, `assistant.html` | `suite/assistant/` |
| `NN_*.js` | `suite/assistant/modules/` |
| `splash.html` | `botui/ui/splash.html` |

---

## 0. O que é o General Bots Desktop (edição gbgov)

O General Bots Desktop é um **assistente de inteligência artificial que roda no computador do próprio servidor público**, pensado para o setor público brasileiro: conversa (Chat) e arquivos (Drive), em que a organização decide para onde cada dado pode ir (`GBGOV/PLAN.md` §1). É uma edição reduzida da plataforma General Bots, que é de código aberto e escrita em Rust (`README.md`); nesta edição só Chat e Drive aparecem, e os outros aplicativos ficam ocultos (`botserver/.product.gblite`, `apps=chat,drive`).

**O que ele faz**
- Conversa com o assistente: a resposta aparece enquanto é escrita, com um cartão de "raciocínio" que pode ser aberto, formatação em Markdown, botão de parar e de gerar de novo (`GBGOV/MVP.md` §2).
- Lê anexos: PDF, Word e texto; o texto de imagens e digitalizações é lido neste computador (OCR, no Windows) (`10c_settings_garage.js:26,245`; `10_settings_model.js` `noImageReading`). Fotos de até 25 MB são reduzidas antes de ir a um modelo que entende imagens (commit `76863182f`).
- Drive: listar, enviar arquivos e usar um arquivo na conversa (`GBGOV/MVP.md` §2).
- Quatro formas de "motor" (Configurações > Modelo, `10c_settings_garage.js:15-29`):
  - **Neste computador**: modelo local; "Nada sai deste computador". Pede placa de vídeo com 6 GB ou mais e um download de 4 a 6 GB.
  - **Servidor ou chave própria**: o servidor da organização ou a chave de um provedor; os textos vão ao servidor indicado.
  - **Modelo gratuito**: hospedado fora do computador, sem chave e com limites de uso menores; "Tudo que você escrever e anexar é enviado a" esse provedor externo (`10_settings_model.js:17,47-49`).
  - **Agente por assinatura**: a assinatura da pessoa (Claude, ChatGPT, Gemini…) "num espaço isolado neste computador" (`10c_settings_garage.js:19-20`). No Windows esse espaço depende de um invólucro WSL2; sem ele o modo agente é desligado (`botserver/.product.gblite`, comentário inicial).
- Pesquisa na web: opcional, a pessoa liga em Configurações, e só aparece se o produto a oferecer (`10b_settings_model_local.js:378-389`; `05b_model_picker.js:47-55`).

**Soberania e privacidade.** Com o modelo local, as conversas ficam na máquina (`10_settings_model.js:54`). Os dados só saem quando a pessoa escolhe um motor externo, e o aplicativo pede confirmação antes ("O que você escrever e anexar será enviado a %h, fora deste computador. Continuar?"). Baixar um modelo também pede confirmação e mostra de onde ele vem (`10_settings_model.js:34-35`). O painel mostra sempre para onde vão os textos: "Ficam neste computador", "Ficam na rede da sua organização" ou "Vão a um provedor externo" (`10c_settings_garage.js:27`).

**Como é empacotado.** É um aplicativo de desktop (Tauri 2) que liga um servidor local (`botserver`, porta 8080) e só mostra a janela quando esse servidor responde (`AGENTS.md`, "Desktop-Local Edition"). Os serviços de apoio vêm dentro do instalador: PostgreSQL, OtterIO (arquivos), Valkey (compatível com Redis), Qdrant (busca vetorial), HashiCorp Vault (segredos), Zitadel (identidade) e llama.cpp (modelo local) (`botserver/3rdparty.toml`). O instalador é offline: não baixa componentes na primeira execução e confere o SHA-256 de cada um (`AGENTS.md`, "Offline Installation"). O plano interno prevê trocar essa pilha de serviços por componentes embutidos; isso ainda não foi feito (`GBGOV/PLAN.md` §2, decisão 1). A entrada exige login do administrador criado no assistente de primeira execução (`no_security=false`).

**Plataformas.** Windows (`.msi`, instalador NSIS em pt-BR e inglês, `.exe`), Linux (arquivo único autoextraível e tarball; `.deb`, `.rpm` e `.AppImage` são feitos sem garantia e nunca saíram numa versão publicada) e macOS arm64 (`.dmg`/`.app` sem assinatura, em pré-lançamento) (`AGENTS.md`, "Tauri Configuration" e "Build Pipeline"). O alvo desta edição é o Windows 11.

---

## 1. Princípios (só os escritos no código ou no plano)

- **Identidade por tokens `--id-*`**: neutros, raios, sombras, movimento, escala de tipo e fontes são fixos; só o acento muda em tempo de execução (`identity-tokens.css:1-9`). O tema Default não define paleta paralela: ele só apelida variáveis legadas para os tokens (`default.css:17-126`).
- **Acento derivado de uma só cor**, com contraste WCAG 2 ≥ 4,5:1 para o texto de acento (`accent.js:1,160-231`).
- **Sem flash no primeiro quadro**: tema e tamanho do texto são aplicados no `<head>` antes da pintura (`theme.js:1-11`, `textsize.js:1-7`, `assistant.html:11-14`).
- **Offline**: ícones e logos inline, "no font, no CDN, no sprite file" (`11_icons.js:5-6`; `10f_provider_logos.js:3`); fontes woff2 locais (`fonts.css:1-7`).
- **Barra de qualidade**: "a experiência, o design e a utilidade dos melhores assistentes de desktop; o diferencial é a soberania dos dados, visível e verificável" (`GBGOV/PLAN.md` §1).

---

## 2. Tokens

Fonte canônica: `identity-tokens.css` (carregada em `desktop.html:17`). `:root` contém o claro; `:root[data-color-scheme="dark"]` (`:167-227`) contém o escuro — um único bloco, sem duplicação. `default.css` só apelida variáveis legadas da suíte (`--background`, `--primary`, `--win-*`, `--sentient-*`) para estes tokens.

### 2.1 Cores — acento (padrão claro; `accent.js` sobrescreve)

| Token | Claro (padrão) | Escuro | Fonte |
|---|---|---|---|
| `--id-accent` | `#F28C28` | `hsl(30 89% 55%)` (L+0,05 máx. 0,82, C×1,05 em `deriveDark`) | `identity-tokens.css:19`, `:169`; `accent.js:98-106` |
| `--id-accent-hover` | `#D4751C` | acento a 86 % com preto (L+0,06) | `:20`, `:170`; `accent.js:109-110` |
| `--id-accent-light` | `#FDB96A` | acento a 68 % com branco | `:21`, `:171`; `accent.js:112-115` |
| `--id-accent-text` | `#B45309` | `hsl(32 96% 72%)` (clareia até 4,5:1) | `:22`, `:172`; `accent.js:120-133` |
| `--id-accent-soft` | `rgb(242 140 40 / .1)` | acento a 12 %, transparente | `:23`, `:173`; `accent.js:138` |
| `--id-accent-glow` | `rgb(242 140 40 / .26)` | acento a 26 %, transparente | `:24`, `:174`; `accent.js:139` |
| `--id-on-accent` | `#2C2C2C` (tinta, não branco) | `hsl(0 0% 17%)` | `:27`, `:175`; `accent.js:141-146` |
| `--id-paper` (fundo da página) | `#FFF8F2` | `hsl(34 11% 12%)` no CSS; `accent.js` deriva `#1c1c28` | `:28`, `:176`; `accent.js:89,117-118` |
| `--id-paper-band` | — (só via JS) | papel a 88 % | `accent.js:257-260` |
| `--id-accent-muted` | `color-mix(accent 55 %, transparent)` | idem, sobre o acento escuro | `:142`, `:192` |
| `--id-accent-rgb` | `242, 140, 40` (triplet com vírgulas, para `rgba(var(--id-accent-rgb), a)`) | mesmo triplet; `accent.js` reescreve com o acento aplicado | `:145`, `:195`; `accent.js:251-256` |
| `--id-accent-bg` | `var(--id-accent-soft)` | idem | `:146`, `:196` |
| `--id-focus` | `0 0 0 2px var(--id-accent-text)` (é um **box-shadow**) | idem | `:156`; use `box-shadow: var(--id-focus)`, nunca `outline` |

### 2.2 Cores — tinta, neutros, superfícies, bordas

| Token | Claro | Escuro | Fonte (claro / escuro) |
|---|---|---|---|
| `--id-ink` (título, texto forte) | `#2C2C2C` | `hsl(240 12% 91%)` (≈ `#e4e4ea`) | `:31` / `:177` |
| `--id-text` (corpo) | `#4B5563` | mistura de 82 % ink com papel | `:32` / `:178` |
| `--id-muted` | `#5F6673` | `hsl(226 11% 56%)` (≈ `#808898`) | `:33` / `:179` |
| `--id-text-secondary` | `var(--id-text)` | `var(--id-text)` | `:143` / `:193` |
| `--id-fg` | `var(--id-ink)` | `var(--id-ink)` | `:144` / `:194` |
| `--id-gray-100…500` | `#f3f4f6 #e5e7eb #d1d5db #9ca3af #6b7280` | tons escuros derivados de ink e papel | `:34-38` / `:180-184` |
| `--id-surface` (translúcida) | `rgb(255 255 255 / .9)` | superfície escura a 90 % | `:41` / `:185` |
| `--id-surface-solid` (cartões) | `#ffffff` | `hsl(240 15% 17%)` (≈ `#262633`) | `:42` / `:186` |
| `--id-surface-alt` / `-hover` | `var(--id-fill)` | `var(--id-fill)` | `:149-150` / `:199-200` |
| `--id-line` (borda) / `--id-border` | `rgb(229 231 235 / .8)` | borda escura a 90 % | `:43`,`:147` / `:187`,`:197` |
| `--id-line-soft` (divisória) | `rgb(243 244 246 / .9)` | borda escura a 55 % | `:44` / `:188` |
| `--id-fill` (hover, fundo leve) | `rgb(243 244 246 / .8)` | superfície escura a 80 % | `:45` / `:189` |
| `--id-overlay` / `-heavy` | `rgb(0 0 0/.3)` / `.7` | `.5` / `.8` | `:118-119` / `:217-218` |
| `--id-scrollbar-thumb` | `var(--id-gray-300)` | `var(--id-gray-300)` (já escuro) | `:134` / `:182` |
| `--id-radius` | `var(--id-radius-control)` | idem | `:148` / `:198` |
| `--gb-field` (fundo de campo, local) | `color-mix(surface-solid 97 %, ink)` | idem (fica mais claro) | `assistant-settings.css:287` |

### 2.3 Cores — estados

| Estado | Token | Claro | Escuro | Fonte |
|---|---|---|---|---|
| Erro | `--id-danger` / `-text` / `-icon` | `#dc2626` / `#b91c1c` / `#ef4444` | `#f87171` / `#fca5a5` / `#f87171` | `identity-tokens.css:105-109`, `:204-208` |
| | `--id-danger-soft` / `-line` | `rgb(220 38 38/.06)` / `.3` | `.12` / `.4` | `:107-108` / `:206-207` |
| Sucesso | `--id-success` / `-text` | `#22c55e` / `#16a34a` | `#4ade80` / `#86efac` | `:110-113`, `:209-212` |
| | `--id-success-soft` / `-line` | `rgb(34 197 94/.06)` / `.25` | `.12` / `.35` | `:112-113` / `:211-212` |
| Aviso | `--id-warn` / `-text` | `#d97706` / `#92400e` | `#fbbf24` / `#fde68a` | `:114-117`, `:213-216` |
| | `--id-warn-soft` / `-bg` | `rgb(234 179 8/.08)` / `#fef3c7` | `.15` / `rgb(120 80 10/.3)` | `:116-117` / `:215-216` |

Realce de sintaxe: `--id-hl-*` (`identity-tokens.css:122-131`, escuro `:219-226`); a palavra-chave usa `--id-accent-text`.

### 2.4 Tipografia

| Item | Valor | Fonte |
|---|---|---|
| Corpo | `var(--id-font-sans)` = `'Inter', system-ui, -apple-system, 'Segoe UI', sans-serif` (Inter variável 100–900) | `identity-tokens.css:48`, `fonts.css:76-92` |
| Títulos e marca | `var(--id-font-heading)` = `'Fira Sans', …` (400, 500, 600, 700) | `:49-50`, `fonts.css:10-74` |
| Monoespaçada | `var(--id-font-mono)` = pilha de sistema (`ui-monospace`, `SFMono-Regular`, `Consolas`, `Liberation Mono`, `monospace`) — sem download | `:151`, `:201`; uso em `assistant-settings.css:84`, `assistant-providers.css:52,120` |
| Pesos usados | 400, 500 (rótulos, ativo), 600 (títulos, marca), 700 (marcas ✓, %) | p.ex. `assistant.css:128,270`, `assistant-garage.css:38,173` |
| Altura de linha | 1.5 base; 1.6 balão do usuário; 1.7 resposta | `assistant.css:12`, `assistant-chat.css:106,122` |

Escala. `identity-tokens.css:57-66` define os valores-base em **rem** (para seguir o zoom do navegador/SO), com piso de 0,75rem (12px). `textsize.js` reescreve os seis primeiros em **px** antes do primeiro quadro, a partir do tamanho escolhido (pequeno 14, **padrão 16**, grande 18, extra 20; `textsize.js:10,20-31`). Valores efetivos:

| Token | CSS | Efetivo no padrão (16) | Regra em `textsize.js` |
|---|---|---|---|
| `--id-text-xs` | 0.75rem (`:57`) | 12px | max(10, px−4) |
| `--id-text-sm` | 0.75rem (`:58`) | 14px | max(11, px−2) |
| `--id-text-base` | 0.875rem (`:59`) | 16px | px |
| `--id-text-md` | 0.9375rem (`:60`) | 17px | px+1 |
| `--id-text-lg` | 1rem (`:61`) | 18px | px+2 |
| `--id-text-xl` | 1.0625rem (`:62`) | 19px | px+3 |
| `--id-text-2xl` / `3xl` / `4xl` / `5xl` | 1,25 / 1,5 / 2 / 2,25rem (`:63-66`) | iguais (não escalam) | — |

Atalhos: Ctrl+= / Ctrl+− / Ctrl+0 (`textsize.js`). Saudação: Fira 600 em `--id-text-4xl`, 3xl abaixo de 960px e 2xl abaixo de 600px (`assistant-chat.css:15-17,486-492`).

### 2.5 Espaçamento (base 4px) — `identity-tokens.css:94-102`

`--id-space-1` 4 · `-2` 8 · `-3` 12 · `-4` 16 · `-5` 20 · `-6` 24 · `-8` 32 · `-10` 40 (px). Coluna da conversa: `max-width: 760px` (`assistant-chat.css:88`, `:393`); cartão do compositor vazio: 680px (`:27`); barra lateral: 280px (`assistant.css:88`); navegação de Configurações: 200px (`assistant-settings.css:14`).

### 2.6 Raios — `identity-tokens.css:69-76`

Derivados de `--radius` (0,75rem no tema Default, `default.css:38`). Valores efetivos no Default:

| Token | Definição | px efetivo | Uso típico |
|---|---|---|---|
| `--id-radius-hero` | `calc(var(--radius, .75rem) + 12px)` (`:70`) | 24 | — |
| `--id-radius-card` | `calc(var(--radius, .75rem) + 8px)` (`:71`) | 20 | `.gb-set-card`, compositor, linhas de provedor |
| `--id-radius-panel` | `calc(var(--radius, .75rem) + 6px)` (`:72`) | 18 | balão do usuário (canto inferior direito em `-sm`) |
| `--id-radius-bar` | `calc(var(--radius, .75rem) + 4px)` (`:73`) | 16 | — |
| `--id-radius-control` / `--id-radius` | `var(--radius, .75rem)` (`:74`, `:148`) | 12 | botão enviar/parar, cartão de modo, campo de busca |
| `--id-radius-sm` | `max(4px, calc(var(--radius, .75rem) - 4px))` (`:75`) | 8 | botões pequenos, campos, itens de navegação, anel de foco |
| `--id-radius-pill` | `999px` (`:76`) | 999 | chips, badges, botão de modelo |

### 2.7 Sombras — `identity-tokens.css:78-83` (sem substituição no escuro)

`--id-shadow-card` `0 1px 2px rgb(0 0 0/.05)`; `--id-shadow-card-hover` `0 8px 20px -10px rgb(0 0 0/.12), 0 1px 2px …`; `--id-shadow-float` (menus) `0 10px 15px -3px rgb(0 0 0/.04), 0 4px 6px -4px …`. As mesmas três sombras valem para claro e escuro; `--win-*` e `--sentient-*` as apelidam em `default.css:73-74,114-116`.

### 2.8 Movimento — `identity-tokens.css:85-92`

| Token | Valor |
|---|---|
| `--id-ease` | `cubic-bezier(.22, 1, .36, 1)` (o mais usado) |
| `--id-ease-standard` | `cubic-bezier(.25, .1, .25, 1)` |
| `--id-dur-fast` / `-normal` / `-medium` / `-slow` | 150 / 200 / 350 / 450 ms |
| `--id-dur-glow` | 20s (0s com reduced-motion, `identity-tokens.css:229-233`) |

Animações nomeadas: deriva do padrão de fundo 30s (`assistant.css:39-44`), cursor de streaming `gb-blink` 1s (`assistant-chat.css:196-200`), pulso de "pensando" 1,4s (`:213-221`), shimmer do chip "Executando" 2s (`assistant-blocks.css:61-68`), abertura do bloco em `--id-dur-medium` (`:77-79`), spinner `gbSetSpin` .8s e barra de espera `gbDlWait` 1,4s (`assistant-settings.css:236-271`). Todas têm bloco `prefers-reduced-motion: reduce` (19 blocos nos CSS do shell).

### 2.9 Camadas (z-index, sem token)

| z | Elemento | Fonte |
|---|---|---|
| 0 | padrão de fundo e faixa de papel (`.gb-main::before/::after`) | `assistant.css:33,52` |
| 1 | conteúdo, padrão CSS, véu fosco | `assistant-chat.css:14,80`, `assistant-patterns.css:16,29` |
| 5 | barra de ações da resposta | `assistant-blocks.css:149` |
| 10 | barra lateral | `assistant.css:86` |
| 20 / 30 | sobreposição de soltar / painel do anexo | `assistant-blocks.css:210,227` |
| 50 | menu de modelos, menu do usuário | `assistant-blocks.css:354`, `assistant-settings.css:191` |
| 99 / 100 | fundo e barra lateral deslizante (< 960px) | `assistant.css:408,418` |
| 200 | barra de título, lightbox, menu de contexto do Drive | `titlebar.css:7`, `assistant-blocks.css:312`, `assistant-drive.css:207` |
| 300 / 400 | diálogo em página, toasts do Drive | `assistant-patterns.css:95`, `assistant-drive.css:272,322` |

---

## 3. Temas

- **Claro / escuro / sistema**: `theme.js` grava `gb-theme` no localStorage, aplica `data-color-scheme` em `<html>` e acompanha o sistema ao vivo (`theme.js:6-11,28-42,64-81`). Há **um único** bloco escuro de tokens, `:root[data-color-scheme="dark"]` (`identity-tokens.css:167-227`), sem duplicação e sem bloco `@media` paralelo. `theme-scheme.js` decide o esquema pela cor de fundo do tema ativo; o esquema nunca é escolhido pelo id do tema no token layer.
- **Tema Default**: não traz paleta própria — só apelida `--background`, `--primary`, `--win-*`, `--sentient-*` para os tokens (`default.css:17-126`). O `--radius` do contrato da suíte é o único escalar concreto (`default.css:35`), do qual derivam os raios `--id-*`.
- **Acento**: oito predefinidos — Laranja `#F28C28`, Azul `#2563EB`, Verde `#059669`, Azul-petróleo `#0D9488`, Roxo `#7C3AED`, Vermelho `#DC2626`, Grafite `#6B7280`, Âmbar `#D97706` (`accent.js:266-275`) — e um seletor de cor livre (`09_settings.js:204-229`). A derivação é feita em OKLCH: hover L−0,078; light L+0,099 com C×0,77; paper L 0,983 com croma ≤ 0,012; texto escurece até 4,5:1 (`accent.js:160-206`). O laranja de referência devolve os literais exatos (`accent.js:163-170`). `apply()` grava as variáveis em `document.documentElement.style` (`accent.js:239-262`), incluindo `--id-accent-rgb`; ao trocar o tema, `reapply()` deriva de novo (`theme.js:40`).
- **Origem do acento**: primeiro `productConfig.accent` (gravado pelo assistente de primeira execução em `<stack>/conf/system/appearance.json`, `botserver/crates/botcore/src/product/appearance.rs:1-6`), depois `localStorage gb-accent` (`09_settings.js:439-445`). Ao escolher um acento em Configurações, ele é salvo no localStorage e enviado a `/api/appearance` (`09_settings.js:303-310`).
- **Fundos** (Configurações > Aparência): `none`, `image` (padrão, `pattern.png` usado como máscara sobre um gradiente do acento, opacidade .32), `dots`, `microdots`, `grid`, `blueprint`, `weave`, `mosaic` (`09b_patterns.js:12-22`; `assistant.css:32-40`). Três controles deslizantes: opacidade 20–300 %, escala 50–300 %, véu fosco 0–100 % (desfoque até 16px e papel até 45 %) (`09b_patterns.js:67-69`; `assistant-patterns.css:21-33`). Uma faixa de papel a 88 % cobre só a coluna da conversa, 808px, com 80px de esmaecimento em cada lado (`assistant.css:48-60`); abaixo de 960px o papel é sólido.
- **Splash** (edição Lite, janela 776x443): tema escuro próprio. A imagem `suite/identity/splash/splash-gear-v3.jpg` (engrenagem de vidro, feixe e arco-íris) ocupa a janela inteira; no canto superior esquerdo, o aviso de licença, o texto legal e o crédito; embaixo, à esquerda, a marca GB + LITE com a versão e o slogan, e à direita a etapa atual com barra branca, contador e porcentagem.

---

## 4. Componentes recorrentes

### 4.1 Botões

| Classe | Aparência | Fonte |
|---|---|---|
| `.gb-set-btn-sm` | borda `--id-line`, fundo `surface-solid`, `text-xs`, raio sm; hover `--id-fill` | `assistant-settings.css:255-260` |
| `.gb-set-btn-accent` | fundo acento, `--id-on-accent`; hover `accent-hover` | `:261-262` |
| `.gb-set-btn-danger` / `.gb-set-signout` | texto `danger`, borda `danger-line`; hover `danger-soft` | `:263-264`, `:87-95` |
| `.gb-send-btn` | 36×36, raio control, acento; desabilitado .4 | `assistant-chat.css:427-435` |
| `.gb-stop-btn` | 36×36, borda, `surface` | `:436-443` |
| `.gb-btn-icon` | 32×32 transparente, hover `fill` | `assistant.css:138-153` |
| `.gb-new-chat-btn` | borda tracejada; hover tom de acento | `assistant.css:156-176` |
| `.gb-consent-allow` / `-deny` | primário com acento / secundário | `assistant-chat.css:334-344` |
| `.gb-prov-btn` | 6×14, raio sm; na linha em hover a borda fica acento | `assistant-providers.css:37-39` |
| `.gb-set-lang-btn` (também tema e tamanho) | botão de bloco; `.active` com borda acento e fundo soft | `assistant-settings.css:170-184,321-325` |

### 4.2 Campos

`.gb-set-input` usa fundo `--gb-field`, borda `line`, raio sm e `text-sm`; no foco a borda fica acento, sem anel (`assistant-settings.css:288-293`). `.gb-set-textarea` usa fundo `--id-paper` e foco com `box-shadow: var(--id-focus)` (`assistant-settings.css:354`). O mesmo padrão aparece em `.gb-prov-btn`/`.gb-prov-row` (`assistant-providers.css:41,114`), `.gb-block-header` (`assistant-blocks.css:40`), `.gb-gar-choice` (`assistant-garage.css:72,109`) e `.gb-agent-toggle` (`assistant-chat-ui.css:175`) — sempre `box-shadow`, nunca `outline: var(--id-focus)`. A busca de provedor `.gb-prov-search` tem borda de 1,5px, raio card e, no foco, halo `0 0 0 3px accent-soft` (`assistant-providers.css:7-13`). A busca da barra lateral `.gb-search-input` tem ícone a 10px (`assistant.css:179-203`).

### 4.3 Badges, pills e chips

| Classe | Estilo | Fonte |
|---|---|---|
| `.gb-badge` (+ `-active`, `-ok`) | pill 1×6px, **10px fixo**, fill/muted; ativo em acento; ok em sucesso | `assistant-settings.css:250-254` |
| `.gb-prov-badge` | pill acento cheio | `assistant-providers.css:35-36` |
| `.gb-prov-state` (rótulo `linked`) | pill de sucesso 5×12px, 600 — é estado, não ação | `:40-42`, `10e_settings_providers.js:151` |
| `.gb-gar-soon` / `.gb-prov-soon` (em breve, conectando) | `warn-soft` + `warn-text` | `assistant-garage.css:184`, `assistant-providers.css:75-76` |
| `.gb-gar-chip` (`.on`) | recurso ligado/desligado | `assistant-garage.css:46-48` |
| `.gb-gar-mark.yes/.part/.no` | círculo 16px com ✓ / ~ / – | `:37-44` |
| `.gb-block-chip--running/done/failed/needs-you` | chip de estado do bloco | `assistant-blocks.css:56-73` |
| `.gb-model-tag` | pill do modelo que escreveu a resposta | `assistant-providers.css:64-69` |
| `.gb-suggestion-chip`, `.gb-file-chip` | sugestões e anexos no compositor | `assistant-chat.css:60-72,410-415` |

### 4.4 Cartões

- `.gb-set-card`: `surface-solid`, borda, raio card, 20px de padding; é a unidade de toda seção de Configurações (`assistant-settings.css:61-65`).
- **Cartões de modo** `.gb-set-mode-card`: `-active` (soft, em uso) e `-selected` (anel de 2px de acento) (`assistant-settings.css:218-231`). Na oficina viram cabeçalhos de coluna elevados `.gb-gar-head`, com hover que sobe 1px e sombra hover (`assistant-garage.css:99-118`).
- **Oficina** (Configurações > Modelo): painel com faixa de gradiente de 4px no topo (`.gb-gar-dash`, `:11-26`); medidores de cinco segmentos (vermelho → verde) (`:29-34`); guia de duas perguntas com borda tracejada (`:51-65`); escolhas `.gb-gar-choice` (`:66-78`); comparação em grade de 4 colunas, mínimo de 480px (`:93-133`); cartão da máquina `.gb-mac-*` com veredito colorido por estado (`:148-181`).
- **Linhas de provedor** `.gb-prov-row`: logo de 40px num quadrado de raio 10px, nome e empresa, descrição, ação; `.is-on` com borda acento de 2px, `.is-soon` sem elevação, `.gb-prov-org` tracejada (`assistant-providers.css:17-29,72-74`).
- **Cartões de erro e de aviso**: `.gb-error-card` com borda danger (`assistant-chat.css:255-262`), `.gb-notice-card`, `.gb-consent-card` (`:297-350`), `.gb-set-warn-card` com faixa esquerda de 3px (`assistant-settings.css:296`).

### 4.5 Toggle (interruptor)

`.gb-set-toggle` é um checkbox com `appearance:none`, trilho de 36×20, botão de 16px; ligado fica em acento e o botão desloca 16px (`assistant-settings.css:358-371`). É usado na pesquisa na web (`10b_settings_model_local.js:384`) e nos modelos de uma chave (`assistant-providers.css:122`). Há também o botão-toggle `.gb-agent-toggle` (`.gb-agent-on` em acento) no compositor (`assistant-chat.css:453-476`).

### 4.6 Acordeões

| Onde | Estado | Seta | Fonte |
|---|---|---|---|
| Menu de modelos, um grupo por conta | classe `.open` | gira 180° | `assistant-providers.css:79-90`, `05b_model_picker.js:246-263` |
| "Modelos disponíveis" (`.gb-km`) | `aria-expanded` + `[hidden]` | gira 90° | `assistant-providers.css:102-125`, `10h_settings_key_models.js` |
| Blocos de atividade | `.open`, `max-height` 0→800px | gira 90° | `assistant-blocks.css:75-94` |
| Raciocínio, citações, aviso bruto | `<details>` nativo | — | `assistant-chat.css:231-248,355-373,305-314` |

### 4.7 Blocos de atividade do chat (`12_blocks.js`, `assistant-blocks.css`)

Contrato: evento `gb.activity` com `{id, kind, status: started|completed|failed|needs_input}`; o mesmo `id` é atualizado no lugar (`12_blocks.js:1-11`). Estado → chip: Executando, Concluído, Falhou, Precisa de você (`:27-39`). Tipos com renderizador próprio: `thinking`, `attachment_read`, `drive_list`, `drive_search`, `drive_read`, `kb_search`, `api_command`, `tool_run`, `permission`, `notice`, `error`; os demais caem em `blockGeneric` (`:311-319`). Anatomia (`assistant-blocks.css:1-19`): `.gb-block` (raio 12px) › `.gb-block-header` (ícone num quadrado de 28px, título, resumo, chip, seta) › `.gb-block-body` › `.gb-block-footer`. Blocos seguidos formam `.gb-block-stack`, com bordas coladas. O ícone de arquivo é colorido por extensão (`12_blocks.js:89-95`).

### 4.8 Menu de modelos e compositor

- `.gb-model-btn`: pill `text-xs` com seta. `.gb-model-menu` abre para cima (`bottom: calc(100% + 6px)`), tem 220–300px, sombra float e z 50; itens com `.active` em soft (`assistant-blocks.css:337-376`). Os modelos são agrupados por conta, com o logo do provedor (`05b_model_picker.js:246-263`).
- Compositor vazio `.gb-composer-card` (680px, raio card, foco-dentro com borda acento e sombra hover) e compositor fixo `.gb-composer-inner` (760px) (`assistant-chat.css:26-51,391-421`). Os textareas não têm anel de foco; o foco aparece no cartão inteiro (`:42-44`).

### 4.9 Barra lateral, barra de título, layout de Configurações

- Barra lateral de 280px, `surface` + borda direita; colapsa para largura 0; abaixo de 960px vira painel deslizante com fundo `overlay` (`assistant.css:85-109,405-428`). Itens `.gb-nav-item` e `.gb-conv-item`, com `.active` em accent-soft e accent-text (`:206-271`). Rodapé com avatar de 24px, linha de estado e ponto verde ou vermelho (`:302-371`).
- Barra de título de 36px, `surface`, 12px; esconde a marca da barra lateral; o botão de fechar no hover é `#e81123` (`titlebar.css:5-20,99,106-108`).
- Configurações: navegação de 200px + painel rolável com `scrollbar-gutter: stable` e barra de rolagem de 12px; abaixo de 900px as abas sobem e mostram só ícones (`assistant-settings.css:4-58,198-211`). Seções: Conta, Aparência, Idioma, Modelo, Sobre (`09_settings.js:4`).

---

## 5. Ícones e logos

- **Phosphor regular** (MIT, `@phosphor-icons/core` 2.1.1), 39 ícones inline em `11_icons.js`, com viewBox 256, `stroke-width="16"` (≈1,5px a 24px), `currentColor` e tamanho padrão 16px. API: `GBIcon(nome, tamanho, classe)` → `<svg class="gb-icon">`; um nome desconhecido gera um `<span>` vazio (`11_icons.js:1-14,69-76`).
- **SVGs escritos à mão** (estilo de traço viewBox 24, `stroke-width="2"`) convivem com o Phosphor: 9 em `assistant.html`, 9 em `10c_settings_garage.js` (ícones de modo 20px), além de `04_sidebar.js`, `08_consent.js` e outros (14 arquivos).
- **Logos de provedores** (`10f_provider_logos.js`, `@lobehub/icons-static-svg` 1.80.0, MIT): `openai, anthropic, gemini, xai, meta, mistral, deepseek, qwen, kimi, zhipu, hunyuan, minimax, openrouter, groq, copilot, cursor, ibm` (17). viewBox 24; as marcas monocromáticas usam `currentColor`, e algumas mantêm as cores da marca (p.ex. hunyuan `#0055E9`). São exibidos a 22px num quadrado de 40px (48/28 no cabeçalho de configuração) e a 16px no menu de modelos (`assistant-providers.css:26-29,48-49,83-84`).

---

## 6. Convenções de código da UI

- **Módulos numerados** carregados em ordem fixa por `<script>` em `assistant.html:265-298` (`01_state` … `15b_feedback`, `99_init`). Letras indicam divisões de um mesmo módulo (`03a_stream`, `10h_…`).
- **IIFE + `"use strict"`** em todos; o namespace compartilhado é `window.GBAssistant`, apelidado `A` (`01_state.js:6-9`). Ferramentas globais à parte: `GBIdentity`, `GBTheme`, `GBTextSize`, `GBIcon`. Funções internas usam o prefixo `A._` (`A._patterns`, `A._providerLogos`).
- **Textos pt-BR + en**: dicionários globais `i18n/pt-BR.js` e `en.js` via `A.t(chave, padrão)` (`01_state.js:80`), mais um dicionário local por módulo (`T`/`L` com chaves `"pt-BR"` e `en`, p.ex. `10_settings_model.js:12`, `10c_settings_garage.js:14`) ou `t(pt, en)` em linha (`12_blocks.js:24`).
- **Limite de 450 linhas por arquivo** (`AGENTS.md:105,639`; comentários de divisão em `03a_stream.js:4`, `09b_patterns.js:4` e outros).
- **Sem CDN**: bibliotecas de terceiros copiadas localmente (`marked`, `highlight.js` 11.9.0, `KaTeX` 0.19.1 — `assistant.html:45-53`), fontes locais.
- **Prefixo `gb-`** em classes e ids (`gbXxx` em camelCase para ids); subprefixos por área: `gb-set-` (Configurações), `gb-gar-` (oficina), `gb-mac-` (máquina), `gb-prov-` (provedores), `gb-km-` (modelos da chave), `gb-block-`, `gb-drv-` (Drive), `gb-attach-`.
- **Escape de HTML** com `esc()` local em cada módulo antes de `innerHTML` (p.ex. `10c_settings_garage.js:11`, `12_blocks.js:15`).

---

## 7. Lacunas e inconsistências observadas

**Tokens antes inexistentes em uso — resolvidos na camada de tokens.** Todos passaram a ter uma definição única nos **dois** blocos de `identity-tokens.css`; os consumidores usam `var(--id-*)`, então os antigos fallbacks literais deixaram de ser necessários.

| Token | Definição (claro / escuro) | Consumidor | Estado |
|---|---|---|---|
| `--id-accent-muted` | `identity-tokens.css:142` / `:192` | `.gb-agent-toggle:focus-visible` (`assistant-chat-ui.css:175`) | resolvido |
| `--id-text-secondary` | `:143` / `:193` | `assistant-chat-ui.css:166`, `assistant-drive.css:362` | resolvido |
| `--id-fg` | `:144` / `:194` | `assistant-chat.css:270,289` | resolvido |
| `--id-accent-rgb` | `:145` / `:195`, sincronizado em `accent.js:251-256` | `assistant-drive.css:297` (sobreposição de soltar) | resolvido |
| `--id-accent-bg` | `:146` / `:196` | `assistant-drive.css:369` (aba ativa) | resolvido |
| `--id-border` | `:147` / `:197` | `assistant-blocks.css:295,303`, `assistant-drive.css:356` | resolvido |
| `--id-radius` | `:148` / `:198` | `assistant-blocks.css:287,296,304`, `assistant-drive.css:363` | resolvido |
| `--id-surface-alt` / `--id-surface-hover` | `:149-150` / `:199-200` | `assistant-feedback.css:14,46`, `assistant-drive.css:365` | resolvido |
| `--id-font-mono` | `:151` / `:201` | `assistant-chat-ui.css:223`, `assistant-providers.css:50,111` | resolvido |

**`--id-focus` usado como box-shadow — resolvido.** É um box-shadow (`identity-tokens.css:156`) e todos os consumidores o aplicam com `box-shadow: var(--id-focus)`: `assistant-settings.css:354`, `assistant-providers.css:41,114`, `assistant-garage.css:72,109`, `assistant-blocks.css:40`, `assistant-chat-ui.css:175`, `assistant-sidebar.css:80`, `titlebar.css:46,96`. Não resta `outline: var(--id-focus)`; o anel global usa a cor `var(--id-accent-text)` (`default.css:151-154`).

**Cores fixas fora dos tokens**: `#fff` no quadro do PDF (`assistant-blocks.css:296`); `white` no botão do toggle (`assistant-settings.css:368`) e no texto do botão de feedback (`assistant-feedback.css:28`); o chip "Precisa de você" com `rgb(234 179 8/.25)` em vez de um token de linha de aviso (`assistant-blocks.css:73`); a sombra do painel e o hover do bloco em rgb literal (`assistant-blocks.css:33,229`); o lightbox em `rgb(0 0 0/.5)` em vez de `--id-overlay` (`:325-328`); os tons por extensão de arquivo em JS (`12_blocks.js:89-95`); erros em linha `#dc2626`/`#a16207` (`12_blocks.js:148,336`; `06a_drive_actions.js:204`); o fallback de `--id-warn` é vermelho (`assistant-drive.css:196`); `.gb-gar-soon` traz fallbacks que não existem nos tokens (`assistant-garage.css:184`). A barra de título escura usa rgba fixos e é decidida por `isDark()` na montagem, sem seguir mudanças de tema (`titlebar.css:110-120`, `titlebar.js:39`). Splash e impressão (`assistant-feedback.css:70-93`) ficam fora do sistema de propósito.

**Tamanhos em px que não escalam com o tamanho do texto**: `.gb-badge` 10px (`assistant-settings.css:252`), `.gb-set-bg-name` 10px (`:345`), contador 11px (`:357`), blocos 13/12/11/10px (`assistant-blocks.css:50,52,57,114,119-120,180`), `kbd` 11px (`assistant-chat.css:513`), Drive `0.8125rem` (`assistant-drive.css:362`). O raio do bloco é 12px literal, em vez de `--id-radius-control` (`assistant-blocks.css:24`).

**Documentação no código — resolvida.** O cabeçalho de `identity-tokens.css:1-9` descreve corretamente claro e escuro; não há mais comentário "Light theme only" na camada de tokens, nem bloco escuro duplicado — existe um único `:root[data-color-scheme="dark"]` (`identity-tokens.css:167-227`). A escala de tipos em rem (`:57-66`) está documentada junto com a reescrita em px feita por `textsize.js` (§2.4). Se `assistant.css:2` ainda contiver o comentário antigo, ele pertence à pele (skin) do assistente e não à camada de tokens.

**Acento e papel — resolvidos.** O padrão `--id-on-accent` é `#2C2C2C` (`identity-tokens.css:27`), o mesmo que `derive("#F28C28")` devolve (`accent.js:163-170`): não há mais troca branco→escuro após `apply()`. `--id-paper` tem valor escuro no CSS (`identity-tokens.css:176`); `accent.js` pode refiná-lo para `#1c1c28` quando um acento é aplicado (`accent.js:117-118`). Sem `apply()`, o escuro ainda é correto pelo bloco `data-color-scheme`, sem depender de `restoreAccent()` (`09_settings.js:439-445`).

**Padrões duplicados**: `@keyframes gb-pulse` é definido duas vezes com corpos diferentes (`assistant-chat.css:218` e `:508`; vale o último, o que muda o pulso de "pensando"). Há três mecânicas de acordeão diferentes (§4.6) e três tratamentos de foco (anel global, borda acento sem anel em `.gb-set-input`, halo soft em `.gb-prov-search`). Convivem dois sistemas de ícones (§5). `--gb-field` é definido em `assistant-settings.css:287` e usado em `assistant-providers.css:8`, o que depende da ordem de carga.

**Tamanho de arquivos**: `03_render.js` tem 452 linhas, acima do limite de 450; `02_api.js` está no limite (450); `assistant-chat.css` tem 513 e `assistant.css` 464 (o limite de `AGENTS.md:639` vale para "a single file").
