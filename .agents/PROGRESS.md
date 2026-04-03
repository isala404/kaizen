Phase 1 complete: Auth + Data Model + CRUD + Board UI
- Migration 0001: users table, task_status enum, tasks table with reactivity
- Auth: register, login, refresh, logout, get_me (unscoped since user IS the row)
- Tasks: list, get, create, update, delete, focus, unfocus, reorder
- Frontend: login page, dashboard with 5-column board, focus dock, detail panel, mobile layout
- 9 unit tests passing (data layer tests via pool, not handler tests)
- WORKAROUND: #[forge::model] strips derives, use plain #[derive(sqlx::FromRow, Serialize, Deserialize)]
- WORKAROUND: TestMutationContext can't call handlers, unit tests hit DB directly

Phase 2 complete: Interactivity
- Live h:mm:ss timer on focused task (1s tick via gloo_timers + use_future)
- Relative time on task cards ("just now", "5m ago", "2h ago")
- time_utils.rs module with shared formatting functions
- Quick capture via `n` key with auto-focus
- Keyboard shortcuts: 1-5 (column), j/k (navigate), Enter (open detail), Space (focus), Backspace (unfocus), Escape (close)
- Editable detail panel: title (input with blur save), description (click-to-edit textarea), due date (date input)
- on_update EventHandler added to DetailPanel for UpdateTaskInput
- Keyboard nav signals: focused_col, focused_row with CSS highlight
- tasks_sig signal to share task data with keyboard handler closure
- Added js-sys, web-sys, gloo-timers to frontend deps

- TODO: Phase 3 - field definitions + task fields
- TODO: Phase 4 - drag-and-drop + touch gestures
- TODO: Phase 5 - attachments + polish
