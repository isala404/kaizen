use dioxus::prelude::*;

use crate::components::{
    Board, DetailPanel, FocusDock, Header, QuickCapture, StatusChange, StatusTabs, TaskList,
};
use crate::forge::{
    CreateTaskInput, DeleteTaskInput, FocusTaskInput, Task, TaskStatus, UpdateTaskInput, Viewer,
    use_create_task, use_delete_task, use_focus_task, use_list_tasks_live, use_unfocus_task,
    use_update_task, use_viewer,
};

#[component]
pub fn Dashboard() -> Element {
    let viewer: Option<Viewer> = use_viewer::<Viewer>();
    let tasks_state = use_list_tasks_live();

    let create = use_create_task();
    let delete = use_delete_task();
    let focus = use_focus_task();
    let unfocus = use_unfocus_task();
    let update = use_update_task();

    let mut selected_task_id = use_signal(|| Option::<String>::None);
    let mut show_capture = use_signal(|| false);
    let mut active_tab = use_signal(|| TaskStatus::Inbox);

    let tasks: Vec<Task> = tasks_state.data.clone().unwrap_or_default();
    let focused_task = tasks
        .iter()
        .find(|t| t.status == TaskStatus::Focused)
        .cloned();

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
        move |id: String| {
            let delete = delete.clone();
            spawn(async move {
                let _ = delete.call(DeleteTaskInput::new(id)).await;
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

    rsx! {
        div { class: "dashboard",
            Header {
                viewer: viewer.clone(),
                tasks: tasks.clone(),
                focused_task: focused_task.clone(),
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

            // Desktop layout
            div { class: "desktop-layout",
                FocusDock {
                    task: focused_task.clone(),
                    on_click: on_select,
                    on_unfocus: on_unfocus.clone(),
                }

                Board {
                    tasks: tasks.clone(),
                    on_select,
                    on_delete: on_delete.clone(),
                    on_focus: on_focus.clone(),
                    on_create: on_create.clone(),
                    on_status_change: on_status_change.clone(),
                }
            }

            // Mobile layout
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
                        span { class: "focus-bar-time", "{format_time(ft.time_spent_secs)}" }
                    }
                }

                StatusTabs {
                    tasks: tasks.clone(),
                    active: active_tab(),
                    on_change: move |status: TaskStatus| active_tab.set(status),
                }

                TaskList {
                    tasks: tasks.clone(),
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
                    on_close: on_close_detail,
                    on_status_change: on_status_change.clone(),
                    on_focus: on_focus.clone(),
                }
            }
        }
    }
}

pub fn format_time(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m:02}m")
    } else if m > 0 {
        format!("{m}m")
    } else {
        String::new()
    }
}
