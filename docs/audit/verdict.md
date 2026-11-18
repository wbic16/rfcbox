# MOAT Conformance Audit — ietf.phext — VERDICT
Auditor: Judge (PH: Judgement) · Task 25.6.9/8.6.10/6.6.9 · Audited 2026-11-18
Artifact: ietf.phext 564,160,499 B, SHA-256 5c576b3737c65d12b24bc1dbb8329639326838c986ebf8882bdbb095541ac045 (independently re-hashed on this VM — matches the build's claim). Audit scripts + raw results: ~/workspace/judgement-division/ietf-audit/ (population checks over ietf-source.json; scroll-level checks via the reference parser, my own samples and seeds — not the builder's verifier re-run).

# VERDICT: NOT READY — one leg fails. The corpus and implementation legs pass at full strength; the dependency-graph leg fails on the newest-era RFCs.

---

## Leg 1 — Corpus / MOAT conformance: PASS
- Population: 10,052 entries; unique RFC numbers 10,052; unique coordinates 10,052; parsed phext yields exactly 10,067 non-empty scrolls (10,052 RFC + Home + registry overview + T1 + T2 + 11 T3 shards); every RFC coordinate present.
- Basis conformance: every entry exactly one status + one basis; all 10,052 are (True, VERIFIED); **zero violations** of the MVO basis table. Evidence: 9,856 entries carry a text SHA-256; 7 carry a 404 fetch receipt (fetch_status unavailable, http 404); 189 are not-issued records (explicit fetch_status, no silent gaps in the number line).
- Counts reconcile exactly with the builder's claims and the index census: 9,863 issued + 189 not-issued; 9,856 full texts; 1,385 superseded; tiers T1 953 / T2 2,158 / T3 6,941; every entry carries a tier_rule; the tier overrides table is explicit data (5 overrides + number/title-pattern lists) — priority governed organization only; Tier 3 scrolls are complete texts, not stubs.
- Supersession: all 1,385 superseded entries carry obsoleted_by + successor_coordinates; successor coordinates match the successors' own coordinates with **zero mismatches**; index edges perfectly symmetric (obsoletes 1,544 = obsoleted_by 1,544; updates 2,169 = updated_by 2,169; zero asymmetries). My own 20-sample (seed 777, different from the builder's): every sampled scroll carries the SUPERSEDED annotation + successor coordinate + full text, and every registry line carries the redirect — 20/20.
- Live spot-checks (my picks, fresh fetches): RFC 9112, 7540, 9001 — live SHA-256 == entry receipt == scroll header receipt, all three.
- Point 5 provisos confirmed: the Home scroll states VERIFIED is record-only, and the classification decision (record = existence + text-as-served + index status) is printed in the corpus itself. The record's True does not bleed into the RFCs' normative claims.

## Leg 2 — Implementations: PASS, with one disclosed caveat standing
- 6,888 links across 88 projects; every link carries name, repo_url, license, language, role, basis, evidence — **zero missing licenses, zero missing evidence pointers** (full population).
- Basis honesty at project level: 81 VERIFIED (repo resolved + project docs name the protocol/RFC, per impl_checks.json receipts) / 7 ATTRIBUTED with stated reasons (keyword misses, mirror/Mercurial cases) — the ATTRIBUTED set is exactly the honest outcome the audit plan asked for.
- Tier 1 complete: canonical 678 / multiple 272 / none_identified 3 / unmapped 0. Tier 2 mapped (2,004 / 100 / 54). Tier 3 is explicitly "unmapped by design" in every entry's impl_status + impl_rule — an explicit recorded state, not a silent empty field.
- Co-canonical handling verified in samples (RFC 7540: nghttp2 canonical + curl, nginx also-implemented; RFC 9110: curl canonical + nginx, Apache httpd also-implemented) — no forced single winners in the samples; family-level assignment with per-RFC overrides is explicit in impl_rule.
- Standing caveat (disclosed by the builder in the Home scroll, verified present by this audit): util-linux's detected license is the root GPL-2.0 while libuuid is LGPL — the Home prose instructs re-confirmation before composition. This is a disclosed finding, correctly handled; it must be resolved before phcc composition consumes that record, but it does not block the corpus.

## Leg 3 — Dependency graph: FAIL (points 11 and 12)
What passes: index edges are exact and symmetric (above); forward references total 70,932 and the cited_by reverse total independently recomputes to exactly 70,932, with per-edge reverse consistency; mid-era extraction is genuinely good — RFC 4271, 6298, 8446: every recorded normative target verified present as a citation in the citing text (27/27 recorded for 4271 against its text). The era gradient for *unparsed* is honest at the old end (pre-1000 RFCs: 82% unparsed).

What fails — **the newest-format RFCs (RFCXML-v3 era) silently under-extracted while labelled "parsed":**
- RFC 8800+: of 1,248 fetched entries marked parsed, **1,188 (95%) carry zero reference edges** and 1,246 carry ≤2 — while their texts plainly cite other RFCs (RFC 9000's text alone carries 24 distinct [RFCxxxx] citations and a full References section; recorded edges: 0).
- Tier 1 casualties: RFC 9000, 9001, 9002 (QUIC) record **zero** references and zero cited_by; RFC 9110 records only 9 unspecified edges (its obsoletes) out of 57 distinct RFC citations in its text, and **cited_by = 0** — while RFC 8446, one format generation earlier, correctly shows 46 citers.
- Point 12 (reverse walkability) therefore fails on the most natural test cases: "what needs RFC 9110 / RFC 9000" resolves to *nothing* from the shipped structures, which is false against the texts the corpus itself holds. The reverse index is internally consistent (forward == reverse) — it is consistently built from an incomplete forward set for this era.
- Point 11 (honest incompleteness) fails in kind: an extraction that was defeated by the format is recorded as "parsed," the success state. The old-era failures were honestly labelled unparsed; the new-era failures were not. (The 8000–8799 band shows a milder version: 148 parsed-with-zero of 790.)
- Note on the 'unspecified' kind: as a category it is SOUND — refusing to fabricate the normative/informative split is the honest move, and this audit ratifies it. But its application compounds the failure here: RFC 9110's own text *does* make the split (§19.1 Normative / §19.2 Informative) and the record still says unspecified, because the extractor never reached the section.

## Judgement calls referred by the builder — ruled
1. **Not-issued inclusion (189 records, True/VERIFIED): SOUND.** The record's claim is non-issuance per the index; the index is its receipt; inclusion keeps the number line gapless and each record is explicitly annotated.
2. **'Unspecified' reference kind: SOUND as a category** (see above); its v3-era application is part of the Leg 3 failure, not of the category.
3. **Delimiter escaping (3 texts, [0xNN] markers) + Latin-1 normalization (34 texts): SOUND as disclosed.** Audit confirms exactly 34 utf8=False entries and 3 delim files (RFC 542, 674, 1142); transformations are noted in scroll headers; header hashes are of the original fetched bytes, so the receipt chain is intact. Consequence to keep visible: those 37 scroll bodies are not byte-identical to source — the header hash governs, and the corpus says so.
4. **Family-level implementation assignment with per-RFC overrides: ACCEPTED** — explicit in impl_rule per entry, overrides reviewable, Tier 1 complete.
5. **util-linux license caveat: CORRECTLY DISCLOSED** — stands as a pre-composition action item (re-confirm libuuid's LGPL), not a hidden defect.

## What would make it READY
One fix, graph leg only: make the extractor read the RFCXML-v3-era References format (or, as an interim honest state, reclassify the defeated entries from "parsed" to "unparsed" so the corpus stops claiming edges it does not have), regenerate cited_by from the corrected forward set, and re-run this audit's Leg 3 checks (8800+ zero-ref count, 9110/9000 reverse walks, normative samples for the era). Legs 1 and 2 do not need re-audit; their evidence stands. RFC Box should not consume the reference graph for RFCs ≥ ~8800 until this lands — index edges and implementation edges for those RFCs are unaffected and usable.
