//! The shipped manifest (spec §3): what rides in the `.crate` beside the
//! code — snapshot identity, payload receipts (compressed/uncompressed
//! bytes and SHA-256 of each), the baseline membership, and the stub
//! accounting. Per-RFC records ride in the index payload (see `payload`).

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadReceipt {
    pub file: String,
    pub compressed_bytes: u64,
    pub uncompressed_bytes: u64,
    pub compressed_sha256: String,
    pub uncompressed_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub crate_name: String,
    pub crate_version: String,
    /// SHA-256 of the audited ietf.phext snapshot this build describes.
    pub snapshot_sha256: String,
    pub snapshot_records: u32,
    pub snapshot_texts: u32,
    pub core_payload: PayloadReceipt,
    pub index_payload: PayloadReceipt,
    /// Baseline core membership (§7.1) — RFC numbers, ascending.
    pub baseline: Vec<u32>,
    pub stub_count: u32,
    pub stub_total_bytes: u64,
    /// Legend-audit coverage behind the receipts in the index payload:
    /// how many issued texts were classified, of how many issued.
    pub legend_audit_classified: u32,
    pub legend_audit_issued: u32,
}

impl Manifest {
    pub fn load_shipped() -> Result<Manifest> {
        let raw = include_str!("../assets/manifest.json");
        let m: Manifest = serde_json::from_str(raw)
            .map_err(|e| Error::Other(format!("shipped manifest does not parse: {e}")))?;
        Ok(m)
    }
}

/// One per-RFC record, carried in the index payload. Metadata is factual,
/// taken from the RFC index via the corpus build of record (ietf-source.json);
/// `legend` is the §2 legend-audit class, determined by `rfcbox-build`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfcRecord {
    pub n: u32,
    pub title: String,
    /// Index status vocabulary: INTERNET STANDARD, PROPOSED STANDARD,
    /// INFORMATIONAL, HISTORIC, UNKNOWN, ... ("not issued" when no text).
    pub status: String,
    pub obsoletes: Vec<u32>,
    pub obsoleted_by: Vec<u32>,
    pub updates: Vec<u32>,
    pub updated_by: Vec<u32>,
    /// SHA-256 of the text as fetched; None when no text exists.
    pub sha256: Option<String>,
    pub text_bytes: Option<u64>,
    /// ok | not_issued | unavailable
    pub fetch_status: String,
    /// §2 legend-audit class; "unaudited" until the audit has run over it.
    pub legend: String,
    pub abstract_text: String,
    pub impl_count: u32,
    pub impl_names: Vec<String>,
    pub date: String,
}

/// The index payload (§3, §8.5): records for every corpus entry, the
/// reference graph, and one discovery stub per RFC not in the baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPayload {
    pub records: Vec<RfcRecord>,
    /// Forward reference edges by kind, keyed by RFC number (string form).
    pub references: std::collections::BTreeMap<String, RefEdges>,
    /// Reverse edges: RFC number -> who cites it, with kind.
    pub cited_by: std::collections::BTreeMap<String, Vec<CitedBy>>,
    pub stubs: Vec<Stub>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefEdges {
    pub normative: Vec<u32>,
    pub informative: Vec<u32>,
    pub unspecified: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedBy {
    pub from: u32,
    pub kind: String,
}

/// A discovery stub (§8.5): our generated metadata about an RFC — never the
/// RFC's text. Four parts, nothing more: number/title (in the record),
/// top-50 signature words, a paragraph-level summary, status/purpose.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stub {
    pub n: u32,
    pub words: Vec<String>,
    pub summary: String,
    pub purpose: String,
}

impl IndexPayload {
    pub fn record(&self, n: u32) -> Option<&RfcRecord> {
        self.records.iter().find(|r| r.n == n)
    }
    pub fn stub(&self, n: u32) -> Option<&Stub> {
        self.stubs.iter().find(|s| s.n == n)
    }
}
