use dioxus::prelude::*;

use crate::forge::{Viewer, use_forge_auth};
use crate::time_utils;

#[component]
pub fn Header(
    viewer: Option<Viewer>,
    daily_total: i64,
    on_manage_fields: EventHandler<()>,
) -> Element {
    let mut auth = use_forge_auth();
    let name = viewer.as_ref().map(|v| v.name.as_str()).unwrap_or("");
    let total_str = time_utils::format_duration(daily_total);

    rsx! {
        header { class: "app-header",
            div { class: "header-left",
                h1 { class: "app-title", "kaizen" }
                span { class: "header-subtitle", "personal workspace" }
            }
            div { class: "header-right",
                if !total_str.is_empty() {
                    span { class: "header-daily-total", "{total_str}" }
                }
                button {
                    class: "logout-btn",
                    onclick: move |_| on_manage_fields.call(()),
                    title: "Manage fields",
                    "Fields"
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
