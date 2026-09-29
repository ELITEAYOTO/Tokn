#[derive(Debug, Clone, Default)]
pub struct CodexCapabilities {
    pub session_rollouts: bool,
    pub trace_reduce: bool,
    pub prompt_input_debug: bool,
    pub app_server_debug: bool,
}

impl CodexCapabilities {
    pub fn names(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.session_rollouts {
            out.push("session-rollouts");
        }
        if self.trace_reduce {
            out.push("trace-reduce");
        }
        if self.prompt_input_debug {
            out.push("prompt-input");
        }
        if self.app_server_debug {
            out.push("app-server");
        }
        out
    }
}
