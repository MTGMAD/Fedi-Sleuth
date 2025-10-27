use dioxus::prelude::*;

#[derive(Props, PartialEq)]
pub struct StatusBarProps {
    pub message: String,
    pub is_searching: bool,
}

pub fn StatusBar(cx: Scope<StatusBarProps>) -> Element {
    cx.render(rsx! {
        div {
            class: "status-bar",
            div {
                class: "status-content",
                if cx.props.is_searching {
                    rsx! {
                        span {
                            class: "status-indicator searching",
                            "🔄"
                        }
                    }
                } else {
                    rsx! {
                        span {
                            class: "status-indicator ready",
                            "✅"
                        }
                    }
                }
                span {
                    class: "status-message",
                    "{cx.props.message}"
                }
            }
        }
    })
}
