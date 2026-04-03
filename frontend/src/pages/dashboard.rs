use dioxus::prelude::*;

use crate::components::{
    ActiveFilters, Board, DetailPanel, DropTarget, FieldFilterBar, FieldManager, FocusDock, Header,
    QuickCapture, StatusChange, StatusTabs, TaskList, UndoAction, UndoToast,
};
use crate::forge::{
    CreateTaskInput, DeleteTaskInput, FieldDefinition, FocusTaskInput, ReorderTaskInput, Task,
    TaskField, TaskStatus, UpdateTaskInput, Viewer, use_create_task, use_delete_task,
    use_focus_task, use_list_all_task_fields_live, use_list_field_definitions_live,
    use_list_tasks_live, use_reorder_task, use_unfocus_task, use_update_task, use_viewer,
};
use crate::time_utils;

const COLUMN_STATUSES: [TaskStatus; 5] = [
    TaskStatus::Inbox,
    TaskStatus::UpNext,
    TaskStatus::InProgress,
    TaskStatus::Paused,
    TaskStatus::Done,
];

#[component]
pub fn Dashboard() -> Element {
    let viewer: Option<Viewer> = use_viewer::<Viewer>();
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
    let mut active_tab = use_signal(|| TaskStatus::Inbox);
    let mut focused_col = use_signal(|| Option::<usize>::None);
    let mut focused_row = use_signal(|| Option::<usize>::None);

    // Live timer tick
    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            tick.set(tick() + 1);
        }
    });

    let field_defs_state = use_list_field_definitions_live();
    let task_fields_state = use_list_all_task_fields_live();
    let mut filters = use_signal(ActiveFilters::new);
    let mut show_field_manager = use_signal(|| false);

    let field_defs: Vec<FieldDefinition> = field_defs_state.data.clone().unwrap_or_default();
    let all_task_fields: Vec<TaskField> = task_fields_state.data.clone().unwrap_or_default();

    let tasks: Vec<Task> = tasks_state.data.clone().unwrap_or_default();
    let focused_task = tasks
        .iter()
        .find(|t| t.status == TaskStatus::Focused)
        .cloned();

    let _tick_val = tick();
    let focused_elapsed = focused_task
        .as_ref()
        .map(|t| time_utils::focused_elapsed(t.time_spent_secs, &t.updated_at));

    let daily_total: i64 = tasks
        .iter()
        .map(|t| {
            if t.status == TaskStatus::Focused {
                focused_elapsed.unwrap_or(t.time_spent_secs)
            } else {
                t.time_spent_secs
            }
        })
        .sum();

    // Filter tasks by active field filters
    let filtered_tasks: Vec<Task> = {
        let f = filters.read();
        if f.is_empty() {
            tasks.clone()
        } else {
            tasks
                .iter()
                .filter(|t| f.matches(&t.id, &all_task_fields))
                .cloned()
                .collect()
        }
    };

    // Store tasks in signal so keyboard handler can access without moving
    let mut tasks_sig = use_signal(Vec::<Task>::new);
    // Update on each render
    *tasks_sig.write() = tasks.clone();

    let on_create = {
        let create = create.clone();
        move |title: String| {
            let create = create.clone();
            spawn(async move {
                let _ = create.call(CreateTaskInput::new(title)).await;
            });
        }
    };

    let on_delete = {
        let delete = delete.clone();
        let tasks_for_delete = tasks.clone();
        move |id: String| {
            // Show undo toast, delay actual delete by 5 seconds
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
                // Wait 5 seconds, then delete if undo wasn't triggered
                gloo_timers::future::TimeoutFuture::new(5_000).await;
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
        move |_: ()| {
            let unfocus = unfocus.clone();
            spawn(async move {
                let _ = unfocus.call(()).await;
            });
        }
    };

    let on_status_change = {
        let update = update.clone();
        move |sc: StatusChange| {
            let update = update.clone();
            spawn(async move {
                let _ = update
                    .call(UpdateTaskInput::new(sc.id).status(sc.status))
                    .await;
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

    let on_drop = {
        let reorder = reorder.clone();
        move |target: DropTarget| {
            let drag_id = dragging_id.read().clone();
            dragging_id.set(None);
            if let Some(task_id) = drag_id {
                let reorder = reorder.clone();
                spawn(async move {
                    let _ = reorder
                        .call(ReorderTaskInput::new(
                            task_id,
                            target.status,
                            target.position,
                        ))
                        .await;
                });
            }
        }
    };

    let on_drop_focus = {
        let focus = focus.clone();
        move |_: String| {
            let drag_id = dragging_id.read().clone();
            dragging_id.set(None);
            if let Some(task_id) = drag_id {
                let focus = focus.clone();
                spawn(async move {
                    let _ = focus.call(FocusTaskInput::new(task_id)).await;
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

    // Keyboard handler uses cloned mutation handles directly
    let on_keydown = {
        let focus_mut = focus.clone();
        let unfocus_mut = unfocus.clone();
        move |e: Event<KeyboardData>| {
            // Skip if an input/textarea is focused (except Escape)
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
                Key::Character(ref c) if ("1"..="5").contains(&c.as_str()) => {
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
                Key::Backspace => {
                    let unfocus_mut = unfocus_mut.clone();
                    spawn(async move {
                        let _ = unfocus_mut.call(()).await;
                    });
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
                viewer: viewer.clone(),
                daily_total,
                on_manage_fields: move |_| show_field_manager.set(true),
            }

            if *show_capture.read() {
                QuickCapture {
                    on_submit: {
                        let on_create = on_create.clone();
                        move |title: String| {
                            on_create(title);
                            show_capture.set(false);
                        }
                    },
                    on_close: move |_: ()| show_capture.set(false),
                }
            }

            if !field_defs.is_empty() {
                FieldFilterBar {
                    fields: field_defs.clone(),
                    filters: filters.read().clone(),
                    on_toggle: move |(fid, val): (String, String)| {
                        filters.write().toggle(&fid, &val);
                    },
                }
            }

            div { class: "desktop-layout",
                FocusDock {
                    task: focused_task.clone(),
                    elapsed_secs: focused_elapsed,
                    is_drag_active: dragging_id.read().is_some(),
                    on_click: on_select,
                    on_unfocus: on_unfocus.clone(),
                    on_drop_focus: on_drop_focus.clone(),
                }

                Board {
                    tasks: filtered_tasks.clone(),
                    field_defs: field_defs.clone(),
                    task_fields: all_task_fields.clone(),
                    focused_col: *focused_col.read(),
                    focused_row: *focused_row.read(),
                    dragging_id: dragging_id.read().clone(),
                    on_select,
                    on_delete: on_delete.clone(),
                    on_focus: on_focus.clone(),
                    on_create: on_create.clone(),
                    on_status_change: on_status_change.clone(),
                    on_drag_start,
                    on_drag_end,
                    on_drop,
                }
            }

            div { class: "mobile-layout",
                if let Some(ref ft) = focused_task {
                    div {
                        class: "focus-bar",
                        onclick: {
                            let id = ft.id.clone();
                            let mut on_select = on_select;
                            move |_| on_select(id.clone())
                        },
                        div { class: "focus-bar-dot" }
                        span { class: "focus-bar-title", "{ft.title}" }
                        span { class: "focus-bar-time",
                            "{time_utils::format_timer(focused_elapsed.unwrap_or(0))}"
                        }
                    }
                }

                StatusTabs {
                    tasks: filtered_tasks.clone(),
                    active: active_tab(),
                    on_change: move |status: TaskStatus| active_tab.set(status),
                }

                TaskList {
                    tasks: filtered_tasks.clone(),
                    active_status: active_tab(),
                    on_select,
                    on_delete: on_delete.clone(),
                    on_focus: on_focus.clone(),
                    on_status_change: on_status_change.clone(),
                }

                button {
                    class: "fab",
                    onclick: move |_| show_capture.set(true),
                    "+"
                }
            }

            if let Some(task) = selected_task {
                DetailPanel {
                    task: task,
                    field_defs: field_defs.clone(),
                    task_fields: all_task_fields.clone(),
                    on_close: on_close_detail,
                    on_status_change: on_status_change.clone(),
                    on_focus: on_focus.clone(),
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
