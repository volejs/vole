pub(crate) const SUPPORTED_SIGNALS: &[&str] = &[
    "SIGABRT", "SIGALRM", "SIGFPE", "SIGHUP", "SIGILL", "SIGINT", "SIGKILL", "SIGPIPE", "SIGQUIT",
    "SIGSEGV", "SIGTERM", "SIGUSR1", "SIGUSR2", "SIGCONT", "SIGSTOP", "SIGTSTP", "SIGTTIN",
    "SIGTTOU",
];

/// 系统信号语义描述。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignalSpec {
    /// 默认信号。
    Default,
    /// 检查信号：对应“probe/探测进程是否存在”。
    Check,
    /// 命名信号（如 `SIGTERM`、`TERM`、`9`）。
    Signal(String),
    /// 无效信号。
    Invalid,
}

pub fn parse_signal(raw: Option<&str>) -> SignalSpec {
    fn is_numeric_signal(raw: &str) -> bool {
        raw.chars().all(|c| c.is_ascii_digit())
    }

    let Some(raw) = raw else {
        return SignalSpec::Default;
    };

    let raw = raw.trim();
    if raw.is_empty() {
        return SignalSpec::Default;
    }

    let raw = raw
        .trim_start_matches("SIG")
        .trim_start_matches('-')
        .to_ascii_uppercase();
    if raw == "0" {
        return SignalSpec::Check;
    }
    if raw.is_empty() {
        return SignalSpec::Invalid;
    }

    let allowed = is_numeric_signal(&raw)
        || matches!(
            raw.as_str(),
            "ABRT"
                | "ALRM"
                | "FPE"
                | "HUP"
                | "ILL"
                | "INT"
                | "KILL"
                | "PIPE"
                | "QUIT"
                | "SEGV"
                | "TERM"
                | "USR1"
                | "USR2"
                | "CONT"
                | "STOP"
                | "TSTP"
                | "TTIN"
                | "TTOU"
        );

    if allowed {
        SignalSpec::Signal(raw)
    } else {
        SignalSpec::Invalid
    }
}

pub fn is_terminating_signal(signal: &SignalSpec) -> bool {
    !matches!(signal, SignalSpec::Check)
}

pub(crate) fn canonicalize_signal_name(raw: &str) -> Option<String> {
    match parse_signal(Some(raw)) {
        SignalSpec::Signal(signal) => {
            if signal.chars().all(|ch| ch.is_ascii_digit()) {
                return None;
            }

            let normalized = if signal.starts_with("SIG") {
                signal
            } else {
                format!("SIG{signal}")
            };

            Some(normalized)
        }
        _ => None,
    }
}

pub(crate) fn is_supported_signal(raw: &str) -> bool {
    canonicalize_signal_name(raw).is_some_and(|value| SUPPORTED_SIGNALS.contains(&value.as_str()))
}

pub(crate) fn signal_list() -> Vec<String> {
    SUPPORTED_SIGNALS
        .iter()
        .map(|signal| (*signal).to_string())
        .collect()
}
