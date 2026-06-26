# VISION.md

## Nome
**classificados-copa-3o-lugar** — Classificados da Copa do Mundo 2026

## Descrição
Ferramenta CLI em Rust que baixa os resultados da fase de grupos da Copa do Mundo FIFA 2026 do dataset openfootball (formato Football.TXT), armazena localmente em JSON, e exibe classificações, a chave do mata-mata, e simula cenários para determinar a probabilidade de classificação de cada seleção.

## Arquitetura
- **CLI** (`copa2026`): comandos de busca, exibição de tabelas, chave do mata-mata e simulação
- **Web App** (crate `web`): placeholder — aplicação Leptos a ser implementada

## Funcionalidades

### 1. Buscar e armazenar (`fetch`)
- Baixa o arquivo `cup.txt` do repositório [openfootball/worldcup](https://github.com/openfootball/worldcup) (formato Football.TXT)
- Faz parse dos grupos, times e resultados (placar no formato `X-Y (A-B)`)
- Mapeia nomes de times para códigos FIFA de 3 letras (com fallback para prefixos)
- Armazena os dados localmente em `data.json`

### 2. Exibir classificações (`standings`)
- Mostrar tabela de cada grupo: posição, time, P, J, V, E, D, GP, GC, SG
- Destacar 1º e 2º lugares (classificados diretos ao mata-mata)
- Exibir ranking separado dos **8 melhores 3º lugares**

### 3. Chave do mata-mata (`bracket`)
- Exibir a chave completa (Round of 32 → Final) no terminal
- Preencher confrontos já definidos com base nos resultados atuais
- Indicar slots ainda pendentes com labels (ex: W1, W2)

### 4. Simulação de classificação (`guaranteed-thirds`)
- Identificar jogos restantes da fase de grupos (sem resultado)
- Simular via **Monte Carlo** (50.000 cenários) ou enumeração exaustiva (até 100.000 cenários)
- Para cada time, mostrar a probabilidade total de classificação com breakdown:
  - **1o%** — chance de terminar em 1º no grupo
  - **2o%** — chance de terminar em 2º no grupo
  - **3o%** — chance de terminar em 3º E ficar entre os 8 melhores 3º lugares
  - **Total%** — soma das três (probabilidade total de avançar ao mata-mata)
- Times 100% garantidos aparecem em verde, desqualificados (0%) em vermelho
- Resumo final: X garantidos, Y incertos, Z desqualificados

### 5. Estatísticas (`stats`)
- Resumo geral: total de jogos, jogos realizados vs. restantes, total de gols, empates

## Stack Técnica

### CLI
- **Linguagem:** Rust (stable, edition 2024)
- **CLI framework:** `clap` (derive API)
- **HTTP client:** `reqwest` + `tokio`
- **Parsing:** `regex-lite` para identificar placares no formato Football.TXT
- **Tabelas:** `comfy-table`

### Core (crate compartilhado)
- **Modelos:** `Team`, `Group`, `Match`, `MatchResult`, `Standing`, `Bracket`
- **Lógica:** cálculo de classificação (pontos, saldo de gols), ranking de 3º lugares, geração da chave do mata-mata, simulação Monte Carlo
- **Serialização:** `serde` / `serde_json`

### Web App (placeholder)
- **Framework planejado:** Leptos + Axum
- **CSS:** `static/style.css` (tema dark, pronto para uso)
- **Estado atual:** binário placeholder que exibe mensagem de "não implementado"

## Estrutura do Projeto
```
copa2026/
├── Cargo.toml          # workspace
├── VISION.md
├── README.md
├── data.json           # dados baixados pelo fetch
├── static/
│   └── style.css       # CSS para o futuro web app
├── crates/
│   ├── core/           # modelos + lógica de negócio
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models.rs
│   │       ├── standings.rs
│   │       ├── bracket.rs
│   │       └── simulation.rs
│   ├── cli/            # binário CLI
│   │   └── src/
│   │       ├── main.rs
│   │       ├── fetch.rs
│   │       └── display.rs
│   └── web/            # placeholder (Leptos)
│       └── src/main.rs
```

## Exemplos de Uso
```bash
# Baixar resultados do openfootball
copa2026 fetch

# Ver classificacao de um grupo
copa2026 standings --group A

# Ver todos os grupos + ranking 3os lugares
copa2026 standings

# Ver apenas o ranking dos 3os lugares
copa2026 best-thirds

# Ver chave do mata-mata
copa2026 bracket

# Simular probabilidades de classificacao
copa2026 guaranteed-thirds

# Resumo de estatisticas
copa2026 stats
```
