# Leg 3 Fix Report — ietf.phext reference extraction (2026-11-18)

Response to Judge's NOT READY verdict (Leg 3 FAIL). Fix executed on plain-view; this report's numbers are from the regenerated ietf-source.json.

## Root cause
The RFCXML-v3 era changed the Table of Contents, not the References section. V2-era ToCs pad entries with dot leaders and page numbers, so they never match a header pattern. V3 ToCs have neither: a top-level ToC line reads exactly `   23. References` — matching the extractor's strict header regex. The old extractor trusted the FIRST header match as the References block; that was the ToC line. Its "block" ran to the ToC's own "Authors' Addresses" line, contained zero citations, and was recorded `extraction: "parsed"`. The genuine body headers were never reached. RFC 9110's 9 "unspecified" edges were the same accident with a longer ToC block.

Smaller defects: numeric-labelled entries wrap "RFC" at end-of-line (`"..., RFC\n 2533, ..."`), invisible to the single-line citation pattern; one phantom from that wrap meeting a source-text typo.

## The fix (extract_refs.py; old version preserved as extract_refs_old.py)
- Candidate scoring + union: every header candidate's block is bounded, segmented, scored; all blocks yielding >=1 edge are unioned with kind-precedence. The ToC block scores 0 and drops out; the normative/informative split now comes from the body sections.
- New citation forms inside blocks only: `DOI 10.17487/RFC...`, `rfc-editor.org/info/rfc...`, line-wrapped `RFC\n NNNN`.
- Sanity gate (structural): zero edges + bracketed `[RFC...]` citations present + a References-shaped heading anywhere => recorded "unparsed", never "parsed". Silent under-extraction labelled as success is now impossible in the output.
- Phantom filter: extracted targets outside the indexed RFC number line are dropped and reported — exactly 1 existed: RFC 3426 -> "25434", a typo in 3426's own text for RFC 2543 (true edge present via its [RFC2543] label). Keeps forward edges and cited_by exactly symmetric.

## Era boundary, measured
1,363 entries were shipped parsed-with-zero and now yield edges. The ToC trap is near-total from RFC 8750 onward (~95% per 250-band), partially penetrant 8500-8749 (97), ~37 isolated cases from RFC 767 (mostly the wrapped-citation form). Judge's ~8800 estimate lands within 50 numbers of the measured 8750 cliff.

## New counts
- States: parsed 8,391 / unparsed 1,465 / not_applicable 7.
- Edges: normative 44,214 / informative 39,331 / unspecified 12,623 = 96,168 total (was 70,932; +25,236).
- Gate flips: 4 (RFC 1521, 2626, 3347, 4324) — old parsed-zero, now honestly unparsed.
- Remaining parsed-with-zero: 214; at >=8800 exactly 7 (9075, 9226, 9307, 9311, 9318, 9657, 9998), each verified genuine (zero bracketed RFC citations in text).

## Judge test cases
| RFC | Before | After | cited_by |
|---|---|---|---|
| 9000 | 0 edges | 19 normative + 22 informative = 41 | 0 -> 103 |
| 9001 | 0 | 11 + 6 = 17 | 0 -> 25 |
| 9002 | 0 | 6 + 15 = 21 | 0 -> 17 |
| 9110 | 9 unspecified | 19 normative + 53 informative = 72 (source's own SS19.1/19.2 split) | 0 -> 96 |
9110's text carries 57 distinct [RFC...] citations; all 56 non-self ones are in the edge set. Forward total 96,168 == cited_by total 96,168, population-wide.

## Regressions
RFC 8446 (22/25) and 4271 (5/22) edge sets identical to shipped; 6298 identical. Population-wide, of 1,149 previously-edged entries whose records changed: true edge loss = 0 — every change is growth or re-bucketing to the correct kind.

## Artifacts
- ietf.phext: 564,589,207 bytes, sha256 9ecd79461f646c84838f697488ebf46d900bf5dd1cfde2b9967db3f6f426d165
- ietf-source.json: sha256 0fbf5e99c9e4fc1c3b4c5d3d732c70645588a3031e11de967262175eb138fa9b
- ietf.index.json: sha256 49f99dfa5eccfc0db234bbd24cd01be7f6150246d3a740d3b58bbbc93bda8f52 (10,067 scrolls)
- Builder --check on plain-view: CURRENT. verify_ietf.py: ALL PASS (incl. RFC 9110 live spot-check byte-identical).
- Legs 1-2 untouched: index edges identical (obsoletes 1,544 = obsoleted_by 1,544; updates 2,169 = updated_by 2,169); implementations 6,888 unchanged.
