# DevVault task runner. Requires `just` (https://github.com/casey/just).
set shell := ["bash", "-cu"]
export DATABASE_URL := env_var_or_default("DATABASE_URL", "postgres://devvault:devpass@localhost:5432/devvault")

# List recipes
default:
    @just --list

# Start everything (db + api + web) → http://localhost:8080
up:
    docker compose up --build -d
    @echo ""
    @echo "  DevVault  →  http://localhost:8080"
    @echo "  Swagger   →  http://localhost:8080/docs"
    @echo ""

# Stop everything (keeps your data)
down:
    docker compose down

# Local development: db + api in Docker, Vite dev server with hot reload on :5173
dev:
    docker compose up -d db api
    cd frontend && pnpm install && pnpm dev

# Backend: format check + lint + tests (what CI runs)
check:
    cd backend && cargo fmt --all --check
    cd backend && cargo clippy --all-targets --all-features -- -D warnings
    cd backend && cargo test

# Frontend: lint + typecheck + production build
check-web:
    cd frontend && pnpm install && pnpm lint && pnpm typecheck && pnpm build

# Run backend tests
test:
    cd backend && cargo test

# Apply migrations (local dev)
migrate:
    cd backend && sqlx migrate run

# Regenerate sqlx offline metadata (run after changing SQL)
prepare:
    cd backend && cargo sqlx prepare

# Stop everything and delete the data volume
clean:
    docker compose down -v
