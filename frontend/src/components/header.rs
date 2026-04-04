use dioxus::prelude::*;

use crate::forge::use_forge_auth;

#[component]
pub fn Header(on_manage_fields: EventHandler<()>) -> Element {
    let mut auth = use_forge_auth();

    rsx! {
        header { class: "app-header",
            div { class: "header-left",
                h1 { class: "app-title", "kaizen" }
                span { class: "header-subtitle", "personal workspace" }
            }
            div { class: "header-right",
                button {
                    class: "header-icon-btn",
                    title: "Manage fields",
                    onclick: move |_| on_manage_fields.call(()),
                    "⚙"
                }
                button {
                    class: "header-icon-btn",
                    title: "Sign out",
                    onclick: move |_| auth.logout(),
                    "⏻"
                }
            }
        }
    }
}
