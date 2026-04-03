use dioxus::prelude::*;

#[component]
pub fn QuickCapture(on_submit: EventHandler<String>, on_close: EventHandler<()>) -> Element {
    let mut title = use_signal(String::new);

    rsx! {
        div { class: "capture-bar",
            form {
                onsubmit: move |e| {
                    e.prevent_default();
                    let t = title.read().trim().to_string();
                    if !t.is_empty() {
                        on_submit.call(t);
                        title.set(String::new());
                    }
                },
                input {
                    class: "capture-input",
                    placeholder: "What needs to be done?",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Escape {
                            on_close.call(());
                        }
                    },
                }
            }
        }
    }
}
