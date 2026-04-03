use dioxus::prelude::*;

use crate::forge::{
    AuthResponse, LoginInput, RegisterInput, use_forge_auth, use_login, use_register,
};

#[component]
pub fn Login() -> Element {
    let mut auth = use_forge_auth();
    let register_mutation = use_register();
    let login_mutation = use_login();
    let navigator = use_navigator();

    let mut is_registering = use_signal(|| false);
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);

    let on_submit = move |e: Event<FormData>| {
        e.prevent_default();
        let register_mutation = register_mutation.clone();
        let login_mutation = login_mutation.clone();
        let nav = navigator;
        let registering = *is_registering.read();
        let n = name.read().clone();
        let em = email.read().clone();
        let pw = password.read().clone();

        spawn(async move {
            loading.set(true);
            error.set(None);

            let result: Result<AuthResponse, _> = if registering {
                register_mutation
                    .call(RegisterInput::new(&n, &em, &pw))
                    .await
            } else {
                login_mutation.call(LoginInput::new(&em, &pw)).await
            };

            loading.set(false);

            match result {
                Ok(resp) => {
                    let viewer = resp.viewer.clone();
                    auth.login_with_viewer(resp.access_token, resp.refresh_token, &viewer);
                    nav.push("/");
                }
                Err(e) => {
                    error.set(Some(e.message.clone()));
                }
            }
        });
    };

    rsx! {
        div { class: "login-page",
            div { class: "login-card",
                h1 { class: "login-title", "kaizen" }
                p { class: "login-subtitle", "personal workspace" }

                if let Some(err) = &*error.read() {
                    div { class: "login-error", "{err}" }
                }

                form { onsubmit: on_submit,
                    if *is_registering.read() {
                        input {
                            class: "login-input",
                            r#type: "text",
                            placeholder: "Name",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    input {
                        class: "login-input",
                        r#type: "email",
                        placeholder: "Email",
                        required: true,
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                    input {
                        class: "login-input",
                        r#type: "password",
                        placeholder: "Password",
                        required: true,
                        minlength: 8,
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                    button {
                        class: "login-button",
                        r#type: "submit",
                        disabled: *loading.read(),
                        if *loading.read() {
                            "..."
                        } else if *is_registering.read() {
                            "Create account"
                        } else {
                            "Sign in"
                        }
                    }
                }

                button {
                    class: "login-toggle",
                    onclick: move |_| {
                        let current = *is_registering.read();
                        is_registering.set(!current);
                        error.set(None);
                    },
                    if *is_registering.read() {
                        "Already have an account? Sign in"
                    } else {
                        "Need an account? Register"
                    }
                }
            }
        }
    }
}
