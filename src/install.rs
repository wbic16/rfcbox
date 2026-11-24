//! systemd deployment for a corpus host — the machinery behind
//! `rfcbox install`.
//!
//! rfcbox has no daemon: every command is a single run that exits.
//! The one invocation a corpus host supervises is the §4 maintenance
//! run — `rfcbox refresh --corpus IETF.PHEXT` — which discovers RFCs
//! published after the audited snapshot, fetches them verbatim at
//! human speed, and appends them to a *candidate* snapshot; the
//! audited artifact is never edited in place, and the candidate
//! replaces the served corpus only after re-audit. `rfcbox install`
//! renders that run as a systemd unit and, run as root, deploys it:
//! write the unit, `systemctl daemon-reload`, and enable/start only
//! when explicitly flagged. Unit rendering is a pure function so the
//! exact bytes a superuser would deploy are testable and printable
//! (`--dry-run`) by anyone.

use std::path::PathBuf;

use crate::{Error, Result};

/// Everything the unit file needs, gathered by the CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallConfig {
    /// Unit name — the `<name>` in `<name>.service`.
    pub name: String,
    /// Absolute path of the rfcbox executable (std::env::current_exe).
    pub exec: PathBuf,
    /// The corpus the refresh run maintains (the `--corpus` of §4).
    pub corpus: String,
    /// WorkingDirectory for the service.
    pub dir: PathBuf,
    /// Optional User= for the service.
    pub user: Option<String>,
}

/// Unit names become file names under /etc/systemd/system — keep them
/// to the characters that are safe there.
pub fn validate_unit_name(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(Error::Usage(format!(
            "invalid unit name {name:?} — letters, digits, '-', '_' and '.' only"
        )));
    }
    Ok(())
}

/// Where a deployed unit lives.
pub fn unit_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/etc/systemd/system/{name}.service"))
}

/// Render the unit. Pure: same config in, same bytes out.
pub fn render_unit(cfg: &InstallConfig) -> String {
    let mut u = String::new();
    u.push_str("[Unit]\n");
    u.push_str("Description=rfcbox corpus refresh — the §4 maintenance run against the audited snapshot (a candidate is written; the served corpus is never edited in place)\n");
    u.push_str("After=network-online.target\n");
    u.push_str("Wants=network-online.target\n");
    u.push_str("\n[Service]\n");
    u.push_str(&format!(
        "ExecStart={} refresh --corpus {}\n",
        cfg.exec.display(),
        cfg.corpus
    ));
    u.push_str(&format!("WorkingDirectory={}\n", cfg.dir.display()));
    if let Some(user) = &cfg.user {
        u.push_str(&format!("User={user}\n"));
    }
    u.push_str("Restart=on-failure\n");
    u.push_str("RestartSec=2s\n");
    u.push_str("TimeoutStopSec=10s\n");
    u.push_str("KillMode=control-group\n");
    u.push_str("\n[Install]\n");
    u.push_str("WantedBy=multi-user.target\n");
    u
}

/// Parse the effective uid out of /proc/self/status contents — the
/// second field of the Uid line (real, effective, saved-set, fs).
pub fn parse_effective_uid(status: &str) -> Option<u32> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() >= 2 {
                return fields[1].parse().ok();
            }
        }
    }
    None
}

/// This process's effective uid, read from /proc (no libc call).
pub fn effective_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    parse_effective_uid(&status)
}

/// Deploying a unit is a superuser's act; everyone else gets the
/// unit printed plus the exact sudo invocation (the CLI does that).
pub fn running_as_root() -> bool {
    effective_uid() == Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(user: Option<&str>) -> InstallConfig {
        InstallConfig {
            name: "rfcbox".into(),
            exec: PathBuf::from("/usr/local/bin/rfcbox"),
            corpus: "ietf.phext".into(),
            dir: PathBuf::from("/srv/rfcbox"),
            user: user.map(|s| s.to_string()),
        }
    }

    #[test]
    fn unit_carries_the_refresh_run_and_supervision() {
        let u = render_unit(&cfg(None));
        assert!(u.contains("ExecStart=/usr/local/bin/rfcbox refresh --corpus ietf.phext\n"));
        assert!(u.contains("WorkingDirectory=/srv/rfcbox\n"));
        assert!(u.contains("Restart=on-failure\n"));
        assert!(u.contains("RestartSec=2s\n"));
        assert!(u.contains("TimeoutStopSec=10s\n"));
        assert!(u.contains("KillMode=control-group\n"));
        assert!(u.contains("WantedBy=multi-user.target\n"));
    }

    #[test]
    fn user_line_present_only_when_set() {
        assert!(!render_unit(&cfg(None)).contains("User="));
        assert!(render_unit(&cfg(Some("wbic16"))).contains("User=wbic16\n"));
    }

    #[test]
    fn unit_names_are_validated() {
        assert!(validate_unit_name("rfcbox").is_ok());
        assert!(validate_unit_name("rfcbox-corpus_2.1").is_ok());
        assert!(validate_unit_name("").is_err());
        assert!(validate_unit_name("../evil").is_err());
        assert!(validate_unit_name("a b").is_err());
        assert!(validate_unit_name("a/b").is_err());
    }

    #[test]
    fn unit_path_is_the_systemd_system_dir() {
        assert_eq!(
            unit_path("rfcbox"),
            PathBuf::from("/etc/systemd/system/rfcbox.service")
        );
    }

    #[test]
    fn effective_uid_is_the_second_uid_field() {
        let status = "Name:\trfcbox\nUid:\t1000\t0\t1000\t1000\nGid:\t1000\t1000\t1000\t1000\n";
        assert_eq!(parse_effective_uid(status), Some(0));
        let status = "Uid:\t1000\t1000\t1000\t1000\n";
        assert_eq!(parse_effective_uid(status), Some(1000));
        assert_eq!(parse_effective_uid("no uid here\n"), None);
    }
}
