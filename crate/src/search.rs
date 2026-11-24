//! §8 search — keywords and intent. Search runs entirely over material
//! shipped in the crate or already on the user's disk; it is a read path,
//! never a fetch path, and it never adds anything by itself (§8.3).
//! Intent results are a ranking, labelled a ranking, and every result
//! carries its why: matched terms, the graph edges that connect it to
//! the matched set, its supersession state, and its implementation
//! links (§8.2, §8.4).

use std::collections::{HashMap, HashSet};

use crate::manifest::IndexPayload;
use crate::text::{stopwords, tokenize};

#[derive(Debug, Clone, PartialEq)]
pub enum Surface {
    FullText,
    Stub,
}

#[derive(Debug)]
pub struct Doc {
    pub n: u32,
    pub surface: Surface,
    /// Searchable text: the verbatim text, or the stub's surface
    /// (signature words + summary + purpose) plus title and abstract.
    pub text: String,
}

#[derive(Debug)]
pub struct Hit {
    pub n: u32,
    pub title: String,
    pub score: f64,
    pub stub_only: bool,
    pub in_set: bool,
    pub superseded_by: Vec<u32>,
    pub whys: Vec<String>,
    pub passages: Vec<String>,
}

pub struct Engine {
    docs: HashMap<u32, Doc>,
    /// term -> (rfc, freq)
    postings: HashMap<String, Vec<(u32, u32)>>,
    df: HashMap<String, u32>,
    n_docs: usize,
}

impl Engine {
    /// Build the runtime index over the surfaces given: full texts where
    /// present (baseline core + the target project's materialized set),
    /// discovery stubs for everything else in the corpus.
    pub fn build(
        index: &IndexPayload,
        full_texts: &[(u32, Vec<u8>)],
        in_set: &HashSet<u32>,
    ) -> Engine {
        let mut docs: HashMap<u32, Doc> = HashMap::new();
        let full: HashMap<u32, &Vec<u8>> = full_texts.iter().map(|(n, t)| (*n, t)).collect();
        for rec in &index.records {
            if rec.fetch_status != "ok" {
                continue;
            }
            if let Some(bytes) = full.get(&rec.n) {
                let mut text = String::from_utf8_lossy(bytes).into_owned();
                text.push('\n');
                text.push_str(&rec.title);
                docs.insert(
                    rec.n,
                    Doc { n: rec.n, surface: Surface::FullText, text },
                );
            } else if let Some(stub) = index.stub(rec.n) {
                let mut text = String::new();
                text.push_str(&rec.title);
                text.push('\n');
                // Signature words are the stub's fingerprint: repeat them
                // so they weigh as the document's characteristic terms.
                text.push_str(&stub.words.join(" "));
                text.push(' ');
                text.push_str(&stub.words.join(" "));
                text.push('\n');
                text.push_str(&stub.summary);
                text.push('\n');
                text.push_str(&stub.purpose);
                text.push('\n');
                text.push_str(&rec.abstract_text);
                docs.insert(rec.n, Doc { n: rec.n, surface: Surface::Stub, text });
            }
        }
        let _ = in_set;
        let mut postings: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
        let mut df: HashMap<String, u32> = HashMap::new();
        for doc in docs.values() {
            let mut counts: HashMap<String, u32> = HashMap::new();
            for tok in tokenize(&doc.text) {
                *counts.entry(tok).or_default() += 1;
            }
            for (term, freq) in counts {
                df.entry(term.clone()).or_insert(0);
                *df.get_mut(&term).unwrap() += 1;
                postings.entry(term).or_default().push((doc.n, freq));
            }
        }
        let n_docs = docs.len();
        Engine { docs, postings, df, n_docs }
    }

    fn idf(&self, term: &str) -> f64 {
        let d = *self.df.get(term).unwrap_or(&1) as f64;
        ((self.n_docs as f64 + 1.0) / (d + 1.0)).ln() + 0.1
    }

    /// Keyword scores for a term set: rfc -> (score, matched terms).
    fn keyword_scores(&self, terms: &[String]) -> HashMap<u32, (f64, Vec<String>)> {
        let mut out: HashMap<u32, (f64, Vec<String>)> = HashMap::new();
        for term in terms {
            if let Some(posts) = self.postings.get(term) {
                let w = self.idf(term);
                for (n, freq) in posts {
                    let e = out.entry(*n).or_default();
                    e.0 += (*freq as f64).ln_1p() * w;
                    if !e.1.contains(term) {
                        e.1.push(term.clone());
                    }
                }
            }
        }
        out
    }

    fn passages(&self, doc: &Doc, terms: &[String], max: usize) -> Vec<String> {
        if doc.surface == Surface::Stub {
            // The stub's summary paragraph is its passage; the caller
            // labels it "summary only".
            let stub_text = doc.text.clone();
            return vec![collapse(&stub_text.chars().take(600).collect::<String>())];
        }
        let lower = doc.text.to_lowercase();
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for term in terms {
            let mut from = 0usize;
            while let Some(rel) = lower[from..].find(term.as_str()) {
                let pos = from + rel;
                let start = pos.saturating_sub(90);
                let end = (pos + term.len() + 130).min(doc.text.len());
                spans.push((start, end));
                from = pos + term.len();
                if spans.len() >= max * 3 {
                    break;
                }
            }
            if spans.len() >= max * 3 {
                break;
            }
        }
        spans.sort();
        let mut out: Vec<String> = Vec::new();
        let mut last_end = 0usize;
        for (s, e) in spans {
            if s < last_end {
                continue;
            }
            last_end = e;
            let slice = &doc.text[s..e];
            out.push(collapse(slice));
            if out.len() >= max {
                break;
            }
        }
        out
    }

    fn make_hit(
        &self,
        index: &IndexPayload,
        n: u32,
        score: f64,
        whys: Vec<String>,
        terms: &[String],
        in_set: &HashSet<u32>,
    ) -> Hit {
        let rec = index.record(n);
        let doc = self.docs.get(&n);
        Hit {
            n,
            title: rec.map(|r| r.title.clone()).unwrap_or_default(),
            score,
            stub_only: doc.map(|d| d.surface == Surface::Stub).unwrap_or(false),
            in_set: in_set.contains(&n),
            superseded_by: rec.map(|r| r.obsoleted_by.clone()).unwrap_or_default(),
            whys,
            passages: doc.map(|d| self.passages(d, terms, 2)).unwrap_or_default(),
        }
    }

    /// §8.1 keyword search over the set on disk plus the stubs.
    pub fn keyword(
        &self,
        index: &IndexPayload,
        query: &str,
        in_set: &HashSet<u32>,
        limit: usize,
    ) -> Vec<Hit> {
        let stops = stopwords();
        let terms: Vec<String> = tokenize(query)
            .into_iter()
            .filter(|t| !stops.contains(t.as_str()))
            .collect();
        if terms.is_empty() {
            return Vec::new();
        }
        let scores = self.keyword_scores(&terms);
        let mut ranked: Vec<(u32, f64, Vec<String>)> = scores
            .into_iter()
            .map(|(n, (s, m))| (n, s, m))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked
            .into_iter()
            .take(limit)
            .map(|(n, s, matched)| {
                let why = format!("matched terms: {}", matched.join(", "));
                self.make_hit(index, n, s, vec![why], &terms, in_set)
            })
            .collect()
    }

    /// §8.2 intent search: keyword evidence + reference graph +
    /// implementation graph. Returns (hits, thin_signal_note).
    pub fn intent(
        &self,
        index: &IndexPayload,
        intent: &str,
        in_set: &HashSet<u32>,
        limit: usize,
    ) -> (Vec<Hit>, Option<String>) {
        let stops = stopwords();
        let terms: Vec<String> = tokenize(intent)
            .into_iter()
            .filter(|t| !stops.contains(t.as_str()))
            .collect();
        let kw = self.keyword_scores(&terms);
        if kw.is_empty() {
            return (
                Vec::new(),
                Some("thin signal: no keyword evidence matched any text or stub".into()),
            );
        }
        // The matched set M: strongest keyword candidates.
        let mut m_sorted: Vec<(u32, f64)> =
            kw.iter().map(|(n, (s, _))| (*n, *s)).collect();
        m_sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let m: HashSet<u32> = m_sorted.iter().take(25).map(|(n, _)| *n).collect();

        // Candidates: M plus their graph neighbourhood.
        let mut cand: HashSet<u32> = m.clone();
        for n in &m {
            if let Some(edges) = index.references.get(&n.to_string()) {
                cand.extend(edges.normative.iter());
                cand.extend(edges.informative.iter());
                cand.extend(edges.unspecified.iter());
            }
            if let Some(citers) = index.cited_by.get(&n.to_string()) {
                cand.extend(citers.iter().map(|c| c.from));
            }
        }

        let mut hits: Vec<Hit> = Vec::new();
        for n in cand {
            let rec = match index.record(n) {
                Some(r) if r.fetch_status == "ok" => r,
                _ => continue,
            };
            let (base, matched) = kw.get(&n).cloned().unwrap_or((0.0, Vec::new()));
            let mut score = base;
            let mut whys: Vec<String> = Vec::new();
            if !matched.is_empty() {
                whys.push(format!("keyword evidence: matched {}", matched.join(", ")));
            }
            // Reference-graph signals, named with direction.
            let mut graph_boost = 0.0f64;
            if let Some(edges) = index.references.get(&n.to_string()) {
                for (kind, list, w) in [
                    ("normative", &edges.normative, 1.0),
                    ("informative", &edges.informative, 0.6),
                    ("unspecified", &edges.unspecified, 0.4),
                ] {
                    for tgt in list {
                        if m.contains(tgt) && *tgt != n {
                            graph_boost += 0.35 * w;
                            whys.push(format!("cites RFC {tgt} ({kind}) from the matched set"));
                        }
                    }
                }
            }
            if let Some(citers) = index.cited_by.get(&n.to_string()) {
                for c in citers {
                    if m.contains(&c.from) && c.from != n {
                        let w = match c.kind.as_str() {
                            "normative" => 1.0,
                            "informative" => 0.6,
                            _ => 0.4,
                        };
                        graph_boost += 0.35 * w;
                        whys.push(format!(
                            "cited by RFC {} ({}) from the matched set",
                            c.from, c.kind
                        ));
                    }
                }
            }
            score += graph_boost;
            if rec.impl_count > 0 {
                let b = 0.5 * (1.0 + rec.impl_count as f64).ln();
                score += b;
                whys.push(format!(
                    "implementation graph: {} canonical implementation link(s) ({})",
                    rec.impl_count,
                    rec.impl_names.join(", ")
                ));
            }
            if !rec.obsoleted_by.is_empty() {
                score *= 0.85;
                whys.push(format!(
                    "superseded by RFC {} — shown annotated, per §8.4",
                    rec.obsoleted_by
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if score <= 0.0 || whys.is_empty() {
                continue;
            }
            hits.push(self.make_hit(index, n, score, whys, &terms, in_set));
        }
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        hits.truncate(limit);
        let thin = if hits.len() < 3 || terms.len() < 2 {
            Some(format!(
                "thin signal: {} intent term(s), {} candidate(s) — stated plainly per §8.4",
                terms.len(),
                hits.len()
            ))
        } else {
            None
        };
        (hits, thin)
    }
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Render a hit list the way the CLI prints it.
pub fn render_hits(hits: &[Hit], ranking: bool) -> String {
    let mut out = String::new();
    if ranking {
        out.push_str("(a ranking of candidates with their evidence — not a verdict, §8.4)\n");
    }
    for (i, h) in hits.iter().enumerate() {
        out.push_str(&format!(
            "{}. RFC {} — {}  [score {:.2}]",
            i + 1,
            h.n,
            h.title,
            h.score
        ));
        if h.in_set {
            out.push_str("  [in your set]");
        }
        if h.stub_only {
            out.push_str("  [summary only — full text not in your set]");
        }
        if !h.superseded_by.is_empty() {
            out.push_str(&format!(
                "  [superseded by {}]",
                h.superseded_by
                    .iter()
                    .map(|x| format!("RFC {x}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        out.push('\n');
        for w in &h.whys {
            out.push_str(&format!("   why: {w}\n"));
        }
        for p in &h.passages {
            out.push_str(&format!("   …{p}…\n"));
        }
    }
    if hits.is_empty() {
        out.push_str("no candidates\n");
    }
    out
}
