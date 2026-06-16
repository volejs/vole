use std::process::Command;

use crate::spec::SignalSpec;

pub fn send_signal(pid: u32, signal: SignalSpec) -> bool {
    match signal {
        SignalSpec::Invalid => return false,
        _ => {}
    }

    #[cfg(unix)]
    {
        let mut cmd = Command::new("kill");
        match signal {
            SignalSpec::Default => {}
            SignalSpec::Check => {
                cmd.arg("-0");
            }
            SignalSpec::Signal(sig) => {
                cmd.arg(format!("-{sig}"));
            }
            SignalSpec::Invalid => unreachable!(),
        }

        return cmd
            .arg(pid.to_string())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
    }

    #[cfg(windows)]
    {
        match signal {
            SignalSpec::Check => {
                let output = Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
                    .output();
                if let Ok(output) = output {
                    if !output.status.success() {
                        return false;
                    }
                    let text = String::from_utf8_lossy(&output.stdout);
                    text.lines()
                        .any(|line| line.contains(&format!("\"{pid}\"")))
                } else {
                    false
                }
            }
            SignalSpec::Default | SignalSpec::Signal(_) | SignalSpec::Invalid => {
                let mut cmd = Command::new("taskkill");
                let _ = cmd.arg("/PID").arg(pid.to_string()).arg("/T").arg("/F");
                cmd.status().map(|status| status.success()).unwrap_or(false)
            }
        }
    }
}
