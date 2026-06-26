# VISION.md

## Nome
**classificados-copa-3o-lugar** — Classificados da Copa do Mundo 2026

## Descrição
Ferramenta CLI + Web App em Rust que busca os resultados da fase de grupos da Copa do Mundo FIFA 2026 via API, armazena localmente, exibe classificações, simula os 3º lugares garantidos, e oferece uma interface web interativa (Leptos) para editar resultados e visualizar a chave do mata-mata em tempo real.

## Arquitetura
- **CLI** (`copa2026`): comandos de busca, exibição de tabelas e estatísticas
- **Web App** (`copa2026 serve`): interface Leptos com chave do mata-mata interativa e simulador de 3º lugares

## Funcionalidades

### 1. Buscar e armazenar (`fetch`)
- Consultar uma API pública (ex: OpenLigaDB, API-Football) para obter resultados dos jogos da fase de grupos
- Armazenar os dados localmente em formato JSON/SQLite para consulta offline e servidos ao frontend
- Permitir atualização incremental (buscar apenas novos resultados)

### 2. Exibir classificações (`standings`)
- Mostrar tabela de cada grupo: posição, time, P, J, V, E, D, GP, GC, SG
- Destacar 1º e 2º lugares (classificados diretos ao mata-mata)
- Exibir ranking separado dos **8 melhores 3º lugares**

### 3. Chave do mata-mata — CLI (`bracket`)
- Exibir a chave completa em formato de árvore/colchetes no terminal
- Indicar confrontos já definidos com base nos resultados
- Mostrar cruzamentos conforme regulamento FIFA 2026

### 4. Simulação — 3º lugares garantidos — CLI (`guaranteed-thirds`)
- Identificar jogos restantes da fase de grupos (sem resultado definido)
- Simular **todas as permutações** possíveis desses jogos (3^n combinações: vitória mandante, empate, vitória visitante)
- Para cada cenário, recalcular a tabela dos grupos e o ranking dos 3º lugares
- Listar os 3º colocados que aparecem entre os **8 melhores em todos os cenários** (classificação matematicamente garantida)
- Mostrar quais ainda dependem de resultados e em quantos cenários se classificam

### 5. Servidor web interativo (`serve`)
- Inicia um servidor HTTP local com a aplicação Leptos
- Rota principal: chave do mata-mata (`/bracket`)
- Rota de simulação: 3º lugares garantidos (`/guaranteed-thirds`)

### 6. Chave do mata-mata — Web (`/bracket`)
- Exibir a chave completa do mata-mata no formato de árvore/colchetes com renderização no navegador
- Mostrar a **tabela de todos os jogos da fase de grupos** (já realizados e pendentes)
- Permitir **clicar em um jogo e alterar seu resultado** (placar, vencedor)
- Recalcular automaticamente a chave do mata-mata com base no resultado alterado
- Indicar visualmente confrontos definidos vs. dependentes de resultados futuros
- Cruzamentos conforme regulamento FIFA 2026

### 7. Simulação — 3º lugares garantidos — Web (`/guaranteed-thirds`)
- Identificar jogos restantes da fase de grupos
- Simular **todas as permutações** possíveis (3^n: vitória mandante, empate, vitória visitante)
- Para cada cenário, recalcular tabelas e ranking dos 3º lugares
- Exibir na interface:
  - Lista dos 3º colocados **matematicamente garantidos** (entre os 8 melhores em 100% dos cenários)
  - Lista dos que ainda dependem de resultados, com percentual de cenários favoráveis
  - Permitir "travar" resultados de alguns jogos e refazer a simulação com essas restrições
- Paralelismo via `rayon` no backend (simulação pesada roda no servidor, não no WASM)

### 8. Estatísticas (`stats`)
- Resumo geral via CLI: total de jogos, gols, aproveitamento
- Jogos realizados vs. restantes
- Estatísticas por seleção

## Stack Técnica

### CLI
- **Linguagem:** Rust (stable)
- **CLI framework:** `clap` (derive API)
- **HTTP client:** `reqwest` + `tokio`

### Backend / Servidor
- **Web framework:** `axum` ou `actix-web` (servindo a aplicação Leptos com SSR/hydration)
- **Armazenamento:** `serde_json` (JSON) ou `rusqlite` (SQLite)
- **Simulação:** `rayon` para paralelismo de cenários

### Frontend (Web)
- **Framework:** **Leptos** (Rust WASM, reativo)
- **Estilo:** `tailwindcss` ou CSS simples
- **Componentes reativos:** tabela de jogos editável, chave do mata-mata dinâmica, ranking de 3º lugares

### Compartilhado (crate `core`)
- Modelos de domínio: `Group`, `Match`, `Team`, `Standing`, `Bracket`
- Lógica de negócio: cálculo de classificação, geração da chave, simulação de cenários
- Compartilhado entre CLI e Web App

## Estrutura de Crates (workspace)
```
copa2026/
├── Cargo.toml          # workspace
├── VISION.md
├── crates/
│   ├── core/           # modelos + lógica de negócio
│   ├── cli/            # binário CLI (fetch, standings, stats)
│   └── web/            # binário servidor Leptos (serve)
```

## Exemplos de Uso

### CLI
```bash
# Buscar resultados mais recentes
copa2026 fetch

# Ver classificacao de um grupo
copa2026 standings --group A

# Ver todos os grupos
copa2026 standings

# Ver chave do mata-mata
copa2026 bracket

# Simular 3os lugares garantidos
copa2026 guaranteed-thirds

# Resumo geral
copa2026 stats
```

### Web App
```bash
# Iniciar servidor web interativo
copa2026 serve

# Abrir no navegador:
# http://localhost:3000/bracket        -> chave do mata-mata + edicao de resultados
# http://localhost:3000/guaranteed-thirds -> simulador de 3os lugares
```
