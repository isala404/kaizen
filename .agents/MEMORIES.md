Tooling
- Stack: Rust (edition 2024, rust-version 1.92), Forge framework (forgex 0.8.1)
- Frontend: Dioxus 0.7.3 (WASM/web/desktop/iOS), forge-dioxus 0.8.1
- DB: PostgreSQL 18 (via docker-compose, port 5432 exposed)
- Build: cargo, dx (dioxus-cli) for frontend release builds
- Observability: OpenTelemetry via custom otel-lgtm image
- Dev: docker compose up for full stack, cargo watch for hot reload
- SQLx offline mode for CI builds (.sqlx/ cache)
- Tests: TEST_DATABASE_URL=postgres://postgres:forge@localhost:5432/kaizen

Project Shape
- Personal task management app with focus timer
- Auth: JWT HS256, register/login/refresh/logout + get_me
- Models: User (pub(crate) to avoid password_hash leak), Task with TaskStatus enum
- Functions: auth.rs (5 handlers), tasks.rs (8 handlers including focus/unfocus/reorder)
- Frontend: Login page, Dashboard with horizontal focus dock + 4-column board, detail panel
- Single layout: board + focus dock everywhere, responsive via CSS
- Removed: mobile-layout (StatusTabs, TaskList, SwipeableCard), drag-and-drop, field filter bar, field pills on cards, time filter pills, play/pause buttons
- Focus dock: click card to toggle focus (start/pause timer), equal-width cards
- Board cards: title + contextual subtitle + optional color bar from enum fields
- Detail panel: no status section, fields/due/time/description only
- `auto_register()` enabled in main.rs
- Build.rs handles frontend: placeholder in dev, dx build in release

Patterns
- forge.toml uses env var substitution: ${DATABASE_URL}, ${FORGE_OTEL_ENABLED-false}
- docker-compose runs backend with --no-default-features (no embedded frontend in dev)
- Gateway on port 9081, frontend dev server on 9080
- CORS configured for localhost:9080 and 127.0.0.1:9080
- Don't use #[forge::model] on structs, it strips derives. Use plain #[derive(sqlx::FromRow, Serialize, Deserialize)]
- wasm-only crates (js-sys, web-sys, gloo-timers, wasm-bindgen) under [target.'cfg(target_arch = "wasm32")'.dependencies], code gated with #[cfg(target_arch = "wasm32")]
- dioxus-sdk 0.7: time (sleep, use_interval) and storage (use_persistent) for cross-platform
- time_utils.rs uses chrono instead of js_sys::Date
- rand on wasm needs both getrandom 0.2 (js feature) and getrandom 0.3 (wasm_js feature)
- DragData::data_transfer() is wasm-only, gate behind cfg (dragging_id signal handles state on all platforms)
- set_dir!() in main.rs initializes storage directory for non-wasm targets
- TestMutationContext doesn't have conn() or issue_token_pair(), can't call handler functions directly in unit tests. Test data layer with pool directly, test handlers via Playwright
- Dioxus Callback<T> is Copy, don't use .clone() on it (clippy error)
- Use String::new instead of || String::new() in use_signal (clippy redundant_closure)
- StatusChange struct needed because Dioxus EventHandler doesn't support tuples well

Domain
- kaizen: personal task manager with focus timer, dark mode only
- Status flow: inbox -> up_next -> in_progress -> focused -> paused -> done -> archived
- Only one focused task per user at a time
- Timer: time_spent_secs + (now - updated_at) when focused
- Position gaps: 10,000 between tasks for insertion
