Phase 1: Auth + Data Model + CRUD + Board UI
Phase 2: Live timer, keyboard shortcuts, editable detail panel
Phase 3: Custom field definitions, task field values, filter bar
Phase 4: Drag-and-drop between columns, mobile swipe gestures
Phase 5: Attachments, undo toast, column collapse, done filter

All phases complete. 12 backend tests passing, forge check green.

UI rebuild: horizontal focus dock, simplified board cards, removed DnD/filters/play buttons
- Removed: drag-and-drop, FieldFilterBar, FieldPills, TimeFilter, play/pause buttons, status section in detail panel
- Focus dock cards: click to toggle focus, equal width via flex: 1 1 0, live seconds on focused card
- Board cards: contextual subtitles (Added Xh ago / description / Paused Xh ago), color bar from enum fields
- Adding tasks: inline "+" in each column creates task with that column's status
- Dead files left on disk: field_filter_bar.rs, field_pills.rs (unreachable, not compiled)

Cross-platform + single layout
- Merged desktop-layout and mobile-layout into single responsive layout (board everywhere)
- Removed StatusTabs, TaskList, SwipeableCard components (mod declarations removed, files on disk)
- Moved js-sys/web-sys/gloo-timers/wasm-bindgen to target-specific deps, gated code with #[cfg(target_arch = "wasm32")]
- Replaced js_sys::Date with chrono, js_sys::Math::random with rand, localStorage with storage.rs helper
- Adopted dioxus-sdk 0.7 for cross-platform sleep/timers and persistent storage (file-backed on native, localStorage on web)
- highlight.js syntax highlighting skipped on native (markdown still renders via pulldown-cmark)

Key decisions:
- #[forge::model] strips derives, use plain #[derive(sqlx::FromRow, Serialize, Deserialize)]
- TestMutationContext can't call handlers, unit tests hit DB directly
- Dioxus Callback<T> is Copy, don't .clone()
- StatusChange struct for EventHandler (tuples don't work well)
- DragPayload/DropTarget structs for DnD state
- Undo toast delays delete by 5s, clearing undo_action cancels the delete
- Column collapse state persisted in localStorage
- Done column filters to today by default with "Show previous" toggle
- Empty hints hidden after 5+ total tasks
