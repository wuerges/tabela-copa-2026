# VISION.md

## Nome
**classificados-copa-3o-lugar** — Classificados da Copa do Mundo 2026

## Descrição
Ferramenta CLI + Web App em Rust para acompanhar a Copa do Mundo FIFA 2026. Baixa resultados do dataset openfootball (formato Football.TXT), armazena localmente em JSON, exibe classificações, a chave do mata-mata, e simula cenários para determinar a probabilidade de classificação de cada seleção. A interface web (Leptos CSR / WebAssembly) permite editar placares da fase de grupos e preencher a chave completa do mata-mata até a final.

## Arquitetura
- **CLI** (`copa2026`): comandos de busca, exibição de tabelas, chave do mata-mata e simulação
- **Web App** (Leptos CSR): interface interativa com edição de placares e preenchimento da chave até a final
- **Core** (crate `copa2026-core`): modelos, lógica de classificação, geração da chave, propagação de vencedores, simulação de cenários, posições garantidas

## Funcionalidades

### 1. Buscar e armazenar (`fetch`)
- Baixa o arquivo `cup.txt` do repositório [openfootball/worldcup](https://github.com/openfootball/worldcup) (formato Football.TXT)
- Faz parse dos grupos, times e resultados (placar no formato `X-Y (A-B)`)
- Mapeia nomes de times para códigos FIFA de 3 letras (com fallback para prefixos e mapeamento manual para casos ambíguos como Austria/Australia)
- Armazena os dados localmente em `data.json`

### 2. Exibir classificações (`standings` / Web)
- Tabela de cada grupo: posição, time, P, J, V, E, D, GP, GC, SG
- Destaque para 1º e 2º lugares (classificados diretos ao mata-mata)
- Ranking separado dos **8 melhores 3º lugares**

### 3. Chave do mata-mata (`bracket` / Web)
- Árvore completa (R32 → R16 → QF → SF → Final → 3º Lugar) com layout CSS Grid de 6 colunas equidistantes
- Preenchimento automático dos confrontos com base nos resultados da fase de grupos
- **Propagação round-by-round**: selecionar vencedores em qualquer fase propaga o time para todas as fases seguintes (inclusive Final e 3º Lugar simultaneamente)
- **Distinção visual** no R32: times com posição garantida (verde) vs. incertos que podem mudar (amarelo itálico), via `clinched_positions()`
- **Três estados por nó**: finished (placar definido, borda verde), determined (times definidos sem placar, borda azul), pending (slot vazio, opaco)
- Vencedor destacado em verde, perdedor em cinza
- Slots pendentes com labels de referência (ex: W1, L29)
- Times com nomes longos truncados com ellipsis para manter largura fixa das colunas

### 4. Simulação de classificação (`guaranteed-thirds` / Web)
- Identifica jogos restantes da fase de grupos (sem resultado)
- Simula via **Monte Carlo** (50.000 cenários) ou enumeração exaustiva (< 100.000 cenários)
- Para cada time, breakdown completo de probabilidade:
  - **1o%** — chance de terminar em 1º no grupo
  - **2o%** — chance de terminar em 2º no grupo
  - **3o%** — chance de terminar em 3º E ficar entre os 8 melhores
  - **Total%** — probabilidade total de avançar ao mata-mata
- Tabela ordenada por Total% decrescente com distinção visual: garantidos (linha verde), incertos, desqualificados (linha opaca riscada em vermelho)
- Badges coloridos no rodapé: verde (garantidos), azul (incertos), vermelho (desqualificados)

### 5. Posições garantidas (`clinched_positions`)
- Para cada grupo, enumera exaustivamente todas as permutações dos jogos pendentes (3^n, max 729 por grupo)
- Identifica quais posições (1º, 2º, 3º) estão matematicamente travadas
- Usado na interface web para destacar times com posição garantida na chave do mata-mata

### 6. Edição interativa (Web)
- Placar dos jogos da fase de grupos:
  - **Inputs numéricos** para partidas sem resultado original
  - **Inputs numéricos** para partidas cujo placar foi digitado manualmente pelo usuário (rastreadas via `user_edited`)
  - **Texto estático** para partidas com resultado vindo do dataset (originais, não editáveis)
- **Preenchimento da chave**: clique no nome do time para declará-lo vencedor (1-0)
- Vencedores propagam automaticamente para as fases seguintes
- É possível trocar o vencedor a qualquer momento clicando no outro time
- Times podem ser selecionados mesmo com adversário indefinido (preenchimento antecipado da chave)

### 7. Estatísticas (`stats`)
- Resumo geral: total de jogos, realizados vs. restantes, total de gols, empates

## Stack Técnica

### CLI
- **Linguagem:** Rust (stable, edition 2024)
- **CLI framework:** `clap` (derive API)
- **HTTP client:** `reqwest` + `tokio`
- **Parsing:** `regex-lite` para identificar placares no formato Football.TXT
- **Tabelas:** `comfy-table`

### Core (crate compartilhado)
- **Modelos:** `Team`, `GroupCode`, `Match`, `MatchResult`, `Standing`, `Bracket`, `BracketSlot`, `KnockoutResult`, `TeamQualificationChance`, `ThirdPlaceSimulation`
- **Lógica:** `calculate_standings`, `group_standings`, `rank_third_places`, `generate_bracket`, `apply_knockout_results`, `simulate_guaranteed_thirds`, `clinched_positions`
- **Serialização:** `serde` / `serde_json`

### Web App (Leptos CSR)
- **Framework:** Leptos 0.8 (client-side rendering, WASM)
- **Roteamento:** `leptos_router`
- **Requisições HTTP:** `gloo-net` (fetch API do navegador)
- **Build tool:** `trunk` (compilação para WebAssembly, hook `pre_build` no `Trunk.toml` para copiar `data.json` e `style.css`)
- **CSS:** `static/style.css` — tema dark, CSS Grid para layout do mata-mata, responsivo
- **Deploy:** Vercel (via `vercel.json`, build com trunk, output estático em `crates/web/dist/`)

## Estrutura do Projeto
```
copa2026/
├── Cargo.toml               # workspace
├── VISION.md
├── README.md
├── Justfile                 # comandos rápidos (just fetch, just web-dev, ...)
├── vercel.json              # configuração de deploy Vercel
├── data.json                # dados baixados pelo fetch
├── static/
│   └── style.css            # CSS para o web app
├── crates/
│   ├── core/                # modelos + lógica de negócio
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models.rs
│   │   │   ├── standings.rs
│   │   │   ├── bracket.rs
│   │   │   └── simulation.rs
│   │   └── tests/
│   │       └── integration_test.rs
│   ├── cli/                 # binário CLI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── fetch.rs
│   │       └── display.rs
│   └── web/                 # web app Leptos CSR
│       ├── Cargo.toml
│       ├── Trunk.toml       # hook pre_build para copiar assets
│       ├── index.html       # shell HTML + copy-file directives
│       └── src/
│           ├── main.rs      # entry point (mount_to_body)
│           ├── app.rs       # componente raiz + router
│           └── pages/
│               ├── mod.rs
│               ├── bracket.rs       # fase de grupos editável + mata-mata interativo
│               └── guaranteed_thirds.rs  # tabela de probabilidades
```

## Exemplos de Uso

### CLI
```bash
just fetch                  # baixar resultados do openfootball
just standings              # todos os grupos + ranking 3os lugares
just standings-group A      # apenas grupo A
just best-thirds            # ranking dos 3os lugares
just bracket                # chave do mata-mata
just guaranteed-thirds      # simulação de probabilidades
just stats                  # estatísticas gerais
```

### Web App
```bash
just web-dev                # servidor dev com hot reload (http://localhost:8080)
just web-build              # build produção (dist/)
just web-serve              # servir build local (http://localhost:8080)
```

### Testes
```bash
just test                   # roda os 10 testes do core
```
