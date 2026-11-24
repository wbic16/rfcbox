//! rfcbox-build — the maintainer tool that produces the crate's shipped
//! payloads from the audited corpus (spec §3, §7.1, §8.5, §2):
//!
//!   rfcbox-build --phext ietf.phext --source ietf-source.json --out CRATE_DIR
//!
//! It parses the corpus through libphext (the only phext parser), verifies
//! every text it touches against the source receipts, runs the §2 legend
//! audit over every issued text, computes TF-IDF signature words, builds
//! the §8.5 discovery stubs (per-stub budget enforced — an overrun fails
//! the build), frames + xz-compresses the baseline core and the index
//! payload, and writes assets/manifest.json with full receipts.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use serde::Deserialize;

use rfcbox::manifest::{CitedBy, IndexPayload, Manifest, PayloadReceipt, RefEdges, RfcRecord, Stub};
use rfcbox::payload::{deflate, frame_core};
use rfcbox::text::{abstract_from_text, legend_class, scroll_text_bytes, stopwords, tokenize, year_of};
use rfcbox::{sha256_hex, Error, Result};

/// The default slice (§7.1 as directed 2026-11-23): a small set of common
/// web development RFCs. Seeds named by Will; membership adjusted only on
/// audit evidence, with receipts printed by this tool:
/// - 8446 (TLS 1.3) is superseded by 9846 in the corpus index -> the
///   successor is the slice member; 8446's stub carries the redirect.
/// - 9112 (HTTP/1.1, INTERNET STANDARD, current) completes the HTTP core
///   beside 9110/9111/9113/9114.
const BASELINE_SEEDS: &[u32] = &[
    9110, 9111, 9112, 9113, 9114, 9000, 8446, 3986, 6455, 8259, 6265, 6749, 6750, 7519, 1035, 9293,
];

const STUB_BUDGET_BYTES: usize = 1024;

#[derive(Debug, Deserialize)]
struct Source {
    entries: Vec<SourceEntry>,
}

#[derive(Debug, Deserialize)]
struct SourceEntry {
    rfc: u32,
    title: String,
    date: String,
    current_status: String,
    obsoletes: Vec<u32>,
    obsoleted_by: Vec<u32>,
    updates: Vec<u32>,
    updated_by: Vec<u32>,
    fetch_status: String,
    text_sha256: Option<String>,
    text_bytes: Option<u64>,
    utf8: bool,
    delim_bytes: u32,
    #[serde(default)]
    abstract_text: Option<String>,
    #[serde(default)]
    implementations: Vec<SourceImpl>,
    #[serde(default)]
    references: Vec<SourceRef>,
    #[serde(default)]
    cited_by: Vec<SourceCitedBy>,
}

#[derive(Debug, Deserialize)]
struct SourceImpl {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SourceRef {
    to_rfc: u32,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct SourceCitedBy {
    from_rfc: u32,
    kind: String,
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::from(0),
        Err(e) => {
            eprintln!("rfcbox-build error: {e}");
            std::process::ExitCode::from(1)
        }
    }
}

fn arg(args: &[String], name: &str) -> Result<String> {
    for i in 0..args.len() {
        if args[i] == name && i + 1 < args.len() {
            return Ok(args[i + 1].clone());
        }
    }
    Err(Error::Usage(format!("missing {name}")))
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let phext_path = arg(&args, "--phext")?;
    let source_path = arg(&args, "--source")?;
    let out_dir = PathBuf::from(arg(&args, "--out")?);

    eprintln!("reading source metadata {source_path} ...");
    let source: Source = serde_json::from_str(&std::fs::read_to_string(&source_path)?)?;
    let by_n: HashMap<u32, &SourceEntry> = source.entries.iter().map(|e| (e.rfc, e)).collect();

    // --- baseline membership, audit-adjusted (§7.1) ---
    let mut baseline: Vec<u32> = Vec::new();
    for seed in BASELINE_SEEDS {
        let e = by_n
            .get(seed)
            .ok_or_else(|| Error::Other(format!("baseline seed {seed} absent from corpus")))?;
        if e.fetch_status != "ok" {
            return Err(Error::Other(format!(
                "baseline seed {seed} has no text (fetch_status {})",
                e.fetch_status
            )));
        }
        if !e.obsoleted_by.is_empty() {
            // Supersession discipline: the successor is the member; the
            // superseded seed stays findable via its stub + annotation.
            for succ in &e.obsoleted_by {
                let se = by_n.get(succ).ok_or_else(|| {
                    Error::Other(format!("successor {succ} of seed {seed} absent"))
                })?;
                if se.fetch_status != "ok" {
                    return Err(Error::Other(format!(
                        "successor {succ} of seed {seed} has no text"
                    )));
                }
                eprintln!(
                    "baseline audit: seed {seed} superseded by {succ} — successor enrolled, seed annotated"
                );
                if !baseline.contains(succ) {
                    baseline.push(*succ);
                }
            }
        } else {
            baseline.push(*seed);
        }
    }
    baseline.sort_unstable();
    eprintln!("baseline core: {} members {:?}", baseline.len(), baseline);

    // --- parse the corpus through libphext ---
    eprintln!("reading corpus {phext_path} ...");
    let raw = std::fs::read_to_string(&phext_path)?;
    let snapshot_sha = sha256_hex(raw.as_bytes());
    eprintln!("corpus sha256 {snapshot_sha}");
    eprintln!("exploding via libphext ...");
    let map = libphext::phext::explode(&raw);
    eprintln!("{} scrolls", map.len());
    let mut scrolls: HashMap<u32, String> = HashMap::new();
    for (coord, scroll) in &map {
        let s = coord.to_string();
        if let Some(tail) = s.strip_prefix("1.1.1/1.2.1/1.1.") {
            if let Ok(n) = tail.parse::<u32>() {
                scrolls.insert(n, scroll.clone());
            }
        }
    }

    // --- extract + verify texts; legend audit; term statistics ---
    let stops = stopwords();
    let mut texts: HashMap<u32, Vec<u8>> = HashMap::new();
    let mut legend: HashMap<u32, &'static str> = HashMap::new();
    let mut unverifiable: Vec<u32> = Vec::new();
    let mut df: HashMap<String, u32> = HashMap::new();
    let mut doc_terms: HashMap<u32, HashMap<String, u32>> = HashMap::new();
    let issued = source
        .entries
        .iter()
        .filter(|e| e.fetch_status != "not_issued")
        .count() as u32;
    let mut classified = 0u32;
    for e in &source.entries {
        if e.fetch_status != "ok" {
            continue;
        }
        let scroll = scrolls
            .get(&e.rfc)
            .ok_or_else(|| Error::Other(format!("RFC {}: scroll missing from corpus", e.rfc)))?;
        let bytes = scroll_text_bytes(scroll)
            .ok_or_else(|| Error::Other(format!("RFC {}: no text marker in scroll", e.rfc)))?;
        // §2 verification against the source receipt. Latin-1-normalized
        // scrolls (utf8=false) cannot be byte-reconstructed from the
        // phext alone — the stated exception class; they are audited by
        // class from their stored text and never enter the core/slices.
        if e.utf8 {
            let recorded = e
                .text_sha256
                .as_deref()
                .ok_or_else(|| Error::Other(format!("RFC {}: no recorded hash", e.rfc)))?;
            rfcbox::verify_hash(&format!("RFC {} corpus text", e.rfc), &bytes, recorded)?;
        } else {
            unverifiable.push(e.rfc);
        }
        let text = String::from_utf8_lossy(&bytes).into_owned();
        legend.insert(e.rfc, legend_class(&text, year_of(&e.date)));
        classified += 1;
        let mut counts: HashMap<String, u32> = HashMap::new();
        for tok in tokenize(&text) {
            if stops.contains(tok.as_str()) || tok.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            *counts.entry(tok).or_default() += 1;
        }
        for term in counts.keys() {
            *df.entry(term.clone()).or_default() += 1;
        }
        doc_terms.insert(e.rfc, counts);
        texts.insert(e.rfc, bytes);
    }
    eprintln!(
        "legend audit: {classified}/{issued} issued texts classified; {} in the Latin-1 exception class",
        unverifiable.len()
    );
    let mut class_counts: BTreeMap<&str, u32> = BTreeMap::new();
    for v in legend.values() {
        *class_counts.entry(v).or_default() += 1;
    }
    eprintln!("legend classes: {class_counts:?}");

    // --- records + graph ---
    let mut records: Vec<RfcRecord> = Vec::new();
    let mut references: BTreeMap<String, RefEdges> = BTreeMap::new();
    let mut cited_by: BTreeMap<String, Vec<CitedBy>> = BTreeMap::new();
    for e in &source.entries {
        records.push(RfcRecord {
            n: e.rfc,
            title: e.title.clone(),
            status: if e.fetch_status == "not_issued" {
                "not issued".into()
            } else {
                e.current_status.clone()
            },
            obsoletes: e.obsoletes.clone(),
            obsoleted_by: e.obsoleted_by.clone(),
            updates: e.updates.clone(),
            updated_by: e.updated_by.clone(),
            sha256: e.text_sha256.clone(),
            text_bytes: e.text_bytes,
            fetch_status: e.fetch_status.clone(),
            legend: legend
                .get(&e.rfc)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unaudited".into()),
            abstract_text: e.abstract_text.clone().unwrap_or_default(),
            impl_count: e.implementations.len() as u32,
            impl_names: e.implementations.iter().map(|i| i.name.clone()).collect(),
            date: e.date.clone(),
        });
        if !e.references.is_empty() {
            let mut edges = RefEdges::default();
            for r in &e.references {
                match r.kind.as_str() {
                    "normative" => edges.normative.push(r.to_rfc),
                    "informative" => edges.informative.push(r.to_rfc),
                    _ => edges.unspecified.push(r.to_rfc),
                }
            }
            references.insert(e.rfc.to_string(), edges);
        }
        if !e.cited_by.is_empty() {
            cited_by.insert(
                e.rfc.to_string(),
                e.cited_by
                    .iter()
                    .map(|c| CitedBy { from: c.from_rfc, kind: c.kind.clone() })
                    .collect(),
            );
        }
    }
    records.sort_by_key(|r| r.n);

    // --- stubs (§8.5): TF-IDF top-50, summary, purpose; budget enforced ---
    let n_docs = doc_terms.len() as f64;
    let baseline_set: HashSet<u32> = baseline.iter().copied().collect();
    let mut stubs: Vec<Stub> = Vec::new();
    let mut stub_total_bytes = 0u64;
    for e in &source.entries {
        if e.fetch_status != "ok" || baseline_set.contains(&e.rfc) {
            continue;
        }
        let counts = doc_terms.get(&e.rfc).cloned().unwrap_or_default();
        let mut scored: Vec<(String, f64)> = counts
            .iter()
            .map(|(t, f)| {
                let d = *df.get(t).unwrap_or(&1) as f64;
                (t.clone(), (*f as f64).ln_1p() * ((n_docs + 1.0) / (d + 1.0)).ln())
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let words: Vec<String> = scored.into_iter().take(50).map(|(t, _)| t).collect();
        let text = texts
            .get(&e.rfc)
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .unwrap_or_default();
        let mut summary = if !e.abstract_text.as_deref().unwrap_or("").is_empty() {
            e.abstract_text.clone().unwrap()
        } else if let Some(a) = abstract_from_text(&text) {
            a
        } else {
            format!(
                "{} — {} ({}). Signature vocabulary: {}.",
                e.title,
                e.current_status,
                e.date,
                words.iter().take(12).cloned().collect::<Vec<_>>().join(", ")
            )
        };
        summary = summary.split_whitespace().collect::<Vec<_>>().join(" ");
        let purpose = {
            let first_sentence = summary
                .split(". ")
                .next()
                .unwrap_or("")
                .chars()
                .take(160)
                .collect::<String>();
            if first_sentence.len() >= 40 {
                first_sentence
            } else {
                format!("{} — the {} specification.", e.title, e.current_status)
            }
        };
        // Budget: the stub's JSON must fit; trim the summary until it
        // does. An overrun that cannot be trimmed fails the build.
        loop {
            let stub = Stub {
                n: e.rfc,
                words: words.clone(),
                summary: summary.clone(),
                purpose: purpose.clone(),
            };
            let size = serde_json::to_string(&stub)?.len();
            if size <= STUB_BUDGET_BYTES {
                stub_total_bytes += size as u64;
                stubs.push(stub);
                break;
            }
            if summary.len() > 120 {
                let cut = summary.len() / 2;
                summary = summary.chars().take(cut).collect();
                continue;
            }
            return Err(Error::Other(format!(
                "stub for RFC {} overruns the {}-byte budget even trimmed ({} bytes) — build fails (§8.5)",
                e.rfc, STUB_BUDGET_BYTES, size
            )));
        }
    }
    eprintln!(
        "stubs: {} built, {} total bytes (budget {} B/stub)",
        stubs.len(),
        stub_total_bytes,
        STUB_BUDGET_BYTES
    );

    // --- payloads ---
    let core_texts: Vec<(u32, Vec<u8>)> = baseline
        .iter()
        .map(|n| (*n, texts.get(n).unwrap().clone()))
        .collect();
    let core_raw = frame_core(&core_texts);
    let core_xz = deflate(&core_raw)?;
    let index_payload = IndexPayload {
        records,
        references,
        cited_by,
        stubs,
    };
    let index_raw = serde_json::to_vec(&index_payload)?;
    let index_xz = deflate(&index_raw)?;
    let manifest = Manifest {
        crate_name: "rfcbox".into(),
        crate_version: env!("CARGO_PKG_VERSION").into(),
        snapshot_sha256: snapshot_sha,
        snapshot_records: source.entries.len() as u32,
        snapshot_texts: texts.len() as u32,
        core_payload: PayloadReceipt {
            file: "assets/core.xz".into(),
            compressed_bytes: core_xz.len() as u64,
            uncompressed_bytes: core_raw.len() as u64,
            compressed_sha256: sha256_hex(&core_xz),
            uncompressed_sha256: sha256_hex(&core_raw),
        },
        index_payload: PayloadReceipt {
            file: "assets/index.xz".into(),
            compressed_bytes: index_xz.len() as u64,
            uncompressed_bytes: index_raw.len() as u64,
            compressed_sha256: sha256_hex(&index_xz),
            uncompressed_sha256: sha256_hex(&index_raw),
        },
        baseline,
        stub_count: index_payload.stubs.len() as u32,
        stub_total_bytes,
        legend_audit_classified: classified,
        legend_audit_issued: issued,
    };
    let assets = out_dir.join("assets");
    std::fs::create_dir_all(&assets)?;
    std::fs::write(assets.join("core.xz"), &core_xz)?;
    std::fs::write(assets.join("index.xz"), &index_xz)?;
    std::fs::write(
        assets.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;
    eprintln!(
        "payloads written: core.xz {} B (raw {}), index.xz {} B (raw {}); package payload total {} B",
        core_xz.len(),
        core_raw.len(),
        index_xz.len(),
        index_raw.len(),
        core_xz.len() + index_xz.len()
    );
    Ok(())
}
