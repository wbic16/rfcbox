//! rfcbox CLI. Commands (spec section in parentheses):
//!   init (§7.2) · add (§7.2/§5) · remove (§7.2) · sync (§7.2/§4)
//!   audit (§2 + §9.4) · verify (§9.4, same audit) · scan (§7.2)
//!   search (§8) · subset (§5) · refresh (§4)
//!   baseline (§9.1) · diff (§9.2) · freeze/thaw (§9.3) · export (§9.5)
//!   core · show — inspect the shipped baseline and its receipts
//!
//! Nothing here is an IETF or RFC Editor product; the corpus is a
//! third-party structure over IETF material (§2).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::process::ExitCode;

use rfcbox::audit::audit_project;
use rfcbox::fetch::{parse_index_supersessions, Fetcher};
use rfcbox::manifest::{IndexPayload, Manifest};
use rfcbox::payload::{load_core, load_index};
use rfcbox::project::{project_root_arg, strip_project_arg, Baseline, Project, SetEntry, SetFile};
use rfcbox::search::{render_hits, Engine};
use rfcbox::{sha256_hex, Error, Result};

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match run(&argv) {
        Ok(()) => ExitCode::from(0),
        Err(Error::HashRefusal(m)) => {
            eprintln!("HASH REFUSAL: {m}");
            ExitCode::from(65)
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

struct Ctx {
    manifest: Manifest,
    index: IndexPayload,
    core: Vec<(u32, Vec<u8>)>,
}

fn ctx() -> Result<Ctx> {
    let manifest = Manifest::load_shipped()?;
    let index = load_index(&manifest)?;
    let core = load_core(&manifest, &index)?;
    Ok(Ctx { manifest, index, core })
}

fn run(argv: &[String]) -> Result<()> {
    if argv.is_empty() {
        print_help();
        return Ok(());
    }
    let root = project_root_arg(argv);
    let args = strip_project_arg(argv);
    let cmd = args[0].as_str();
    let rest = &args[1..];
    match cmd {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "core" => cmd_core(),
        "show" => cmd_show(rest, &root),
        "init" => cmd_init(rest, &root),
        "add" => cmd_add(rest, &root),
        "remove" => cmd_remove(rest, &root),
        "sync" => cmd_sync(rest, &root),
        "audit" | "verify" => cmd_audit(&root),
        "scan" => cmd_scan(rest, &root),
        "search" => cmd_search(rest, &root),
        "subset" => cmd_subset(rest),
        "refresh" => cmd_refresh(rest),
        "baseline" => cmd_baseline(rest, &root),
        "diff" => cmd_diff(rest, &root),
        "freeze" => cmd_freeze(&root, true),
        "thaw" => cmd_freeze(&root, false),
        "export" => cmd_export(&root),
        other => Err(Error::Usage(format!("unknown command {other} (try: rfcbox help)"))),
    }
}

fn print_help() {
    println!(
        "rfcbox — the RFC corpus as a Rust crate (code MIT; RFC texts under IETF terms, verbatim, hash-verified)\n\
         \n\
         usage: rfcbox [--project DIR] <command> [args]\n\
         \n\
         set commands (§7.2):\n\
         \x20 init                     create rfcs.json from the baseline core and materialize it\n\
         \x20 add N... [--closure --depth D --kinds normative,informative,unspecified --direction out|in|both] [--corpus PHEXT]\n\
         \x20 remove N...              drop RFCs from the declared set (local only)\n\
         \x20 sync [--corpus PHEXT]    bring the declared set current (supersessions annotated, never silently swapped)\n\
         \x20 scan [--reconcile]       document which RFCs the project uses (strong/weak evidence, three-set diff)\n\
         search (§8):\n\
         \x20 search TERMS...          keyword search over the set on disk + discovery stubs\n\
         \x20 search --intent \"...\"    intent search: ranked candidates, every one with its why\n\
         audit (§2/§9.4):\n\
         \x20 audit | verify           materialized? hash-verified? status as of today? + legend receipts. fetches nothing\n\
         corpus (§4/§5):\n\
         \x20 subset N... --depth D --out DIR [--kinds ..] [--direction ..]   self-contained slice with provenance manifest\n\
         \x20 refresh --corpus PHEXT   discover + fetch new RFCs into a candidate snapshot (never in place)\n\
         waterfall (§9):\n\
         \x20 baseline NAME · diff A [B] · freeze · thaw · export\n\
         shipped baseline:\n\
         \x20 core                     list the baseline core with its audit receipts\n\
         \x20 show N                   print RFC N's verbatim text (core or your set), hash-verified first"
    );
}

// ---------- shipped baseline ----------

fn cmd_core() -> Result<()> {
    let c = ctx()?;
    println!(
        "rfcbox {} — audited snapshot {} ({} records, {} texts)",
        c.manifest.crate_version,
        &c.manifest.snapshot_sha256[..16],
        c.manifest.snapshot_records,
        c.manifest.snapshot_texts
    );
    println!(
        "legend audit: {}/{} issued texts classified",
        c.manifest.legend_audit_classified, c.manifest.legend_audit_issued
    );
    println!(
        "discovery stubs: {} stubs, {} bytes (§8.5)",
        c.manifest.stub_count, c.manifest.stub_total_bytes
    );
    println!("\nbaseline core (§7.1) — verbatim, hash-verified post-inflation:");
    for (n, bytes) in &c.core {
        let rec = c.index.record(*n).unwrap();
        println!(
            "  RFC {:<5} {:<58} {:>7} B  legend: {}",
            n,
            rec.title.chars().take(58).collect::<String>(),
            bytes.len(),
            rec.legend
        );
    }
    Ok(())
}

fn cmd_show(rest: &[String], root: &PathBuf) -> Result<()> {
    let n: u32 = rest
        .first()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::Usage("show needs an RFC number".into()))?;
    let c = ctx()?;
    if let Some((_, bytes)) = c.core.iter().find(|(cn, _)| *cn == n) {
        print!("{}", String::from_utf8_lossy(bytes));
        return Ok(());
    }
    let project = Project::new(root.clone());
    if let Some(set) = project.load_set()? {
        if set.contains(n) {
            let bytes = project
                .materialized(n)
                .ok_or_else(|| Error::Other(format!("RFC {n} declared but not materialized")))?;
            rfcbox::verify_hash(&format!("RFC {n}"), &bytes, &set.get(n).unwrap().sha256)?;
            print!("{}", String::from_utf8_lossy(&bytes));
            return Ok(());
        }
    }
    Err(Error::Other(format!(
        "RFC {n} is not in the baseline core or your set — only its discovery stub ships (add it first)"
    )))
}

// ---------- set commands ----------

fn baseline_entries(c: &Ctx) -> Vec<SetEntry> {
    c.manifest
        .baseline
        .iter()
        .map(|n| {
            let rec = c.index.record(*n).unwrap();
            SetEntry {
                number: *n,
                sha256: rec.sha256.clone().unwrap_or_default(),
                source: "baseline-core".into(),
                superseded_by: rec.obsoleted_by.first().copied(),
                origin: "baseline".into(),
                origin_detail: format!(
                    "rfcbox baseline core, crate {} (§7.1)",
                    c.manifest.crate_version
                ),
            }
        })
        .collect()
}

fn cmd_init(_rest: &[String], root: &PathBuf) -> Result<()> {
    let project = Project::new(root.clone());
    if project.load_set()?.is_some() {
        return Err(Error::Usage(format!(
            "rfcs.json already exists in {} — refusing to overwrite a declaration",
            root.display()
        )));
    }
    let c = ctx()?;
    let mut set = SetFile::new();
    set.rfcs = baseline_entries(&c);
    for (n, bytes) in &c.core {
        let rec = c.index.record(*n).unwrap();
        project.materialize(*n, bytes, rec.sha256.as_deref().unwrap())?;
    }
    project.save_set(&set)?;
    project.rebuild_local_index(&set)?;
    println!(
        "initialized {} from the baseline core: {} RFCs declared and materialized (hash-verified)",
        project.set_path().display(),
        set.rfcs.len()
    );
    Ok(())
}

/// Resolve a text for materialization: baseline core, then a local
/// corpus (--corpus / RFCBOX_CORPUS), then the one limited fetch path.
/// Every route's bytes are verified against the index record (§2).
fn resolve_text(
    c: &Ctx,
    n: u32,
    corpus: &mut Option<(String, HashMap<u32, Vec<u8>>)>,
    fetcher: &mut Option<Fetcher>,
) -> Result<(Vec<u8>, String)> {
    let rec = c
        .index
        .record(n)
        .ok_or_else(|| Error::Usage(format!("RFC {n} is not in the corpus catalogue")))?;
    if rec.fetch_status != "ok" {
        return Err(Error::Other(format!(
            "RFC {n} has no text in the audited snapshot (fetch_status: {})",
            rec.fetch_status
        )));
    }
    let recorded = rec.sha256.clone().unwrap();
    if let Some((_, bytes)) = c.core.iter().find(|(cn, _)| *cn == n) {
        return Ok((bytes.clone(), "baseline-core".into()));
    }
    if let Some((path, texts)) = corpus {
        let bytes = texts
            .get(&n)
            .ok_or_else(|| Error::Other(format!("RFC {n} not found in corpus {path}")))?;
        rfcbox::verify_hash(&format!("RFC {n} from corpus"), bytes, &recorded)?;
        return Ok((bytes.clone(), format!("corpus:{path}")));
    }
    let f = fetcher.get_or_insert_with(Fetcher::new);
    let bytes = f.fetch_rfc_text(n)?;
    rfcbox::verify_hash(&format!("RFC {n} as fetched"), &bytes, &recorded)?;
    Ok((bytes, format!("{}/rfc/rfc{n}.txt", rfcbox::fetch::RFC_EDITOR_BASE)))
}

fn load_corpus_arg(rest: &[String]) -> Result<Option<(String, HashMap<u32, Vec<u8>>)>> {
    let mut path: Option<String> = None;
    for i in 0..rest.len() {
        if rest[i] == "--corpus" && i + 1 < rest.len() {
            path = Some(rest[i + 1].clone());
        }
    }
    if path.is_none() {
        path = std::env::var("RFCBOX_CORPUS").ok();
    }
    let Some(path) = path else { return Ok(None) };
    let raw = std::fs::read_to_string(&path)?;
    let map = libphext::phext::explode(&raw);
    let mut texts = HashMap::new();
    for (coord, scroll) in &map {
        let s = coord.to_string();
        // RFC scrolls live at 1.1.1/1.2.1/1.1.N
        if let Some(tail) = s.strip_prefix("1.1.1/1.2.1/1.1.") {
            if let Ok(n) = tail.parse::<u32>() {
                if let Some(bytes) = rfcbox::text::scroll_text_bytes(scroll) {
                    texts.insert(n, bytes);
                }
            }
        }
    }
    Ok(Some((path, texts)))
}

fn opt_value(rest: &[String], name: &str) -> Option<String> {
    for i in 0..rest.len() {
        if rest[i] == name && i + 1 < rest.len() {
            return Some(rest[i + 1].clone());
        }
    }
    None
}

fn parse_kinds(s: &str) -> BTreeSet<String> {
    s.split(',').map(|x| x.trim().to_string()).collect()
}

/// §5 dependency closure over the shipped reference graph.
fn closure(
    index: &IndexPayload,
    seeds: &[u32],
    depth: u32,
    kinds: &BTreeSet<String>,
    direction: &str,
) -> BTreeSet<u32> {
    let mut seen: BTreeSet<u32> = seeds.iter().copied().collect();
    let mut frontier: VecDeque<(u32, u32)> = seeds.iter().map(|n| (*n, 0)).collect();
    while let Some((n, d)) = frontier.pop_front() {
        if d >= depth {
            continue;
        }
        let mut next: Vec<u32> = Vec::new();
        if direction == "out" || direction == "both" {
            if let Some(e) = index.references.get(&n.to_string()) {
                if kinds.contains("normative") {
                    next.extend(&e.normative);
                }
                if kinds.contains("informative") {
                    next.extend(&e.informative);
                }
                if kinds.contains("unspecified") {
                    next.extend(&e.unspecified);
                }
            }
        }
        if direction == "in" || direction == "both" {
            if let Some(citers) = index.cited_by.get(&n.to_string()) {
                for c in citers {
                    if kinds.contains(&c.kind) {
                        next.push(c.from);
                    }
                }
            }
        }
        for m in next {
            if seen.insert(m) {
                frontier.push_back((m, d + 1));
            }
        }
    }
    seen
}

fn cmd_add(rest: &[String], root: &PathBuf) -> Result<()> {
    let project = Project::new(root.clone());
    let state = project.load_state()?;
    if state.frozen {
        return Err(Error::Frozen(
            "add refused — the set is frozen (§9.3); thaw first".into(),
        ));
    }
    let c = ctx()?;
    let use_closure = rest.iter().any(|a| a == "--closure");
    let depth: u32 = opt_value(rest, "--depth")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let kinds = opt_value(rest, "--kinds")
        .map(|s| parse_kinds(&s))
        .unwrap_or_else(|| ["normative".to_string()].into_iter().collect());
    let direction = opt_value(rest, "--direction").unwrap_or_else(|| "out".into());
    let mut nums: Vec<u32> = Vec::new();
    let mut skip_next = false;
    for a in rest {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a.starts_with("--") {
            if ["--depth", "--kinds", "--direction", "--corpus"].contains(&a.as_str()) {
                skip_next = true;
            }
            continue;
        }
        nums.push(
            a.parse()
                .map_err(|_| Error::Usage(format!("not an RFC number: {a}")))?,
        );
    }
    if nums.is_empty() {
        return Err(Error::Usage("add needs at least one RFC number".into()));
    }
    // A fresh project's rfcs.json starts from the baseline core (§7.2).
    let mut set = match project.load_set()? {
        Some(s) => s,
        None => {
            let mut s = SetFile::new();
            s.rfcs = baseline_entries(&c);
            for (n, bytes) in &c.core {
                let rec = c.index.record(*n).unwrap();
                project.materialize(*n, bytes, rec.sha256.as_deref().unwrap())?;
            }
            println!("(fresh project: rfcs.json started from the baseline core)");
            s
        }
    };
    let targets: BTreeSet<u32> = if use_closure {
        closure(&c.index, &nums, depth, &kinds, &direction)
    } else {
        nums.iter().copied().collect()
    };
    let mut corpus = load_corpus_arg(rest)?;
    let mut fetcher: Option<Fetcher> = None;
    let mut added = Vec::new();
    for n in &targets {
        if set.contains(*n) {
            println!("RFC {n}: already declared — skipped");
            continue;
        }
        let (bytes, source) = resolve_text(&c, *n, &mut corpus, &mut fetcher)?;
        let rec = c.index.record(*n).unwrap();
        project.materialize(*n, &bytes, rec.sha256.as_deref().unwrap())?;
        let detail = if use_closure {
            format!(
                "subset closure of seeds {:?} depth {} kinds {:?} direction {} against snapshot {} (§5)",
                nums, depth, kinds, direction, &c.manifest.snapshot_sha256[..16]
            )
        } else {
            "explicit add".into()
        };
        set.rfcs.push(SetEntry {
            number: *n,
            sha256: rec.sha256.clone().unwrap(),
            source,
            superseded_by: rec.obsoleted_by.first().copied(),
            origin: if use_closure { "closure-add".into() } else { "add".into() },
            origin_detail: detail,
        });
        added.push(*n);
    }
    set.rfcs.sort_by_key(|e| e.number);
    project.save_set(&set)?;
    project.rebuild_local_index(&set)?;
    println!(
        "added {} RFC(s): {:?} — declared set now {} RFCs",
        added.len(),
        added,
        set.rfcs.len()
    );
    if let Some(f) = &fetcher {
        println!(
            "fetch record: {} request(s) at human speed ({} ms spacing, sequential — §7.4)",
            f.requests, f.observed_interval_ms
        );
    }
    Ok(())
}

fn cmd_remove(rest: &[String], root: &PathBuf) -> Result<()> {
    let project = Project::new(root.clone());
    let state = project.load_state()?;
    if state.frozen {
        return Err(Error::Frozen(
            "remove refused — the set is frozen (§9.3); thaw first".into(),
        ));
    }
    let mut set = project.require_set()?;
    let mut removed = Vec::new();
    for a in rest {
        let n: u32 = a
            .parse()
            .map_err(|_| Error::Usage(format!("not an RFC number: {a}")))?;
        if set.contains(n) {
            set.rfcs.retain(|e| e.number != n);
            project.dematerialize(n)?;
            removed.push(n);
        } else {
            println!("RFC {n}: not in the set — nothing to remove");
        }
    }
    project.save_set(&set)?;
    project.rebuild_local_index(&set)?;
    println!(
        "removed {} RFC(s): {:?} — declared set now {} RFCs (local only; the declaration's history is the record)",
        removed.len(),
        removed,
        set.rfcs.len()
    );
    Ok(())
}

fn cmd_sync(rest: &[String], root: &PathBuf) -> Result<()> {
    let project = Project::new(root.clone());
    let mut state = project.load_state()?;
    let mut set = project.require_set()?;
    let c = ctx()?;
    let mut fetcher = Fetcher::new();
    let xml = fetcher.fetch_index()?;
    let idx_sup: HashMap<u32, Vec<u32>> = parse_index_supersessions(&xml).into_iter().collect();
    let mut notices: Vec<String> = Vec::new();
    let mut corpus = load_corpus_arg(rest)?;
    let mut fetch_wrap: Option<Fetcher> = None;
    for e in set.rfcs.iter_mut() {
        if let Some(obs) = idx_sup.get(&e.number) {
            if !obs.is_empty() && e.superseded_by.is_none() {
                let msg = format!(
                    "RFC {} has been superseded by {} (rfc-index.xml) — annotated, not swapped",
                    e.number,
                    obs.iter()
                        .map(|x| format!("RFC {x}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                if state.frozen {
                    notices.push(msg);
                } else {
                    e.superseded_by = Some(obs[0]);
                    notices.push(format!("applied: {msg}"));
                }
            }
        }
        if project.materialized(e.number).is_none() {
            if state.frozen {
                notices.push(format!(
                    "RFC {} declared but not materialized (held by freeze)",
                    e.number
                ));
            } else {
                let (bytes, source) =
                    resolve_text(&c, e.number, &mut corpus, &mut fetch_wrap)?;
                project.materialize(e.number, &bytes, &e.sha256)?;
                e.source = source;
                notices.push(format!("RFC {} materialized", e.number));
            }
        }
    }
    if state.frozen {
        state.pending_notices.extend(notices.iter().cloned());
        project.save_state(&state)?;
        println!(
            "sync under freeze: {} notice(s) recorded, nothing applied (§9.3)",
            notices.len()
        );
    } else {
        project.save_set(&set)?;
        project.rebuild_local_index(&set)?;
        // Frozen-interval notices are carried into this sync (§9.3).
        if !state.pending_notices.is_empty() {
            println!(
                "carried from the frozen interval: {} earlier notice(s)",
                state.pending_notices.len()
            );
            for n in state.pending_notices.drain(..) {
                println!("  notice: {n}");
            }
            project.save_state(&state)?;
        }
        println!("sync complete: {} notice(s)", notices.len());
    }
    for n in &notices {
        println!("  {n}");
    }
    let total_requests = fetcher.requests + fetch_wrap.as_ref().map(|f| f.requests).unwrap_or(0);
    println!(
        "fetch record: {} request(s) at human speed ({} ms spacing, sequential — §7.4)",
        total_requests, fetcher.observed_interval_ms
    );
    Ok(())
}

fn cmd_audit(root: &PathBuf) -> Result<()> {
    let project = Project::new(root.clone());
    let set = project.require_set()?;
    let c = ctx()?;
    let rep = audit_project(&project, &set, &c.index, &rfcbox::project::today_utc())?;
    print!("{}", rep.render());
    if !rep.hash_failures.is_empty() {
        return Err(Error::HashRefusal(format!(
            "{} hash failure(s) — see audit report above",
            rep.hash_failures.len()
        )));
    }
    Ok(())
}

// ---------- scan (§7.2) ----------

const WEAK_SIGNALS: &[(&str, u32, &str)] = &[
    ("websocket", 6455, "WebSocket protocol fingerprint"),
    ("oauth", 6749, "OAuth 2.0 fingerprint"),
    ("jsonwebtoken", 7519, "JWT fingerprint"),
    ("jwt", 7519, "JWT fingerprint"),
    ("http/2", 9113, "HTTP/2 fingerprint"),
    ("http/3", 9114, "HTTP/3 fingerprint"),
    ("quic", 9000, "QUIC fingerprint"),
    ("cookie", 6265, "HTTP cookies fingerprint"),
    ("bearer", 6750, "Bearer token fingerprint"),
];

fn cmd_scan(rest: &[String], root: &PathBuf) -> Result<()> {
    let project = Project::new(root.clone());
    let set = project.load_set()?.unwrap_or_default();
    let reconcile = rest.iter().any(|a| a == "--reconcile");
    let mut strong: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut weak: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    for f in files {
        let Ok(bytes) = std::fs::read(&f) else { continue };
        if bytes.len() > 2_000_000 {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        let rel = f
            .strip_prefix(root)
            .unwrap_or(&f)
            .to_string_lossy()
            .into_owned();
        for (lineno, line) in text.lines().enumerate() {
            let lower = line.to_lowercase();
            let mut rest_line = lower.as_str();
            while let Some(pos) = find_rfc_marker(rest_line) {
                let after = &rest_line[pos + 3..];
                let num_str: String = after
                    .trim_start()
                    .chars()
                    .take_while(|ch| ch.is_ascii_digit())
                    .collect();
                if let Ok(n) = num_str.parse::<u32>() {
                    if (1..=10100).contains(&n) {
                        strong
                            .entry(n)
                            .or_default()
                            .push(format!("{rel}:{}", lineno + 1));
                    }
                }
                rest_line = &rest_line[pos + 3..];
            }
            for (kw, n, label) in WEAK_SIGNALS {
                if lower.contains(kw) {
                    weak.entry(*n)
                        .or_default()
                        .push(format!("{rel}:{} ({label})", lineno + 1));
                }
            }
        }
    }
    // Weak evidence for an RFC with strong evidence is not reported as weak.
    for n in strong.keys() {
        weak.remove(n);
    }
    let declared: HashSet<u32> = set.rfcs.iter().map(|e| e.number).collect();
    let used: HashSet<u32> = strong.keys().copied().collect();
    let confirmed: Vec<u32> = declared.intersection(&used).copied().collect();
    let gaps: Vec<u32> = used.difference(&declared).copied().collect();
    let drift: Vec<u32> = declared.difference(&used).copied().collect();
    let today = rfcbox::project::today_utc();
    let mut report = format!("rfcbox scan — {today} — project {}\n", root.display());
    report.push_str(&format!("\nconfirmed — in use and declared ({}):\n", confirmed.len()));
    for n in &confirmed {
        report.push_str(&format!(
            "  RFC {n} — evidence: {}\n",
            strong[n].iter().take(3).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    report.push_str(&format!("\ngaps — in use but not declared ({}):\n", gaps.len()));
    for n in &gaps {
        report.push_str(&format!(
            "  RFC {n} — evidence: {}\n",
            strong[n].iter().take(3).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    report.push_str(&format!(
        "\npossible drift — declared, no evidence found ({}):\n",
        drift.len()
    ));
    for n in &drift {
        report.push_str(&format!(
            "  RFC {n} — no evidence found (absence of evidence, stated as such)\n"
        ));
    }
    report.push_str(&format!(
        "\nweak signals — labelled weak, never promoted ({}):\n",
        weak.len()
    ));
    for (n, ev) in &weak {
        report.push_str(&format!(
            "  RFC {n} — weak: {}\n",
            ev.iter().take(2).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    print!("{report}");
    std::fs::create_dir_all(project.reports_dir())?;
    let report_path = project.reports_dir().join(format!("scan-{today}.md"));
    std::fs::write(&report_path, &report)?;
    println!("\nreport kept at {}", report_path.display());
    if reconcile {
        let c = ctx()?;
        let mut set2 = set;
        let mut added = Vec::new();
        for n in &gaps {
            let Some(rec) = c.index.record(*n) else { continue };
            let Some(sha) = rec.sha256.clone() else { continue };
            set2.rfcs.push(SetEntry {
                number: *n,
                sha256: sha,
                source: "declared-by-scan (text not fetched — run sync/add)".into(),
                superseded_by: rec.obsoleted_by.first().copied(),
                origin: "found-by-scan".into(),
                origin_detail: format!(
                    "evidence: {}",
                    strong[n].iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                ),
            });
            added.push(*n);
        }
        set2.rfcs.sort_by_key(|e| e.number);
        project.save_set(&set2)?;
        println!(
            "reconciled on your confirmation: {} strong-evidence gap(s) entered into rfcs.json as found-by-scan: {:?}",
            added.len(),
            added
        );
    }
    Ok(())
}

fn find_rfc_marker(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 0;
    while i + 3 < b.len() {
        if (b[i] == b'r') && (b[i + 1] == b'f') && (b[i + 2] == b'c') {
            let before_ok = i == 0 || !(b[i - 1] as char).is_ascii_alphanumeric();
            let after = &s[i + 3..];
            let after_ok = after
                .trim_start_matches([' ', '-', '_'])
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit());
            if before_ok && after_ok {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn collect_files(root: &PathBuf, dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if [".git", ".rfcbox", "target", "node_modules"].contains(&name.as_str()) {
                continue;
            }
            collect_files(root, &p, out);
        } else {
            out.push(p);
        }
    }
}

// ---------- search (§8) ----------

fn cmd_search(rest: &[String], root: &PathBuf) -> Result<()> {
    let c = ctx()?;
    let project = Project::new(root.clone());
    let mut full: Vec<(u32, Vec<u8>)> = c.core.clone();
    let mut in_set: HashSet<u32> = HashSet::new();
    if let Some(set) = project.load_set()? {
        in_set = set.rfcs.iter().map(|e| e.number).collect();
        for (n, bytes) in project.materialized_texts(&set)? {
            if !full.iter().any(|(cn, _)| *cn == n) {
                full.push((n, bytes));
            }
        }
    }
    let engine = Engine::build(&c.index, &full, &in_set);
    if rest.first().map(|s| s.as_str()) == Some("--intent") {
        let intent = rest[1..].join(" ");
        if intent.is_empty() {
            return Err(Error::Usage("search --intent needs an intent statement".into()));
        }
        let (hits, thin) = engine.intent(&c.index, &intent, &in_set, 10);
        println!("intent search: \"{intent}\"");
        print!("{}", render_hits(&hits, true));
        if let Some(t) = thin {
            println!("{t}");
        }
    } else {
        let query = rest.join(" ");
        if query.is_empty() {
            return Err(Error::Usage("search needs terms, or --intent \"...\"".into()));
        }
        let hits = engine.keyword(&c.index, &query, &in_set, 10);
        println!("keyword search: \"{query}\" (scope: set on disk + discovery stubs; a query never fetches)");
        print!("{}", render_hits(&hits, false));
    }
    Ok(())
}

// ---------- subset (§5) ----------

fn cmd_subset(rest: &[String]) -> Result<()> {
    let out_dir = opt_value(rest, "--out")
        .ok_or_else(|| Error::Usage("subset needs --out DIR".into()))?;
    let depth: u32 = opt_value(rest, "--depth")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let kinds = opt_value(rest, "--kinds")
        .map(|s| parse_kinds(&s))
        .unwrap_or_else(|| ["normative".to_string()].into_iter().collect());
    let direction = opt_value(rest, "--direction").unwrap_or_else(|| "out".into());
    let mut seeds: Vec<u32> = Vec::new();
    let mut skip_next = false;
    for a in rest {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a.starts_with("--") {
            skip_next = true;
            continue;
        }
        seeds.push(
            a.parse()
                .map_err(|_| Error::Usage(format!("not an RFC number: {a}")))?,
        );
    }
    if seeds.is_empty() {
        return Err(Error::Usage("subset needs seed RFC numbers".into()));
    }
    let c = ctx()?;
    let members = closure(&c.index, &seeds, depth, &kinds, &direction);
    let mut corpus = load_corpus_arg(rest)?;
    let mut fetcher: Option<Fetcher> = None;
    let dir = PathBuf::from(&out_dir);
    std::fs::create_dir_all(dir.join("texts"))?;
    let mut entries = Vec::new();
    let mut edges_out = Vec::new();
    for n in &members {
        let (bytes, _src) = resolve_text(&c, *n, &mut corpus, &mut fetcher)?;
        std::fs::write(dir.join("texts").join(format!("rfc{n}.txt")), &bytes)?;
        let rec = c.index.record(*n).unwrap();
        entries.push(serde_json::json!({
            "number": n,
            "title": rec.title,
            "sha256": rec.sha256,
            "superseded_by": rec.obsoleted_by,
        }));
        if let Some(e) = c.index.references.get(&n.to_string()) {
            for (kind, list) in [
                ("normative", &e.normative),
                ("informative", &e.informative),
                ("unspecified", &e.unspecified),
            ] {
                for t in list {
                    if members.contains(t) {
                        edges_out.push(serde_json::json!({"from": n, "to": t, "kind": kind}));
                    }
                }
            }
        }
    }
    let slice_canon = entries
        .iter()
        .map(|e| format!("{}:{}", e["number"], e["sha256"].as_str().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(";");
    let slice_hash = sha256_hex(slice_canon.as_bytes());
    let superseded: Vec<u32> = members
        .iter()
        .filter(|n| {
            c.index
                .record(**n)
                .map(|r| !r.obsoleted_by.is_empty())
                .unwrap_or(false)
        })
        .copied()
        .collect();
    let slice_manifest = serde_json::json!({
        "slice": "rfcbox subset (§5)",
        "seeds": seeds,
        "depth": depth,
        "kinds": kinds.iter().collect::<Vec<_>>(),
        "direction": direction,
        "snapshot_sha256": c.manifest.snapshot_sha256,
        "slice_sha256": slice_hash,
        "members": entries,
        "edges_among_members": edges_out,
        "superseded_members_annotated": superseded,
        "tlp_notice": "RFC texts are IETF/author material under IETF Trust (now IETF IPMC) Legal Provisions terms, carried verbatim; this slice is a third-party structure over IETF material, not an IETF product.",
    });
    std::fs::write(
        dir.join("slice-manifest.json"),
        serde_json::to_string_pretty(&slice_manifest)? + "\n",
    )?;
    println!(
        "subset: {} RFC(s) from seeds {:?} (depth {}, kinds {:?}, direction {}) -> {}",
        members.len(),
        seeds,
        depth,
        kinds,
        direction,
        dir.display()
    );
    println!("slice sha256: {slice_hash} (same request + same snapshot = same slice, byte-for-byte)");
    if !superseded.is_empty() {
        println!(
            "superseded members included, annotated with successors: {:?}",
            superseded
        );
    }
    Ok(())
}

// ---------- refresh (§4) ----------

fn cmd_refresh(rest: &[String]) -> Result<()> {
    let corpus_path = opt_value(rest, "--corpus")
        .or_else(|| std::env::var("RFCBOX_CORPUS").ok())
        .ok_or_else(|| Error::Usage("refresh needs --corpus IETF.PHEXT (the local snapshot to extend)".into()))?;
    let c = ctx()?;
    let mut fetcher = Fetcher::new();
    let xml = fetcher.fetch_index()?;
    let idx = parse_index_supersessions(&xml);
    let max_known = c
        .index
        .records
        .iter()
        .map(|r| r.n)
        .max()
        .unwrap_or(0);
    let new_numbers: Vec<u32> = idx
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| *n > max_known)
        .collect();
    let changed: Vec<(u32, Vec<u32>)> = idx
        .iter()
        .filter(|(n, obs)| {
            !obs.is_empty()
                && c.index
                    .record(*n)
                    .map(|r| r.obsoleted_by.is_empty())
                    .unwrap_or(false)
        })
        .cloned()
        .collect();
    println!(
        "discovery: {} RFC number(s) published after the snapshot (max known {}), {} supersession change(s) to held records",
        new_numbers.len(),
        max_known,
        changed.len()
    );
    for (n, obs) in &changed {
        println!(
            "  changed: RFC {n} now obsoleted by {:?}",
            obs
        );
    }
    if new_numbers.is_empty() {
        println!("no new RFCs to fetch; candidate snapshot not produced");
        return Ok(());
    }
    // Fetch verbatim, hash at fetch, append as scrolls to a CANDIDATE
    // copy — the audited artifact is never edited in place (§4).
    let raw = std::fs::read_to_string(&corpus_path)?;
    let mut candidate = raw.clone();
    let mut appended = Vec::new();
    for n in &new_numbers {
        let bytes = fetcher.fetch_rfc_text(*n)?;
        let sha = sha256_hex(&bytes);
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let coord = format!("1.1.1/1.2.1/1.1.{n}");
        let scroll = format!(
            "# RFC {n} -- (title per rfc-index.xml at refresh)\n\nRFC: {n}\nRecord (MOAT): True / VERIFIED -- record only.\nText SHA-256 (as fetched): {sha}\nText bytes (as fetched): {}\nCoordinate: {coord}\n\n--- RFC TEXT AS SERVED BY THE RFC EDITOR ---\n\n{text}",
            bytes.len()
        );
        candidate = libphext::phext::insert(
            candidate,
            libphext::phext::to_coordinate(&coord),
            &scroll,
        );
        appended.push(*n);
    }
    let out_path = format!("{corpus_path}.candidate");
    std::fs::write(&out_path, &candidate)?;
    println!(
        "candidate snapshot written to {out_path}: {} new scroll(s) appended {:?}; it replaces the served corpus only after re-audit (§4)",
        appended.len(),
        appended
    );
    println!(
        "fetch record: {} request(s) at human speed ({} ms spacing, sequential — §7.4)",
        fetcher.requests, fetcher.observed_interval_ms
    );
    Ok(())
}

// ---------- waterfall (§9) ----------

fn cmd_baseline(rest: &[String], root: &PathBuf) -> Result<()> {
    let name = rest
        .first()
        .ok_or_else(|| Error::Usage("baseline needs a name".into()))?;
    let project = Project::new(root.clone());
    let set = project.require_set()?;
    let c = ctx()?;
    if project.load_baseline(name).is_ok() {
        return Err(Error::Usage(format!(
            "baseline {name} already exists — baselines are immutable (§9.1); choose a new name"
        )));
    }
    let b = Baseline {
        name: name.clone(),
        date: rfcbox::project::today_utc(),
        set_hash: set.set_hash(),
        snapshot_sha256: c.manifest.snapshot_sha256.clone(),
        set,
    };
    project.save_baseline(&b)?;
    let mut state = project.load_state()?;
    state
        .history
        .push(format!("{} baseline {} taken (set hash {})", b.date, b.name, &b.set_hash[..16]));
    project.save_state(&state)?;
    println!(
        "baseline {} frozen: set hash {}, against snapshot {}",
        b.name,
        &b.set_hash[..16],
        &b.snapshot_sha256[..16]
    );
    Ok(())
}

fn cmd_diff(rest: &[String], root: &PathBuf) -> Result<()> {
    let a_name = rest
        .first()
        .ok_or_else(|| Error::Usage("diff needs a baseline name".into()))?;
    let project = Project::new(root.clone());
    let a = project.load_baseline(a_name)?;
    let (b_set, b_label) = match rest.get(1) {
        Some(name) => {
            let b = project.load_baseline(name)?;
            (b.set, format!("baseline {name}"))
        }
        None => (project.require_set()?, "current set".to_string()),
    };
    let a_nums: HashMap<u32, &SetEntry> = a.set.rfcs.iter().map(|e| (e.number, e)).collect();
    let b_nums: HashMap<u32, &SetEntry> = b_set.rfcs.iter().map(|e| (e.number, e)).collect();
    println!("diff: baseline {a_name} ({}) -> {b_label}", a.date);
    let mut any = false;
    for n in b_nums.keys() {
        if !a_nums.contains_key(n) {
            println!("  added:   RFC {n}");
            any = true;
        }
    }
    for n in a_nums.keys() {
        if !b_nums.contains_key(n) {
            println!("  removed: RFC {n}");
            any = true;
        }
    }
    for (n, ea) in &a_nums {
        if let Some(eb) = b_nums.get(n) {
            if ea.superseded_by != eb.superseded_by {
                println!(
                    "  status-changed: RFC {n} supersession {:?} -> {:?} (successor named, never silently swapped)",
                    ea.superseded_by, eb.superseded_by
                );
                any = true;
            }
        }
    }
    if !any {
        println!("  (no changes)");
    }
    Ok(())
}

fn cmd_freeze(root: &PathBuf, freeze: bool) -> Result<()> {
    let project = Project::new(root.clone());
    let mut state = project.load_state()?;
    state.frozen = freeze;
    let today = rfcbox::project::today_utc();
    state.history.push(format!(
        "{today} {}",
        if freeze { "freeze" } else { "thaw" }
    ));
    project.save_state(&state)?;
    if freeze {
        println!("set frozen: sync will report notices without applying; add/remove refuse (§9.3)");
    } else {
        println!(
            "set thawed: {} pending notice(s) from the frozen interval carry into the next sync (§9.3)",
            state.pending_notices.len()
        );
    }
    Ok(())
}

fn cmd_export(root: &PathBuf) -> Result<()> {
    let project = Project::new(root.clone());
    let set = project.require_set()?;
    let c = ctx()?;
    let today = rfcbox::project::today_utc();
    // Traceability from scan's evidence: re-run the scan read path.
    let mut out = String::new();
    out.push_str(&format!("# rfcbox export — {today}\n\n"));
    out.push_str(&format!(
        "provenance: corpus snapshot {} · crate {} · baselines: {:?}\n\n",
        c.manifest.snapshot_sha256,
        c.manifest.crate_version,
        project.list_baselines()
    ));
    out.push_str("## traceability matrix (declared RFC × where the project uses it)\n\n");
    // Evidence: reuse scan by collecting strong citations again.
    let mut strong: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    for f in files {
        let Ok(bytes) = std::fs::read(&f) else { continue };
        if bytes.len() > 2_000_000 {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        let rel = f.strip_prefix(root).unwrap_or(&f).to_string_lossy().into_owned();
        for (lineno, line) in text.lines().enumerate() {
            let lower = line.to_lowercase();
            let mut rest_line = lower.as_str();
            while let Some(pos) = find_rfc_marker(rest_line) {
                let after = &rest_line[pos + 3..];
                let num_str: String = after
                    .trim_start()
                    .chars()
                    .take_while(|ch| ch.is_ascii_digit())
                    .collect();
                if let Ok(n) = num_str.parse::<u32>() {
                    strong
                        .entry(n)
                        .or_default()
                        .push(format!("{rel}:{}", lineno + 1));
                }
                rest_line = &rest_line[pos + 3..];
            }
        }
    }
    for e in &set.rfcs {
        let ev = strong
            .get(&e.number)
            .map(|v| v.iter().take(5).cloned().collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "no evidence found".into());
        out.push_str(&format!(
            "- RFC {} ({}) — used at: {}\n",
            e.number,
            c.index
                .record(e.number)
                .map(|r| r.title.clone())
                .unwrap_or_default(),
            ev
        ));
    }
    out.push_str("\n## declaration (rfcs.json)\n\n```json\n");
    out.push_str(&serde_json::to_string_pretty(&set)?);
    out.push_str("\n```\n\n## latest verify report\n\n```\n");
    let rep = audit_project(&project, &set, &c.index, &today)?;
    out.push_str(&rep.render());
    out.push_str("```\n");
    std::fs::create_dir_all(project.reports_dir())?;
    let path = project.reports_dir().join(format!("export-{today}.md"));
    std::fs::write(&path, &out)?;
    println!("export written to {}", path.display());
    Ok(())
}
