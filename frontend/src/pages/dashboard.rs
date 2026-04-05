use std::collections::HashMap;

use dioxus::prelude::*;

use crate::components::{
    Board, DetailPanel, DropTarget, FieldManager, FocusDock, Header, QuickCapture,
    UndoAction, UndoToast,
};
use crate::forge::{
    CreateTaskInput, DeleteTaskInput, FieldDefinition, FocusTaskInput, ReorderTaskInput, Task,
    TaskField, TaskStatus, UnfocusTaskInput, UpdateTaskInput, use_create_task, use_delete_task,
    use_focus_task, use_list_all_task_fields_live, use_list_field_definitions_live,
    use_list_tasks_live, use_reorder_task, use_unfocus_task, use_update_task,
};

const COLUMN_STATUSES: [TaskStatus; 4] = [
    TaskStatus::Inbox,
    TaskStatus::UpNext,
    TaskStatus::Paused,
    TaskStatus::Done,
];

#[component]
pub fn Dashboard() -> Element {
    let tasks_state = use_list_tasks_live();

    let create = use_create_task();
    let delete = use_delete_task();
    let focus = use_focus_task();
    let unfocus = use_unfocus_task();
    let update = use_update_task();
    let reorder = use_reorder_task();

    let mut selected_task_id = use_signal(|| Option::<String>::None);
    let mut show_capture = use_signal(|| false);
    let mut dragging_id = use_signal(|| Option::<String>::None);
    let mut undo_action = use_signal(|| Option::<UndoAction>::None);
    let mut focused_col = use_signal(|| Option::<usize>::None);
    let mut focused_row = use_signal(|| Option::<usize>::None);

    // Live timer tick
    let mut tick = use_signal(|| 0u64);
    dioxus_sdk::time::use_interval(std::time::Duration::from_secs(1), move |()| {
        tick.set(tick() + 1);
    });

    let field_defs_state = use_list_field_definitions_live();
    let task_fields_state = use_list_all_task_fields_live();
    let mut show_field_manager = use_signal(|| false);

    let field_defs: Vec<FieldDefinition> = field_defs_state.data.clone().unwrap_or_default();
    let all_task_fields: Vec<TaskField> = task_fields_state.data.clone().unwrap_or_default();

    // Pending moves: task_id → (new_status, new_position).
    // Applied on top of server data so optimistic updates survive
    // even when the server response hasn't arrived yet.
    let mut pending_moves = use_signal(HashMap::<String, (TaskStatus, i32)>::new);

    let tasks: Vec<Task> = {
        let mut base = tasks_state.data.clone().unwrap_or_default();
        let moves = pending_moves.read();
        for t in &mut base {
            if let Some((status, position)) = moves.get(&t.id) {
                t.status = status.clone();
                t.position = *position;
            }
        }
        base
    };

    let mut tasks_sig = use_signal(Vec::<Task>::new);
    *tasks_sig.write() = tasks.clone();

    let mut dock_tasks: Vec<Task> = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Focused || t.status == TaskStatus::InProgress)
        .cloned()
        .collect();
    dock_tasks.sort_by_key(|t| t.position);

    let _tick_val = tick();

    let on_create = {
        let create = create.clone();
        move |(title, status): (String, TaskStatus)| {
            let create = create.clone();
            spawn(async move {
                let _ = create
                    .call(CreateTaskInput::new(title).status(status))
                    .await;
            });
        }
    };

    let on_delete = {
        let delete = delete.clone();
        let tasks_for_delete = tasks.clone();
        move |id: String| {
            let task_title = tasks_for_delete
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.title.clone())
                .unwrap_or_default();

            undo_action.set(Some(UndoAction {
                label: format!("\"{}\" deleted", task_title),
                task_id: id.clone(),
            }));

            let delete = delete.clone();
            spawn(async move {
                dioxus_sdk::time::sleep(std::time::Duration::from_secs(5)).await;
                let current = undo_action.read().clone();
                if current.as_ref().is_some_and(|a| a.task_id == id) {
                    undo_action.set(None);
                    let _ = delete.call(DeleteTaskInput::new(id)).await;
                }
            });
        }
    };

    let on_focus = {
        let focus = focus.clone();
        move |id: String| {
            let focus = focus.clone();
            spawn(async move {
                let _ = focus.call(FocusTaskInput::new(id)).await;
            });
        }
    };

    let on_unfocus = {
        let unfocus = unfocus.clone();
        move |id: String| {
            let unfocus = unfocus.clone();
            spawn(async move {
                let _ = unfocus.call(UnfocusTaskInput::new(id)).await;
            });
        }
    };

    let on_update = {
        let update = update.clone();
        move |input: UpdateTaskInput| {
            let update = update.clone();
            spawn(async move {
                let _ = update.call(input).await;
            });
        }
    };

    let on_drag_start = move |id: String| {
        dragging_id.set(Some(id));
    };

    let on_drag_end = move |_: ()| {
        dragging_id.set(None);
    };

    // Touch drag state
    let mut touch_pos = use_signal(|| Option::<(f64, f64)>::None);
    let mut touch_dragging_title = use_signal(|| Option::<String>::None);

    let on_touch_drag_start = {
        move |(id, x, y): (String, f64, f64)| {
            let title = tasks_sig
                .read()
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.title.clone())
                .unwrap_or_default();
            touch_dragging_title.set(Some(title));
            touch_pos.set(Some((x, y)));
            dragging_id.set(Some(id));
        }
    };

    let on_touch_drag_move = {
        move |(x, y): (f64, f64)| {
            touch_pos.set(Some((x, y)));
        }
    };

    let on_touch_drag_end = {
        move |(x, y): (f64, f64)| {
            touch_pos.set(None);
            touch_dragging_title.set(None);
            let _x = x;
            let _y = y;

            #[cfg(target_arch = "wasm32")]
            {
                let drag_id = dragging_id.read().clone();
                dragging_id.set(None);

                if let Some(task_id) = drag_id {
                    if let Some(window) = web_sys::window()
                        && let Some(doc) = window.document()
                    {
                        let mut target_el = doc.element_from_point(_x as f32, _y as f32);

                        // Walk up the DOM to find an element with data-drop-status
                        let mut drop_status = None;
                        let mut drop_position = None;
                        while let Some(el) = target_el {
                            if let Some(status) = el.get_attribute("data-drop-status") {
                                drop_status = Some(status);
                                drop_position = el
                                    .get_attribute("data-drop-position")
                                    .and_then(|p| p.parse::<i32>().ok());
                                break;
                            }
                            target_el = el.parent_element();
                        }

                        if let Some(status_str) = drop_status {
                            let is_focused = tasks_sig
                                .read()
                                .iter()
                                .any(|t| t.id == task_id && t.status == TaskStatus::Focused);

                            match status_str.as_str() {
                                "focus_dock" => {
                                    let max_pos = tasks_sig
                                        .read()
                                        .iter()
                                        .filter(|t| {
                                            t.status == TaskStatus::Focused
                                                || t.status == TaskStatus::InProgress
                                        })
                                        .map(|t| t.position)
                                        .max()
                                        .unwrap_or(0);
                                    let new_pos = max_pos + 10_000;

                                    {
                                        let mut map = pending_moves.read().clone();
                                        map.insert(
                                            task_id.clone(),
                                            (TaskStatus::InProgress, new_pos),
                                        );
                                        pending_moves.set(map);
                                    }

                                    let reorder = reorder.clone();
                                    spawn(async move {
                                        let _ = reorder
                                            .call(ReorderTaskInput::new(
                                                task_id,
                                                TaskStatus::InProgress,
                                                new_pos,
                                            ))
                                            .await;
                                    });
                                }
                                "dock_reorder" => {
                                    if let Some(position) = drop_position {
                                        let status = tasks_sig
                                            .read()
                                            .iter()
                                            .find(|t| t.id == task_id)
                                            .map(|t| t.status.clone());
                                        let target_status = match status {
                                            Some(
                                                TaskStatus::Focused | TaskStatus::InProgress,
                                            ) => status.unwrap(),
                                            _ => TaskStatus::InProgress,
                                        };

                                        {
                                            let mut map = pending_moves.read().clone();
                                            map.insert(
                                                task_id.clone(),
                                                (target_status.clone(), position),
                                            );
                                            pending_moves.set(map);
                                        }

                                        let reorder = reorder.clone();
                                        spawn(async move {
                                            let _ = reorder
                                                .call(ReorderTaskInput::new(
                                                    task_id,
                                                    target_status,
                                                    position,
                                                ))
                                                .await;
                                        });
                                    }
                                }
                                _ => {
                                    let target_status = match status_str.as_str() {
                                        "inbox" => Some(TaskStatus::Inbox),
                                        "up_next" => Some(TaskStatus::UpNext),
                                        "paused" => Some(TaskStatus::Paused),
                                        "done" => Some(TaskStatus::Done),
                                        "in_progress" => Some(TaskStatus::InProgress),
                                        "focused" => Some(TaskStatus::Focused),
                                        _ => None,
                                    };

                                    if let (Some(status), Some(position)) =
                                        (target_status, drop_position)
                                    {
                                        {
                                            let mut map = pending_moves.read().clone();
                                            map.insert(
                                                task_id.clone(),
                                                (status.clone(), position),
                                            );
                                            pending_moves.set(map);
                                        }

                                        let reorder = reorder.clone();
                                        let unfocus = unfocus.clone();
                                        spawn(async move {
                                            if is_focused {
                                                let _ = unfocus
                                                    .call(UnfocusTaskInput::new(
                                                        task_id.clone(),
                                                    ))
                                                    .await;
                                            }
                                            let _ = reorder
                                                .call(ReorderTaskInput::new(
                                                    task_id, status, position,
                                                ))
                                                .await;
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                dragging_id.set(None);
            }
        }
    };

    let on_drop = {
        let reorder = reorder.clone();
        let unfocus = unfocus.clone();
        move |target: DropTarget| {
            let drag_id = dragging_id.read().clone();
            dragging_id.set(None);
            if let Some(task_id) = drag_id {
                let is_focused = tasks_sig
                    .read()
                    .iter()
                    .any(|t| t.id == task_id && t.status == TaskStatus::Focused);

                {
                    let mut map = pending_moves.read().clone();
                    map.insert(task_id.clone(), (target.status.clone(), target.position));
                    pending_moves.set(map);
                }

                let reorder = reorder.clone();
                let unfocus = unfocus.clone();
                spawn(async move {
                    if is_focused {
                        let _ = unfocus.call(UnfocusTaskInput::new(task_id.clone())).await;
                    }
                    let _ = reorder
                        .call(ReorderTaskInput::new(
                            task_id.clone(),
                            target.status,
                            target.position,
                        ))
                        .await;
                    // Pending move is cleaned up during render when server data confirms it
                });
            }
        }
    };

    let on_drop_focus = {
        let reorder = reorder.clone();
        move |_: String| {
            let drag_id = dragging_id.read().clone();
            dragging_id.set(None);
            if let Some(task_id) = drag_id {
                let max_pos = tasks_sig
                    .read()
                    .iter()
                    .filter(|t| {
                        t.status == TaskStatus::Focused || t.status == TaskStatus::InProgress
                    })
                    .map(|t| t.position)
                    .max()
                    .unwrap_or(0);
                let new_pos = max_pos + 10_000;

                {
                    let mut map = pending_moves.read().clone();
                    map.insert(task_id.clone(), (TaskStatus::InProgress, new_pos));
                    pending_moves.set(map);
                }

                let reorder = reorder.clone();
                spawn(async move {
                    let _ = reorder
                        .call(ReorderTaskInput::new(
                            task_id.clone(),
                            TaskStatus::InProgress,
                            new_pos,
                        ))
                        .await;
                    // Pending move is cleaned up during render when server data confirms it
                });
            }
        }
    };

    let on_dock_reorder = {
        let reorder = reorder.clone();
        move |position: i32| {
            let drag_id = dragging_id.read().clone();
            dragging_id.set(None);
            if let Some(task_id) = drag_id {
                let reorder = reorder.clone();
                let status = tasks_sig
                    .read()
                    .iter()
                    .find(|t| t.id == task_id)
                    .map(|t| t.status.clone());
                let target_status = match status {
                    Some(TaskStatus::Focused | TaskStatus::InProgress) => status.unwrap(),
                    _ => TaskStatus::InProgress,
                };

                {
                    let mut map = pending_moves.read().clone();
                    map.insert(task_id.clone(), (target_status.clone(), position));
                    pending_moves.set(map);
                }

                spawn(async move {
                    let _ = reorder
                        .call(ReorderTaskInput::new(task_id.clone(), target_status, position))
                        .await;
                    // Pending move is cleaned up during render when server data confirms it
                });
            }
        }
    };

    let on_select = move |id: String| {
        selected_task_id.set(Some(id));
    };

    let on_close_detail = move |_: ()| {
        selected_task_id.set(None);
    };

    let selected_task = {
        let id = selected_task_id.read().clone();
        id.and_then(|sid| tasks.iter().find(|t| t.id == sid).cloned())
    };

    let on_keydown = {
        let focus_mut = focus.clone();
        move |e: Event<KeyboardData>| {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window()
                    && let Some(doc) = window.document()
                    && let Some(el) = doc.active_element()
                {
                    let tag = el.tag_name().to_lowercase();
                    if tag == "input" || tag == "textarea" {
                        if e.key() == Key::Escape {
                            show_capture.set(false);
                            selected_task_id.set(None);
                        }
                        return;
                    }
                }
            }

            let get_col_tasks = |col: usize| -> Vec<Task> {
                let all = tasks_sig.read();
                all.iter()
                    .filter(|t| t.status == COLUMN_STATUSES[col])
                    .cloned()
                    .collect()
            };

            match e.key() {
                Key::Escape => {
                    show_capture.set(false);
                    selected_task_id.set(None);
                    focused_col.set(None);
                    focused_row.set(None);
                }
                Key::Character(ref c) if c == "n" => {
                    show_capture.set(true);
                }
                Key::Character(ref c) if c == "f" => {
                    let current = *show_field_manager.read();
                    show_field_manager.set(!current);
                }
                Key::Character(ref c) if ("1"..="4").contains(&c.as_str()) => {
                    let col = c.parse::<usize>().unwrap_or(1) - 1;
                    focused_col.set(Some(col));
                    focused_row.set(Some(0));
                }
                Key::Character(ref c) if c == "j" => {
                    if let Some(col) = *focused_col.read() {
                        let count = get_col_tasks(col).len();
                        if count > 0 {
                            let row = focused_row.read().unwrap_or(0);
                            focused_row.set(Some((row + 1).min(count - 1)));
                        }
                    }
                }
                Key::Character(ref c) if c == "k" => {
                    if focused_col.read().is_some() {
                        let row = focused_row.read().unwrap_or(0);
                        focused_row.set(Some(row.saturating_sub(1)));
                    }
                }
                Key::Enter => {
                    if let (Some(col), Some(row)) = (*focused_col.read(), *focused_row.read()) {
                        let col_tasks = get_col_tasks(col);
                        if let Some(task) = col_tasks.get(row) {
                            selected_task_id.set(Some(task.id.clone()));
                        }
                    }
                }
                Key::Character(ref c) if c == " " => {
                    e.prevent_default();
                    if let (Some(col), Some(row)) = (*focused_col.read(), *focused_row.read()) {
                        let col_tasks = get_col_tasks(col);
                        if let Some(task) = col_tasks.get(row) {
                            let focus_mut = focus_mut.clone();
                            let id = task.id.clone();
                            spawn(async move {
                                let _ = focus_mut.call(FocusTaskInput::new(id)).await;
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    };

    rsx! {
        div {
            class: "dashboard",
            tabindex: 0,
            onkeydown: on_keydown,

            Header {
                on_manage_fields: move |_| show_field_manager.set(true),
            }

            if *show_capture.read() {
                QuickCapture {
                    on_submit: {
                        let on_create = on_create.clone();
                        move |title: String| {
                            on_create((title, TaskStatus::Inbox));
                            show_capture.set(false);
                        }
                    },
                    on_close: move |_: ()| show_capture.set(false),
                }
            }

            FocusDock {
                tasks: dock_tasks.clone(),
                is_drag_active: dragging_id.read().is_some(),
                on_focus: on_focus.clone(),
                on_unfocus: on_unfocus.clone(),
                on_drop_focus: on_drop_focus.clone(),
                on_reorder: on_dock_reorder.clone(),
                on_drag_start,
                on_drag_end,
                on_select,
                on_touch_drag_start: on_touch_drag_start,
                on_touch_drag_move: on_touch_drag_move,
                on_touch_drag_end: on_touch_drag_end,
            }

            Board {
                tasks: tasks.clone(),
                focused_col: *focused_col.read(),
                focused_row: *focused_row.read(),
                dragging_id: dragging_id.read().clone(),
                on_select,
                on_delete: on_delete.clone(),
                on_create: on_create.clone(),
                on_drag_start,
                on_drag_end,
                on_drop,
                on_touch_drag_start: on_touch_drag_start,
                on_touch_drag_move: on_touch_drag_move,
                on_touch_drag_end: on_touch_drag_end,
            }

            // Touch drag ghost card
            if let Some((x, y)) = *touch_pos.read() {
                if let Some(ref title) = *touch_dragging_title.read() {
                    div {
                        class: "touch-drag-ghost",
                        style: "left: {x}px; top: {y}px;",
                        "{title}"
                    }
                }
            }

            if let Some(task) = selected_task {
                DetailPanel {
                    task: task,
                    field_defs: field_defs.clone(),
                    task_fields: all_task_fields.clone(),
                    on_close: on_close_detail,
                    on_update,
                }
            }

            if *show_field_manager.read() {
                FieldManager {
                    on_close: move |_| show_field_manager.set(false),
                }
            }

            UndoToast {
                action: undo_action.read().clone(),
                on_undo: move |_task_id: String| {
                    undo_action.set(None);
                },
            }
        }
    }
}
