use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UndoAction {
    pub label: String,
    pub task_id: String,
}

#[component]
pub fn UndoToast(action: Option<UndoAction>, on_undo: EventHandler<String>) -> Element {
    let Some(ref act) = action else {
        return rsx! {};
    };

    rsx! {
        div { class: "undo-toast",
            span { "{act.label}" }
            button {
                class: "undo-btn",
                onclick: {
                    let id = act.task_id.clone();
                    move |_| on_undo.call(id.clone())
                },
                "Undo"
            }
        }
    }
}
