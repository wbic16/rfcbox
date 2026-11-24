//! rfcbox — a builder/reader library plus CLI over ietf.phext, the phext
//! corpus of every IETF RFC (10,052 records; audited snapshot 2026-11-18).
//!
//! Licensing is split by construction (spec §1): this code is MIT; the RFC
//! texts it carries, fetches, or emits are IETF/author material under the
//! IETF Trust (now IETF IPMC) Legal Provisions (TLP) and are never
//! relicensed. Texts travel verbatim, byte-for-byte as published by the
//! RFC Editor, and every text is verified against its recorded SHA-256
//! before it is served, sliced, or appended — a mismatch refuses loudly
//! (spec §2). Nothing in this crate is an IETF or RFC Editor product.

pub mod audit;
pub mod fetch;
pub mod install;
pub mod manifest;
pub mod payload;
pub mod project;
pub mod search;
pub mod text;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    /// A text or payload failed its recorded SHA-256. This is §2's loud
    /// refusal: it is never downgraded to a warning anywhere in the crate.
    HashRefusal(String),
    /// The set is frozen (§9.3); the operation was refused, with reason.
    Frozen(String),
    Fetch(String),
    Usage(String),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io: {e}"),
            Error::Json(e) => write!(f, "json: {e}"),
            Error::HashRefusal(m) => write!(f, "HASH REFUSAL: {m}"),
            Error::Frozen(m) => write!(f, "refused (set frozen): {m}"),
            Error::Fetch(m) => write!(f, "fetch: {m}"),
            Error::Usage(m) => write!(f, "usage: {m}"),
            Error::Other(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// SHA-256 of bytes, lowercase hex — the crate's one hash form.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// Verify bytes against a recorded hash; refuse loudly on mismatch (§2).
pub fn verify_hash(what: &str, bytes: &[u8], recorded: &str) -> Result<()> {
    let got = sha256_hex(bytes);
    if got != recorded {
        return Err(Error::HashRefusal(format!(
            "{what}: sha256 {got} != recorded {recorded}"
        )));
    }
    Ok(())
}
