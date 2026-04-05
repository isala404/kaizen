mod components;
mod forge;
mod layout;
mod pages;
pub mod time_utils;

use dioxus::prelude::*;
use forge::{ForgeAuthProvider, use_auth_key};

use layout::ProtectedLayout;
use pages::{Dashboard, Login, NotFound};

fn api_url() -> &'static str {
    option_env!("FORGE_API_URL").unwrap_or("http://localhost:9081")
}

#[derive(Routable, Clone)]
#[rustfmt::skip]
enum Route {
    #[route("/login")]
    Login {},
    #[layout(ProtectedLayout)]
        #[route("/")]
        Dashboard {},
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

fn main() {
    dioxus_sdk::storage::set_dir!();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "kaizen" }
        document::Stylesheet { href: asset!("/public/style.css") }
        ForgeAuthProvider {
            url: api_url().to_string(),
            app_name: "kaizen".to_string(),
            refresh_interval_secs: 2400,
            AppShell {}
        }
    }
}

#[component]
fn AppShell() -> Element {
    let auth_key = use_auth_key();
    rsx! {
        main { key: "{auth_key}",
            Router::<Route> {}
        }
    }
}
