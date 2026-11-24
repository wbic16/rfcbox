//! The one fetch path (§7.4). Every fetch the crate makes — §4 refresh,
//! §7.2 sync/add, §7.3 whole-set — goes through here. The limiter is a
//! property of this path, not a setting: sequential requests, human-scale
//! spacing, no parallelism, no retry storms. There is no flag, config
//! value, or environment setting that raises the rate; the interval is a
//! constant of the code. A 429 or a bot wall from the source ends the
//! run — it is never routed around.
//!
//! Test seam: `RFCBOX_FETCH_BASE_FOR_TESTS` may redirect the base URL so
//! the path can be exercised against a local fixture server. It changes
//! where requests go, never how fast: the limiter applies identically.

use std::time::{Duration, Instant};

use crate::{Error, Result};

pub const RFC_EDITOR_BASE: &str = "https://www.rfc-editor.org";

/// Human fetching speed: the spacing a person browsing the RFC Editor
/// would produce. A constant — not configurable, by design (§7.4).
pub const MIN_INTERVAL: Duration = Duration::from_millis(2000);

pub struct Fetcher {
    base: String,
    last: Option<Instant>,
    agent: ureq::Agent,
    /// The rate limit being observed, recorded in run records (§7.4).
    pub observed_interval_ms: u64,
    pub requests: u32,
}

impl Fetcher {
    pub fn new() -> Fetcher {
        let base = std::env::var("RFCBOX_FETCH_BASE_FOR_TESTS")
            .unwrap_or_else(|_| RFC_EDITOR_BASE.to_string());
        Fetcher {
            base,
            last: None,
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(120))
                .user_agent("rfcbox/0.1 (third-party corpus tool; human-speed fetcher)")
                .build(),
            observed_interval_ms: MIN_INTERVAL.as_millis() as u64,
            requests: 0,
        }
    }

    /// Test/fixture constructor: same limiter, explicit base.
    pub fn with_base(base: &str) -> Fetcher {
        let mut f = Fetcher::new();
        f.base = base.trim_end_matches('/').to_string();
        f
    }

    fn pace(&mut self) {
        if let Some(last) = self.last {
            let elapsed = last.elapsed();
            if elapsed < MIN_INTERVAL {
                std::thread::sleep(MIN_INTERVAL - elapsed);
            }
        }
        self.last = Some(Instant::now());
    }

    pub fn get(&mut self, path: &str) -> Result<Vec<u8>> {
        self.pace();
        self.requests += 1;
        let url = format!("{}/{}", self.base, path.trim_start_matches('/'));
        match self.agent.get(&url).call() {
            Ok(resp) => {
                use std::io::Read;
                let mut buf = Vec::new();
                resp.into_reader()
                    .read_to_end(&mut buf)
                    .map_err(|e| Error::Fetch(format!("{url}: read failed: {e}")))?;
                Ok(buf)
            }
            Err(ureq::Error::Status(429, _)) => Err(Error::Fetch(format!(
                "{url}: source returned 429 (rate limited) — run ended; a 429 is final (good-netizen rule, §7.4)"
            ))),
            Err(ureq::Error::Status(code, _)) => {
                Err(Error::Fetch(format!("{url}: HTTP {code}")))
            }
            Err(e) => Err(Error::Fetch(format!("{url}: {e}"))),
        }
    }

    pub fn fetch_rfc_text(&mut self, n: u32) -> Result<Vec<u8>> {
        self.get(&format!("rfc/rfc{n}.txt"))
    }

    pub fn fetch_index(&mut self) -> Result<Vec<u8>> {
        self.get("rfc-index.xml")
    }
}

/// Parse (number, obsoleted_by) pairs for issued RFCs out of
/// rfc-index.xml — enough of the index of record for §4 discovery and
/// §7.2 sync's supersession checks. Deliberately small: it reads the
/// `<rfc-entry>` blocks' `<doc-id>` and `<obsoleted-by>` children.
pub fn parse_index_supersessions(xml: &[u8]) -> Vec<(u32, Vec<u32>)> {
    let s = String::from_utf8_lossy(xml);
    let mut out = Vec::new();
    for block in s.split("<rfc-entry>").skip(1) {
        let Some(docid) = tag_contents(block, "doc-id") else { continue };
        let Some(num) = docid.trim().strip_prefix("RFC").and_then(|x| x.parse::<u32>().ok())
        else {
            continue;
        };
        let mut obs = Vec::new();
        if let Some(ob) = tag_contents(block, "obsoleted-by") {
            for part in ob.split(',') {
                if let Some(n) = part
                    .trim()
                    .strip_prefix("RFC")
                    .and_then(|x| x.parse::<u32>().ok())
                {
                    obs.push(n);
                }
            }
        }
        out.push((num, obs));
    }
    out
}

fn tag_contents<'a>(block: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&close)? + start;
    Some(&block[start..end])
}
