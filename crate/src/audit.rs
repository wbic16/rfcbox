//! `audit` — two legs, both read-only (fetches nothing):
//! 1. §9.4 verify semantics over the target project's declared set:
//!    every declared RFC materialized locally, hash-verified under §2
//!    (a mismatch fails loudly and headlines the report), and
//!    status-checked as of a stated date.
//! 2. §2 legend audit receipts for the declared set: each member's
//!    legend class as recorded in the shipped manifest's audit.

use crate::manifest::IndexPayload;
use crate::project::{Project, SetFile};
use crate::Result;

#[derive(Debug, Default)]
pub struct AuditReport {
    pub as_of: String,
    pub pass: Vec<String>,
    pub gaps: Vec<String>,
    pub hash_failures: Vec<String>,
    pub legend_lines: Vec<String>,
}

impl AuditReport {
    pub fn clean(&self) -> bool {
        self.gaps.is_empty() && self.hash_failures.is_empty()
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("rfcbox audit — as of {}\n", self.as_of));
        if !self.hash_failures.is_empty() {
            out.push_str("\n!! HASH FAILURES (headline — §2 refuses on mismatch) !!\n");
            for h in &self.hash_failures {
                out.push_str(&format!("  FAIL {h}\n"));
            }
        }
        out.push_str(&format!("\npass ({}):\n", self.pass.len()));
        for p in &self.pass {
            out.push_str(&format!("  ok   {p}\n"));
        }
        out.push_str(&format!("\ngaps ({}):\n", self.gaps.len()));
        for g in &self.gaps {
            out.push_str(&format!("  gap  {g}\n"));
        }
        out.push_str("\nlegend audit receipts (§2):\n");
        for l in &self.legend_lines {
            out.push_str(&format!("  {l}\n"));
        }
        out.push_str(&format!(
            "\nverdict: {}\n",
            if self.clean() {
                "PASS — declared set materialized, hash-verified, status-checked"
            } else {
                "GAPS/FAILURES PRESENT — see items above (repair is sync/add's work, done deliberately)"
            }
        ));
        out
    }
}

pub fn audit_project(
    project: &Project,
    set: &SetFile,
    index: &IndexPayload,
    as_of: &str,
) -> Result<AuditReport> {
    let mut rep = AuditReport {
        as_of: as_of.to_string(),
        ..Default::default()
    };
    for e in &set.rfcs {
        let rec = index.record(e.number);
        // Legend receipt.
        match rec {
            Some(r) => rep.legend_lines.push(format!(
                "RFC {} — legend class: {} (text sha256 {})",
                e.number,
                r.legend,
                r.sha256.as_deref().unwrap_or("none")
            )),
            None => rep
                .legend_lines
                .push(format!("RFC {} — no index record; legend unaudited", e.number)),
        }
        // Materialized?
        match project.materialized(e.number) {
            None => rep.gaps.push(format!(
                "RFC {} declared but not materialized",
                e.number
            )),
            Some(bytes) => {
                // Hash-verified under §2 — against the declaration, and
                // against the shipped index record when one exists.
                let got = crate::sha256_hex(&bytes);
                if got != e.sha256 {
                    rep.hash_failures.push(format!(
                        "RFC {}: materialized sha256 {got} != declared {}",
                        e.number, e.sha256
                    ));
                    continue;
                }
                if let Some(r) = rec {
                    if let Some(rs) = &r.sha256 {
                        if *rs != got {
                            rep.hash_failures.push(format!(
                                "RFC {}: materialized sha256 {got} != audited snapshot {rs}",
                                e.number
                            ));
                            continue;
                        }
                    }
                }
                // Status as of the stated date (the shipped snapshot's
                // index state; sync is what learns newer state).
                let status = rec
                    .map(|r| {
                        if r.obsoleted_by.is_empty() {
                            format!("current ({})", r.status)
                        } else {
                            format!(
                                "superseded by {} ({})",
                                r.obsoleted_by
                                    .iter()
                                    .map(|x| format!("RFC {x}"))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                r.status
                            )
                        }
                    })
                    .unwrap_or_else(|| "no index record".into());
                rep.pass.push(format!(
                    "RFC {} — materialized, hash-verified, status: {}",
                    e.number, status
                ));
            }
        }
    }
    Ok(rep)
}
