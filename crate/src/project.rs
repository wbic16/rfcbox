//! The target project's declared set (§7.2): `rfcs.json` at the project
//! root is the source of truth; `.rfcbox/` holds the materialization and
//! derived state. rfcbox itself carries no `rfcs.json`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{sha256_hex, Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEntry {
    pub number: u32,
    /// SHA-256 of the text as fetched (the §2 enforcement hash).
    pub sha256: String,
    /// Where the text came from: baseline-core | corpus:<path> |
    /// https://www.rfc-editor.org/rfc/rfcN.txt | ...
    pub source: String,
    /// Supersession state: null when current, else the successor.
    #[serde(default)]
    pub superseded_by: Option<u32>,
    /// baseline | add | closure-add | found-by-scan | sync
    pub origin: String,
    #[serde(default)]
    pub origin_detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFile {
    pub version: u32,
    pub rfcs: Vec<SetEntry>,
}

impl Default for SetFile {
    fn default() -> Self {
        Self::new()
    }
}

impl SetFile {
    pub fn new() -> SetFile {
        SetFile { version: 1, rfcs: Vec::new() }
    }
    pub fn get(&self, n: u32) -> Option<&SetEntry> {
        self.rfcs.iter().find(|e| e.number == n)
    }
    pub fn contains(&self, n: u32) -> bool {
        self.get(n).is_some()
    }
    /// Deterministic set hash (§9.1): over the sorted declaration.
    pub fn set_hash(&self) -> String {
        let mut nums: Vec<u32> = self.rfcs.iter().map(|e| e.number).collect();
        nums.sort_unstable();
        let mut canon = String::new();
        for n in nums {
            let e = self.get(n).unwrap();
            canon.push_str(&format!(
                "{}:{}:{:?}:{};",
                e.number, e.sha256, e.superseded_by, e.origin
            ));
        }
        sha256_hex(canon.as_bytes())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectState {
    pub frozen: bool,
    /// §9.3: notices accumulated while frozen, carried into next sync.
    pub pending_notices: Vec<String>,
    pub history: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub name: String,
    pub date: String,
    pub set_hash: String,
    pub snapshot_sha256: String,
    pub set: SetFile,
}

pub struct Project {
    pub root: PathBuf,
}

impl Project {
    pub fn new(root: PathBuf) -> Project {
        Project { root }
    }
    pub fn set_path(&self) -> PathBuf {
        self.root.join("rfcs.json")
    }
    pub fn state_dir(&self) -> PathBuf {
        self.root.join(".rfcbox")
    }
    pub fn texts_dir(&self) -> PathBuf {
        self.state_dir().join("texts")
    }
    pub fn text_path(&self, n: u32) -> PathBuf {
        self.texts_dir().join(format!("rfc{n}.txt"))
    }
    pub fn baselines_dir(&self) -> PathBuf {
        self.state_dir().join("baselines")
    }
    pub fn reports_dir(&self) -> PathBuf {
        self.state_dir().join("reports")
    }

    pub fn load_set(&self) -> Result<Option<SetFile>> {
        let p = self.set_path();
        if !p.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&p)?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    pub fn require_set(&self) -> Result<SetFile> {
        self.load_set()?.ok_or_else(|| {
            Error::Usage(format!(
                "no rfcs.json in {} — run `rfcbox init` or `rfcbox add` first",
                self.root.display()
            ))
        })
    }

    pub fn save_set(&self, set: &SetFile) -> Result<()> {
        let raw = serde_json::to_string_pretty(set)?;
        std::fs::write(self.set_path(), raw + "\n")?;
        Ok(())
    }

    pub fn load_state(&self) -> Result<ProjectState> {
        let p = self.state_dir().join("state.json");
        if !p.exists() {
            return Ok(ProjectState::default());
        }
        Ok(serde_json::from_str(&std::fs::read_to_string(&p)?)?)
    }

    pub fn save_state(&self, st: &ProjectState) -> Result<()> {
        std::fs::create_dir_all(self.state_dir())?;
        let raw = serde_json::to_string_pretty(st)?;
        std::fs::write(self.state_dir().join("state.json"), raw + "\n")?;
        Ok(())
    }

    /// Materialize a text: verify-then-write. The caller supplies the
    /// expected hash (the index record's, for snapshot texts); a mismatch
    /// refuses before anything touches disk (§2).
    pub fn materialize(&self, n: u32, bytes: &[u8], expected_sha: &str) -> Result<()> {
        crate::verify_hash(&format!("RFC {n} materialization"), bytes, expected_sha)?;
        std::fs::create_dir_all(self.texts_dir())?;
        std::fs::write(self.text_path(n), bytes)?;
        Ok(())
    }

    pub fn dematerialize(&self, n: u32) -> Result<()> {
        let p = self.text_path(n);
        if p.exists() {
            std::fs::remove_file(&p)?;
        }
        Ok(())
    }

    pub fn materialized(&self, n: u32) -> Option<Vec<u8>> {
        std::fs::read(self.text_path(n)).ok()
    }

    /// All materialized texts, hash-verified against the declaration
    /// before use; a mismatch is a loud refusal (§2), never a skip.
    pub fn materialized_texts(&self, set: &SetFile) -> Result<Vec<(u32, Vec<u8>)>> {
        let mut out = Vec::new();
        for e in &set.rfcs {
            if let Some(bytes) = self.materialized(e.number) {
                crate::verify_hash(
                    &format!("RFC {} materialized text", e.number),
                    &bytes,
                    &e.sha256,
                )?;
                out.push((e.number, bytes));
            }
        }
        Ok(out)
    }

    /// §8.1: the inverted index over the local materialization is built
    /// at add/sync time, not query time. Derived state; rebuildable.
    pub fn rebuild_local_index(&self, set: &SetFile) -> Result<()> {
        let texts = self.materialized_texts(set)?;
        let mut postings: BTreeMap<String, Vec<(u32, u32)>> = BTreeMap::new();
        for (n, bytes) in &texts {
            let text = String::from_utf8_lossy(bytes);
            let mut counts: BTreeMap<String, u32> = BTreeMap::new();
            for tok in crate::text::tokenize(&text) {
                *counts.entry(tok).or_default() += 1;
            }
            for (term, freq) in counts {
                postings.entry(term).or_default().push((*n, freq));
            }
        }
        std::fs::create_dir_all(self.state_dir())?;
        let raw = serde_json::to_string(&postings)?;
        std::fs::write(self.state_dir().join("index.json"), raw)?;
        Ok(())
    }

    pub fn save_baseline(&self, b: &Baseline) -> Result<()> {
        std::fs::create_dir_all(self.baselines_dir())?;
        let raw = serde_json::to_string_pretty(b)?;
        std::fs::write(
            self.baselines_dir().join(format!("{}.json", b.name)),
            raw + "\n",
        )?;
        Ok(())
    }

    pub fn load_baseline(&self, name: &str) -> Result<Baseline> {
        let p = self.baselines_dir().join(format!("{name}.json"));
        if !p.exists() {
            return Err(Error::Usage(format!("no baseline named {name}")));
        }
        Ok(serde_json::from_str(&std::fs::read_to_string(&p)?)?)
    }

    pub fn list_baselines(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(self.baselines_dir()) {
            for e in rd.flatten() {
                if let Some(name) = e.file_name().to_str() {
                    if let Some(stem) = name.strip_suffix(".json") {
                        out.push(stem.to_string());
                    }
                }
            }
        }
        out.sort();
        out
    }
}

pub fn today_utc() -> String {
    // No chrono dependency: derive from the system clock via `date`.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86400) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn project_root_arg(args: &[String]) -> PathBuf {
    for i in 0..args.len() {
        if args[i] == "--project" && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
    }
    PathBuf::from(".")
}

pub fn strip_project_arg(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--project" && i + 1 < args.len() {
            i += 2;
        } else {
            out.push(args[i].clone());
            i += 1;
        }
    }
    out
}
