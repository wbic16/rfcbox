//! Shipped payloads (§3): the index payload and the baseline-core payload
//! ride inside the crate xz-compressed; the crate inflates them itself and
//! verifies the uncompressed bytes against the manifest receipt before
//! trusting a single entry. The verbatim guarantee for core texts is
//! defined on the inflated bytes, and each text is then verified again
//! against its per-RFC SHA-256 from the index records (§2, §7.1).

use std::io::Read;

use crate::manifest::{IndexPayload, Manifest, PayloadReceipt};
use crate::{verify_hash, Error, Result};

static CORE_XZ: &[u8] = include_bytes!("../assets/core.xz");
static INDEX_XZ: &[u8] = include_bytes!("../assets/index.xz");

fn inflate(receipt: &PayloadReceipt, compressed: &[u8]) -> Result<Vec<u8>> {
    if compressed.len() as u64 != receipt.compressed_bytes {
        return Err(Error::HashRefusal(format!(
            "{}: shipped payload is {} bytes, manifest receipt says {}",
            receipt.file,
            compressed.len(),
            receipt.compressed_bytes
        )));
    }
    verify_hash(
        &format!("{} (compressed payload)", receipt.file),
        compressed,
        &receipt.compressed_sha256,
    )?;
    let mut dec = xz2::read::XzDecoder::new(compressed);
    let mut out = Vec::with_capacity(receipt.uncompressed_bytes as usize);
    dec.read_to_end(&mut out)
        .map_err(|e| Error::Other(format!("{}: xz inflate failed: {e}", receipt.file)))?;
    if out.len() as u64 != receipt.uncompressed_bytes {
        return Err(Error::HashRefusal(format!(
            "{}: inflated to {} bytes, manifest receipt says {}",
            receipt.file,
            out.len(),
            receipt.uncompressed_bytes
        )));
    }
    verify_hash(
        &format!("{} (inflated payload)", receipt.file),
        &out,
        &receipt.uncompressed_sha256,
    )?;
    Ok(out)
}

/// Inflate + verify the index payload and parse it.
pub fn load_index(manifest: &Manifest) -> Result<IndexPayload> {
    let raw = inflate(&manifest.index_payload, INDEX_XZ)?;
    let idx: IndexPayload = serde_json::from_slice(&raw)?;
    Ok(idx)
}

/// The baseline core texts, framed as repeated
/// `[u32 number][u32 len][verbatim bytes]` (little-endian), inflated and
/// verified as a payload; each text is additionally verified against its
/// per-RFC SHA-256 from the index records before it is returned.
pub fn load_core(manifest: &Manifest, index: &IndexPayload) -> Result<Vec<(u32, Vec<u8>)>> {
    let raw = inflate(&manifest.core_payload, CORE_XZ)?;
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < raw.len() {
        if pos + 8 > raw.len() {
            return Err(Error::Other("core payload: truncated frame header".into()));
        }
        let n = u32::from_le_bytes(raw[pos..pos + 4].try_into().unwrap());
        let len = u32::from_le_bytes(raw[pos + 4..pos + 8].try_into().unwrap()) as usize;
        pos += 8;
        if pos + len > raw.len() {
            return Err(Error::Other(format!("core payload: truncated text for RFC {n}")));
        }
        let text = raw[pos..pos + len].to_vec();
        pos += len;
        let rec = index
            .record(n)
            .ok_or_else(|| Error::Other(format!("core payload: RFC {n} has no index record")))?;
        let recorded = rec
            .sha256
            .as_deref()
            .ok_or_else(|| Error::Other(format!("core payload: RFC {n} record has no hash")))?;
        verify_hash(&format!("RFC {n} core text (post-inflation)"), &text, recorded)?;
        out.push((n, text));
    }
    Ok(out)
}

/// Frame core texts for shipment (used by rfcbox-build).
pub fn frame_core(texts: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (n, text) in texts {
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&(text.len() as u32).to_le_bytes());
        out.extend_from_slice(text);
    }
    out
}

/// xz-compress a payload at preset 6 (used by rfcbox-build).
pub fn deflate(bytes: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    let mut enc = xz2::write::XzEncoder::new(Vec::new(), 6);
    enc.write_all(bytes)?;
    let out = enc
        .finish()
        .map_err(|e| Error::Other(format!("xz deflate failed: {e}")))?;
    Ok(out)
}
