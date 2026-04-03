use dioxus::prelude::*;

use crate::Route;
use crate::forge::use_require_auth;

#[component]
pub fn ProtectedLayout() -> Element {
    if !use_require_auth("/login") {
        return rsx! {
            div { class: "loading", "Loading..." }
        };
    }

    rsx! {
        Outlet::<Route> {}
    }
}
