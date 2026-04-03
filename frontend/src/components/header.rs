use dioxus::prelude::*;

use crate::forge::{Task, Viewer, use_forge_auth};
use crate::pages::dashboard::format_time;

#[component]
pub fn Header(viewer: Option<Viewer>, tasks: Vec<Task>, focused_task: Option<Task>) -> Element {
    let mut auth = use_forge_auth();

    let daily_total: i64 = tasks.iter().map(|t| t.time_spent_secs).sum();

    let name = viewer.as_ref().map(|v| v.name.as_str()).unwrap_or("");

    rsx! {
        header { class: "app-header",
            div { class: "header-left",
                h1 { class: "app-title", "kaizen" }
                span { class: "header-subtitle", "personal workspace" }
            }
            div { class: "header-right",
                if daily_total > 0 {
                    span { class: "header-daily-total", "{format_time(daily_total)}" }
                }
                span { class: "header-user", "{name}" }
                button {
                    class: "logout-btn",
                    onclick: move |_| auth.logout(),
                    "Sign out"
                }
            }
        }
    }
}
