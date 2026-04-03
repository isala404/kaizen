Phase 1: Auth + Data Model + CRUD + Board UI
Phase 2: Live timer, keyboard shortcuts, editable detail panel
Phase 3: Custom field definitions, task field values, filter bar
Phase 4: Drag-and-drop between columns, mobile swipe gestures
Phase 5: Attachments, undo toast, column collapse, done filter

All phases complete. 12 backend tests passing, forge check green.

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
