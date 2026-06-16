use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use rquickjs::{Ctx, Exception, Result};
use tokio::sync::{mpsc, Mutex as AsyncMutex};

#[cfg(unix)]
use tokio::signal::unix::{self, SignalKind};

use crate::spec::{canonicalize_signal_name, SUPPORTED_SIGNALS};

struct SignalSubscription {
    signals: HashSet<String>,
    sender: mpsc::UnboundedSender<String>,
}

#[derive(Default)]
struct SignalRegistry {
    next_id: u64,
    subscriptions: HashMap<u64, SignalSubscription>,
    watching: HashSet<String>,
}

static SIGNAL_REGISTRY: OnceLock<Arc<Mutex<SignalRegistry>>> = OnceLock::new();

fn registry() -> &'static Arc<Mutex<SignalRegistry>> {
    SIGNAL_REGISTRY.get_or_init(|| Arc::new(Mutex::new(SignalRegistry::default())))
}

fn next_subscription_id() -> u64 {
    let mut state = registry().lock().unwrap_or_else(|err| err.into_inner());
    state.next_id += 1;
    state.next_id
}

#[cfg(unix)]
fn signal_to_kind(signal: &str) -> Option<SignalKind> {
    let raw = match signal {
        "SIGABRT" => libc::SIGABRT,
        "SIGALRM" => libc::SIGALRM,
        "SIGFPE" => libc::SIGFPE,
        "SIGHUP" => libc::SIGHUP,
        "SIGILL" => libc::SIGILL,
        "SIGINT" => libc::SIGINT,
        "SIGKILL" => libc::SIGKILL,
        "SIGPIPE" => libc::SIGPIPE,
        "SIGQUIT" => libc::SIGQUIT,
        "SIGSEGV" => libc::SIGSEGV,
        "SIGTERM" => libc::SIGTERM,
        "SIGUSR1" => libc::SIGUSR1,
        "SIGUSR2" => libc::SIGUSR2,
        "SIGCONT" => libc::SIGCONT,
        "SIGSTOP" => libc::SIGSTOP,
        "SIGTSTP" => libc::SIGTSTP,
        "SIGTTIN" => libc::SIGTTIN,
        "SIGTTOU" => libc::SIGTTOU,
        _ => return None,
    };

    Some(SignalKind::from_raw(raw))
}

#[cfg(unix)]
fn ensure_watcher<'js>(ctx: &Ctx<'js>, signal: &str) -> Result<()> {
    let kind = signal_to_kind(signal).ok_or_else(|| {
        Exception::throw_type(
            ctx,
            &format!("signal is not supported for watching: {signal}"),
        )
    })?;

    {
        let mut state = registry().lock().unwrap_or_else(|err| err.into_inner());
        if !state.watching.insert(signal.to_string()) {
            return Ok(());
        }
    }

    let signal = signal.to_string();
    tokio::spawn(async move {
        let Ok(mut stream) = unix::signal(kind) else {
            return;
        };

        while stream.recv().await.is_some() {
            dispatch_to_subscribers(signal.clone());
        }
    });

    Ok(())
}

#[cfg(not(unix))]
fn ensure_watcher<'js>(ctx: &Ctx<'js>, _signal: &str) -> Result<()> {
    Err(Exception::throw_type(
        ctx,
        "signal subscription is not supported on this platform",
    ))
}

#[cfg(unix)]
fn dispatch_to_subscribers(raw_signal: String) {
    let mut to_remove = Vec::new();
    let registry = registry();
    let mut state = registry.lock().unwrap_or_else(|err| err.into_inner());
    for (id, sub) in &state.subscriptions {
        if !sub.signals.contains(&raw_signal) {
            continue;
        }

        if sub.sender.send(raw_signal.clone()).is_err() {
            to_remove.push(*id);
        }
    }

    for id in to_remove {
        state.subscriptions.remove(&id);
    }
}

fn subscription_signals<'js>(ctx: &Ctx<'js>, raw: Vec<String>) -> Result<HashSet<String>> {
    if raw.is_empty() {
        return Ok(SUPPORTED_SIGNALS
            .iter()
            .map(|signal| signal.to_string())
            .collect());
    }

    raw.into_iter()
        .map(|signal| {
            canonicalize_signal_name(&signal)
                .filter(|value| SUPPORTED_SIGNALS.contains(&value.as_str()))
                .ok_or_else(|| {
                    Exception::throw_type(ctx, &format!("unsupported signal name: {signal}"))
                })
        })
        .collect()
}

pub(crate) fn create_subscription<'js>(
    ctx: &Ctx<'js>,
    signals: Vec<String>,
) -> Result<(u64, Arc<AsyncMutex<mpsc::UnboundedReceiver<String>>>)> {
    let subscribe_set = subscription_signals(ctx, signals)?;

    for signal in &subscribe_set {
        ensure_watcher(ctx, signal)?;
    }

    let (sender, receiver) = mpsc::unbounded_channel();
    let receiver = Arc::new(AsyncMutex::new(receiver));
    let id = next_subscription_id();
    let subscription = SignalSubscription {
        signals: subscribe_set,
        sender,
    };

    let mut state = registry().lock().unwrap_or_else(|err| err.into_inner());
    state.subscriptions.insert(id, subscription);
    Ok((id, receiver))
}

pub(crate) fn remove_subscription(id: u64) -> bool {
    let mut state = registry().lock().unwrap_or_else(|err| err.into_inner());
    state.subscriptions.remove(&id).is_some()
}
