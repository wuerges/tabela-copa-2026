# VISION.md

## Nome
**classificados-copa-3o-lugar** — Classificados da Copa do Mundo 2026

## Descrição
Ferramenta CLI em Rust que busca os resultados da fase de grupos da Copa do Mundo FIFA 2026 via API, armazena localmente, e exibe classificações, o ranking dos melhores 3º lugares, a chave do mata-mata, e simula cenários para determinar quais 3º colocados já estão **matematicamente garantidos**.

## Funcionalidades

### 1. Buscar e armazenar (`fetch`)
- Consultar uma API pública (ex: OpenLigaDB, API-Football) para obter resultados dos jogos da fase de grupos
- Armazenar os dados localmente em formato JSON/SQLite para consulta offline
- Permitir atualização incremental (buscar apenas novos resultados)

### 2. Exibir classificações (`standings`)
- Mostrar tabela de cada grupo: posição, time, P, J, V, E, D, GP, GC, SG
- Destacar 1º e 2º lugares (classificados diretos ao mata-mata)
- Exibir ranking separado dos **8 melhores 3º lugares**

### 3. Chave do mata-mata (`bracket`)
- Exibir a chave completa em formato de árvore/colchetes
- Indicar confrontos já definidos com base nos resultados
- Mostrar cruzamentos conforme regulamento FIFA 2026

### 4. Simulação — 3º lugares garantidos (`guaranteed-thirds`)
- Identificar jogos restantes da fase de grupos (sem resultado definido)
- Simular **todas as permutações** possíveis desses jogos (3^n combinações: vitória mandante, empate, vitória visitante)
- Para cada cenário, recalcular a tabela dos grupos e o ranking dos 3º lugares
- Listar os 3º colocados que aparecem entre os **8 melhores em todos os cenários** (classificação matematicamente garantida)
- Mostrar quais ainda dependem de resultados e em quantos cenários se classificam

### 5. Estatísticas (`stats`)
- Resumo geral: total de jogos, gols, aproveitamento
- Jogos realizados vs. restantes
- Estatísticas por seleção

## Stack Técnica
- **Linguagem:** Rust (stable)
- **CLI framework:** `clap` (derive API) para argumentos e subcomandos
- **HTTP client:** `reqwest` com `tokio` para chamadas assíncronas à API
- **Armazenamento local:** `serde_json` (arquivos JSON) ou `rusqlite` (SQLite)
- **Tabelas formatadas:** `comfy-table` ou `tabled`
- **Simulação:** Algoritmo de permutação com `itertools` ou paralelismo via `rayon` para acelerar a varredura de cenários

## Exemplos de Uso
```bash
# Buscar resultados mais recentes
copa2026 fetch

# Ver classificacao de um grupo
copa2026 standings --group A

# Ver todos os grupos
copa2026 standings

# Ver ranking dos 8 melhores 3os lugares
copa2026 best-thirds

# Simular e listar 3os lugares ja garantidos
copa2026 guaranteed-thirds

# Ver chave do mata-mata
copa2026 bracket

# Resumo geral de estatisticas
copa2026 stats
```
