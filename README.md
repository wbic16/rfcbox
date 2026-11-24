# rfcbox

A Rust builder/reader library plus CLI over **ietf.phext** — the phext corpus
that holds every IETF RFC in one coordinate space (10,052 records: 9,863 issued
RFCs plus 189 not-issued numbers, annotated; 9,856 full texts; 96,168 reference
edges; 6,888 implementation links across 88 projects; audited snapshot
2026-11-18, sha256 `9ecd7946…`).

The crate is useful the moment it installs: a small audited **baseline core**
of common web-development RFCs rides inside it, and every other RFC is
represented by a **discovery stub** (our generated summary + its top-50
signature words), so search runs corpus-wide with no fetch at all.

## Licensing — split by construction

- **Our code is ours: MIT** (see `LICENSE`). The builder, reader, graph
  machinery, and fetch machinery are Phext's work product.
- **The RFC texts are not ours and are never relicensed.** Copyright in the
  RFCs sits with their original authors (or their employers); the texts
  travel under IETF terms wherever this crate carries, fetches, or emits
  them. No crate artifact presents the texts as MIT-licensed, and the
  crate's license field does not cover them.

> **TLP notice.** The RFC texts reproduced by this crate are IETF documents
> distributed under the IETF Trust's Legal Provisions (TLP); in February 2026
> the Trust's role and copyrights passed to the IETF IPMC, carrying the same
> terms. Outside the IETF standards process, the TLP licenses copying and
> distribution of RFCs **in full and without modification**; this crate
> therefore carries, fetches, and emits texts byte-for-byte as published by
> the RFC Editor, verifies each against its recorded SHA-256 before serving
> it, and refuses — loudly — on mismatch. Code components extracted from
> RFCs (ABNF, MIB modules, sample code) ride TLP §4's Revised BSD License,
> except for RFCs bearing §6.c restrictive legends, whose code is not
> BSD-available; the crate's manifest records each text's legend class.

This crate is a **third-party structure over IETF material**. It is not an
IETF or RFC Editor product, and nothing in it is an IETF designation.
"Canonical implementation" in the corpus is a curation judgment recorded
with its basis, never an IETF designation.

## Good-netizen defaults (have teeth)

- The baseline set is a small core, audited and included in the crate.
  `sync` / `add` / `remove` work on the **target project's** declared set.
- The whole corpus is fetched **only** when you explicitly specify a
  package location for it. No default path does it.
- All fetching is rate-limited **in the fetch path itself** to human
  fetching speed: sequential, human-scale spacing, no parallel floods, no
  retry storms. A 429 or bot wall ends the run. The crate **cannot be
  configured to fetch faster** — there is no such setting.

## The declared set: `rfcs.json`

The target project (the project consuming RFCs — *not* rfcbox itself)
declares its set in `rfcs.json` at its root: per RFC, its number, the
SHA-256 of the text as fetched, its source, and its supersession state
(successor named, never silently swapped). `rfcs.json` is the source of
truth; the texts on disk are its verified materialization.

```console
$ rfcbox init                 # start rfcs.json from the baseline core
$ rfcbox add 6455             # declare + fetch the WebSocket RFC
$ rfcbox add 9110 --closure --depth 1 --kinds normative
$ rfcbox remove 6265
$ rfcbox sync                 # annotate supersessions, materialize the declared
$ rfcbox scan                 # document which RFCs the project actually uses
$ rfcbox audit                # verify: materialized? hash-verified? status? + legend receipts
```

## Search — the front door to `add`

```console
$ rfcbox search cache-control max-age        # keyword, over the set on disk + stubs
$ rfcbox search --intent "how should a client handle a stale cached response"
```

Intent search ranks candidates from the corpus's own structure — keyword
evidence, the reference graph, and the implementation graph — and **every
result carries its why**. A ranking is a ranking, never a verdict; thin
signals are stated plainly. Search never fetches and never adds anything
by itself.

## Waterfall commands

`baseline <name>` freezes the declared set as an immutable phase-gate
artifact; `diff` calculates the change record between gates; `freeze` /
`thaw` put the set under change control (sync reports notices, applies
nothing; add/remove refuse); `verify` is the verification-phase audit;
`export` emits the reviewer package (traceability matrix, declaration,
baseline, latest verify report).

## Corpus features

`subset 9110 --depth 1 --out ./slice` produces a self-contained,
byte-for-byte reproducible slice (texts verbatim, per-text hashes, the
edge set among members, a provenance manifest). `refresh --corpus
ietf.phext` discovers RFCs published after the snapshot, fetches them
verbatim at human speed, and appends them as scrolls to a **candidate**
snapshot — the audited artifact is never edited in place.

## Maintainers

`rfcbox-build --phext ietf.phext --source ietf-source.json --out .`
regenerates the shipped payloads (`assets/`): it runs the per-RFC legend
audit across every issued text, computes the signature words, builds the
discovery stubs under their size budget (an overrun fails the build),
and writes the manifest with full payload receipts.
