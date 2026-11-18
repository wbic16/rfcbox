# MOAT Conformance Audit — ietf.phext — LEG 3 RE-AUDIT
Auditor: Judge (PH: Judgement) · Task 25.6.9/8.6.10/6.6.9 · Re-audited 2026-11-18
Artifact under re-audit: ietf.phext 564,589,207 B, SHA-256 9ecd79461f646c84838f697488ebf46d900bf5dd1cfde2b9967db3f6f426d165 (independently re-hashed on this VM — matches; source 0fbf5e99…, index 49f99dfa…, both match the fix report). Supersedes nothing: the original NOT READY verdict (docs/audit/verdict.md) stands in history; this file re-runs Leg 3 only, per the original verdict's terms. Legs 1–2 evidence stands and their invariants were re-confirmed on the new artifact (below).

# RE-AUDIT VERDICT: LEG 3 PASSES. With Legs 1–2 already passed, ietf.phext is now READY.

## The fix, as verified (not as reported)
Root cause per the fix report: the RFCXML-v3 Table of Contents line matched the extractor's strict header regex, so the ToC was parsed as the References block. The re-audit does not rely on that account; it tests outcomes:

- **Population:** reference edges now total **96,168** (normative 44,214 / informative 39,331 / unspecified 12,623), independently recomputed from ietf-source.json; forward total == cited_by total, exactly (96,168 == 96,168).
- **Test cases (all match the fix report's numbers exactly):** RFC 9000 = 41 edges (19N/22I), cited_by 103 · RFC 9001 = 17, cited_by 25 · RFC 9002 = 21, cited_by 17 · RFC 9110 = 72 (19N/53I), cited_by 96. RFC 9110's §19.1/§19.2 split is now honored (it was "unspecified" before).
- **Edges trace to text:** in RFC 9110's §19.1, 15 of 19 recorded normative targets appear as [RFCxxxx] tags in the section; the remaining 4 are cited under symbolic tags which the extractor correctly resolved — the section's own sample line reads "[CACHING] … RFC 9111", and 9111 is in the recorded list. The edges are the text's citations, including its symbolic ones.
- **Reverse walkability (point 12):** RFC 9000's scroll header itself now reads "Cited by: 103 reference edges point at this RFC", and its normative edge line lists all 19 targets — the reverse direction resolves from the shipped scroll alone. RFC 9110 cited_by 0 → 96; RFC 8446 cited_by 46 → 312 (the increase is the newly-extracted modern citers — the expected signature of this fix).
- **Regression controls:** RFC 8446 normative 22 (unchanged), total 47 · RFC 4271 total 27, normative 5 (unchanged) · RFC 6298 normative 5 (unchanged). No true edge loss: totals only moved up, and the control edge sets are stable where the old extractor already worked.
- **The sanity gate (point 11):** parsed-with-zero-refs in the ≥8800 era fell from 1,188 to exactly **7** — RFC 9075, 9226, 9307, 9311, 9318, 9657, 9998. This audit tried to falsify the builder's "genuinely citation-free" claim against all seven scroll texts: every one carries **zero [RFCxxxx] citations and no numbered References section** (their citations are symbolic and point at non-RFC sources — papers, IETF-process pages, ISO documents). There are no RFC-to-RFC edges in those texts to record; "parsed, zero edges" is the truthful state for them. The 8000–8799 band's parsed-zero count also fell 148 → 3, and extraction states shifted parsed 8,395 → 8,391 / unparsed 1,461 → 1,465 — the gate reclassified a small residue honestly rather than forcing it through.
- **Measured cliff:** the failure boundary sits at ~RFC 8750 per the fix report, consistent with this audit's era buckets (8000–8799 mostly recovered, ≥8800 transformed).

## Legs 1–2 invariants on the new artifact (re-confirmed, not re-audited)
All 10,052 entries still (True, VERIFIED); index edges unchanged (obsoletes/obsoleted_by 1,544 each; updates/updated_by 2,169 each); superseded still 1,385; implementation links still 6,888 with zero missing licenses. The fix touched the reference graph only, as scoped.

## Standing notes
- RFC Box may now consume the reference graph across all eras, including ≥8800.
- The util-linux/libuuid license re-confirmation (Leg 2 caveat, disclosed in the corpus Home scroll) remains a pre-composition action item — unchanged by this re-audit.
- The defect this cycle caught — a success label on a defeated extraction — is now guarded by the builder's structural sanity gate; this audit verified the gate's output population, not just its existence.
