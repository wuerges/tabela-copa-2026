# ── CLI ──────────────────────────────────────────

# Fetch latest data from openfootball
fetch:
    cargo run -p copa2026-cli -- fetch

# Show all group standings
standings:
    cargo run -p copa2026-cli -- standings

# Show specific group standings (e.g. just standings A)
standings-group group:
    cargo run -p copa2026-cli -- standings --group {{group}}

# Show third place ranking
best-thirds:
    cargo run -p copa2026-cli -- best-thirds

# Show knockout bracket
bracket:
    cargo run -p copa2026-cli -- bracket

# Run qualification probability simulation
guaranteed-thirds:
    cargo run -p copa2026-cli -- guaranteed-thirds

# Show statistics
stats:
    cargo run -p copa2026-cli -- stats

# ── Build ────────────────────────────────────────

# Build CLI in release mode
build:
    cargo build --release -p copa2026-cli

# Build everything (CLI + web)
build-all: build web-build

# Check compilation without building
check:
    cargo check --workspace

# Run all tests
test:
    cargo test -p copa2026-core

# Clean build artifacts
clean:
    cargo clean

# ── Web ──────────────────────────────────────────

# Start web dev server with hot reload
web-dev:
    cd crates/web && trunk serve

# Build web app for production/Vercel
web-build:
    cd crates/web && trunk build --release

# Serve built web app locally (run web-build first)
web-serve:
    cd crates/web/dist && python3 -m http.server 8080

# ── Default ──────────────────────────────────────

default:
    @just --list
