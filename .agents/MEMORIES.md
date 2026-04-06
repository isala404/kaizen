Tooling
- Stack: Rust 1.92, Forge framework, Dioxus 0.7.3 (WASM frontend)
- Database: PostgreSQL (shared TimescaleDB instance in k8s)
- Auth: JWT HS256 via Forge built-in auth
- Build: Multi-stage Dockerfile (frontend-builder → backend-builder → runtime)
- Production binary: Single executable with embedded frontend (rust-embed)
- Package manager: cargo
- Dev: `docker compose up --build` (backend + pg + otel)
- Frontend dev: `cd frontend && dx serve`

Deployment
- Hosted at: kaizen.tallisa.dev
- Registry image: registry.tallisa.dev/tallisa/kaizen
- CI: GitHub Actions, OIDC auth to registry (no stored secrets)
- CD: FluxCD image automation in cumulus-gitops repo (v2 branch)
- Tag pattern: prod-<run_id>
- Namespace: kaizen
- Port: 9081 (forge.toml gateway.port, Dockerfile EXPOSE says 8080 but that's wrong)

Runtime Env Vars
- DATABASE_URL: postgres connection string (managed via Bitwarden + ESO)
- JWT_SECRET: signing key for auth tokens (managed via Bitwarden + ESO)
- FORGE_OTEL_ENABLED: observability toggle (not set in prod currently)
- Forge auto-runs migrations from /app/migrations on startup

Patterns
- forge.toml uses ${VAR} syntax for env var interpolation
- SQLx offline mode for compile-time query checking (sqlx.toml + .sqlx/)
- Frontend uses Dioxus with forge-dioxus bindings for live subscriptions (SSE)
