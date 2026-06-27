# classificados-copa-3o-lugar

Classificados da Copa do Mundo FIFA 2026 — ferramenta CLI em Rust para acompanhar a fase de grupos, o ranking dos melhores 3º lugares, a chave do mata-mata e simular os 3º lugares garantidos.

## Instalação

```bash
cargo build --release -p copa2026-cli
```

O binário fica em `target/release/copa2026`.

## Uso

```bash
# Buscar resultados da API openfootball
copa2026 fetch

# Ver classificação de todos os grupos
copa2026 standings

# Ver apenas um grupo
copa2026 standings --group A

# Ver ranking dos 8 melhores 3º lugares
copa2026 best-thirds

# Ver chave do mata-mata
copa2026 bracket

# Simular 3º lugares garantidos (Monte Carlo)
copa2026 guaranteed-thirds

# Resumo de estatísticas
copa2026 stats
```

## Estrutura do Projeto

```
copa2026/
├── Cargo.toml          # workspace
├── data.json           # dados de exemplo
├── crates/
│   ├── core/           # modelos + lógica (standings, bracket, simulation)
│   ├── cli/            # binário CLI (fetch, standings, bracket, ...)
│   └── web/            # web app Leptos (a implementar)
```

## Funcionalidades

- **Tabelas de grupo** com destaque para 1º e 2º lugares (classificados diretos)
- **Ranking dos 3º lugares** com os 8 melhores avançando ao mata-mata
- **Chave completa** do mata-mata (Round of 32 até a Final)
- **Simulação combinatória** dos jogos pendentes para identificar 3º lugares matematicamente garantidos
- **Busca de dados** via API openfootball (resultados reais da Copa 2026)
- **Web app** (Leptos CSR / WebAssembly) com edição interativa de placares e chave do mata-mata
