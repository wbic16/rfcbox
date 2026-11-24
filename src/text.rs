//! Text handling: tokenization (shared by §8 search and §8.5 stub
//! building), reconstruction of original fetched bytes from a corpus
//! scroll, and the §2 legend classification.

use std::collections::HashSet;

/// Words that carry no findability signal — excluded from signature words
/// and intent terms by construction (§8.5).
pub fn stopwords() -> HashSet<&'static str> {
    [
        "the", "a", "an", "and", "or", "of", "to", "in", "is", "are", "be",
        "for", "on", "with", "as", "by", "this", "that", "it", "its", "from",
        "at", "an", "not", "no", "if", "when", "which", "such", "other",
        "than", "then", "thus", "may", "must", "should", "shall", "can",
        "will", "would", "one", "two", "all", "any", "each", "per", "via",
        "used", "using", "use", "also", "more", "into", "over", "under",
        "between", "within", "without", "upon", "about", "after", "before",
        "these", "those", "they", "them", "their", "there", "here", "been",
        "being", "were", "was", "has", "have", "had", "do", "does", "did",
        "but", "so", "we", "you", "he", "she", "i", "rfc", "section",
        "page", "internet", "draft", "memo", "category", "issn", "obsoletes",
        "updates", "status", "author", "authors", "copyright", "reserved",
        "rights", "document", "documents", "protocol", "protocols",
    ]
    .into_iter()
    .collect()
}

/// Lowercase word tokens: runs of [a-z0-9] with internal '-'/'_' kept when
/// flanked by alphanumerics (so "http/2" -> http, 2; "content-length"
/// stays one token). Length >= 2.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let bytes = text.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let c = b as char;
        if c.is_ascii_alphanumeric() {
            cur.push(c.to_ascii_lowercase());
        } else if (c == '-' || c == '_') && !cur.is_empty()
            && i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_alphanumeric()
        {
            cur.push(c);
        } else {
            if cur.len() >= 2 {
                out.push(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() >= 2 {
        out.push(cur);
    }
    out
}

/// Extract the verbatim text from a corpus scroll and reconstruct the
/// original fetched bytes. The scroll is a MOAT header, then the marker
/// line `--- RFC TEXT AS SERVED BY THE RFC EDITOR ---`, then the text.
/// Phext-delimiter bytes in the source are stored as `[0xNN]` markers;
/// unescaping restores the original bytes (valid when the source was
/// UTF-8; Latin-1-normalized scrolls cannot be byte-reconstructed and
/// are the caller's stated exception class — see the build report).
pub fn scroll_text_bytes(scroll: &str) -> Option<Vec<u8>> {
    let marker = "--- RFC TEXT AS SERVED BY THE RFC EDITOR ---\n";
    let idx = scroll.find(marker)?;
    let body = &scroll[idx + marker.len()..];
    // The builder joins header + "\n" + marker-line + "\n" + text; the
    // marker find above consumed through the marker's own newline, and
    // the text follows after one blank line.
    let body = body.strip_prefix('\n').unwrap_or(body);
    Some(unescape_markers(body))
}

fn unescape_markers(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if i + 5 < b.len()
            && b[i] == b'['
            && b[i + 1] == b'0'
            && b[i + 2] == b'x'
            && b[i + 5] == b']'
            && (b[i + 3] as char).is_ascii_hexdigit()
            && (b[i + 4] as char).is_ascii_hexdigit()
        {
            let hex = std::str::from_utf8(&b[i + 3..i + 5]).unwrap();
            out.push(u8::from_str_radix(hex, 16).unwrap());
            i += 6;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

/// §2 legend classes. Determined by scanning the text itself; the class
/// names are the manifest's vocabulary.
/// - `restrictive-no-derivatives`: carries the no-modification /
///   no-derivative-works legend (the §6.c / pre-5378 class) — its code
///   components are NOT BSD-available.
/// - `earlier-era`: published before the TLP era (pre-Nov-2008 texts sit
///   under their era's policy: RFC 2026 / 3978 / 4748 terms).
/// - `standard-tlp`: current-terms text; code components ride TLP §4's
///   Revised BSD License.
pub fn legend_class(text: &str, year: Option<i32>) -> &'static str {
    let lower = text.to_lowercase();
    if lower.contains("may not be modified, and derivative works of it may not be created")
        || lower.contains("may not be modified and derivative works of it may not be")
        || lower.contains("no derivative works are permitted")
    {
        return "restrictive-no-derivatives";
    }
    if let Some(y) = year {
        if y < 2008 {
            return "earlier-era";
        }
    }
    "standard-tlp"
}

/// Parse a year out of an index date string like "April 1969" / "May 2020".
pub fn year_of(date: &str) -> Option<i32> {
    date.split_whitespace()
        .filter_map(|w| w.parse::<i32>().ok())
        .find(|y| (1950..=2100).contains(y))
}

/// A paragraph-level summary candidate from the text itself: the RFC's
/// own Abstract section when present (used by the stub builder when the
/// index record carries no abstract), whitespace-collapsed.
pub fn abstract_from_text(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let start = lower.find("\nabstract")?;
    let rest = &text[start + 1..];
    // Abstract body begins after the heading line.
    let nl = rest.find('\n')?;
    let body = &rest[nl + 1..];
    let mut para = String::new();
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() {
            if !para.is_empty() {
                break;
            }
            continue;
        }
        // A numbered section heading ends the abstract.
        if t.chars().next().is_some_and(|c| c.is_ascii_digit())
            && t.contains('.') && t.len() < 80 && para.len() > 200
        {
            break;
        }
        if !para.is_empty() {
            para.push(' ');
        }
        para.push_str(t);
        if para.len() > 1200 {
            break;
        }
    }
    if para.len() >= 80 {
        Some(para)
    } else {
        None
    }
}
