# RFC Box

One of the first multiversal effects we implement. **ietf.phext is the first phext in its data model.**

The coordinate space holds the absolute dependency graph. RFC Box generates an arbitrary dependency graph from it: given a snapshot of the RFCs and a user's specific requirements (which standards they care about), it returns the subset slice of all the standards that matter; from that slice it computes all the libraries and all the program services needed to work with code that meets those standards; and where there are gaps, it provides requirements to synthesize the missing program. Those gap requirements come back as build requests (ticketing: tasks.phext 25.8.12/11.7.14/6.7.9).

Program task: tasks.phext **34.5.9/7.6.10/6.5.9** · Data task: **25.6.9/8.6.10/6.6.9** (ietf.phext One Day Build, 2026-11-18)

## The data model (ietf.phext, 2026-11-18)

- 10,052 records — every RFC (9,863 issued + 189 not-issued numbers, annotated), MOAT-ruled: one status, one basis, evidence pointers per entry; superseded RFCs (1,385) kept in full, annotated, redirected — never pruned.
- Dependency edges in the coordinate space: index edges (obsoletes / obsoleted-by / updates / updated-by), reference edges extracted from each RFC's own References section, and implementation edges to the canonical open-source implementation per standard (6,888 links, 88 projects; Tier 1 complete) — the resolution step: name standards (e.g. S/MIME + TLS + AES-256) → canonical code → compose with phcc → a working baseline under business logic.
- Access: SQ on logos-prime, `http://100.107.204.62:1338/api/v2`, tenant `p=ietf`, API key required (obtain from Watch/Orion). Mirror: free-fall `~/ietf-mirror/`. Access guide: `docs/access.md` in this repo.

## Standing caveat

Reference edges for RFCs ≥ ~8800 are **not authoritative** until the RFCXML-v3 extraction fix lands and the graph leg is re-audited (Judge, PH: Judgement): the v3-era extractor silently under-extracted under a "parsed" label in the first build. Index edges and implementation edges in that range are unaffected. Slice computations must treat that range's reference edges as absent, not as truth.

## Documents

- `docs/access.md` — how every platform consumes ietf.phext (Reed, PH: Revenue)
- `docs/audit/` — MOAT conformance audit plan + verdict (Judge, PH: Judgement)
- `docs/verification/` — independent integrity evidence + hosting record (Watch, PH: DevSecOps)
- `docs/fleet-mirror.md` — fleet truth + mirror record (Rogue, PH: Evolution)
