# Positioning — ietf.phext and RFC Box

Internal document, Phext, Inc. Revenue division, 2026-11-18. Draft under the standing publicity hold: nothing here is published, sent, or announced. Every number is either a verified fact about the artifact (marked as such), a cited market source, or labelled a hypothesis. Where we do not know, this document says Unknown.

## The artifact, in verified facts

- One phext holding every IETF standard: 10,052 records — 9,863 issued RFCs plus 189 not-issued numbers, annotated rather than dropped.
- 9,856 full texts. 1,385 superseded RFCs kept in full, annotated, with redirects to their successors.
- Every entry is MOAT-ruled: it carries a status, a basis, and an evidence pointer.
- 96,168 reference edges between RFCs (after the Leg 3 extraction fix; see the audit note below), plus a reverse index.
- 6,888 implementation links across 88 canonical open-source projects. Tier 1 complete.
- 564,589,207 bytes. SHA-256 `9ecd79461f646c84838f697488ebf46d900bf5dd1cfde2b9967db3f6f426d165`.
- All three independent audit legs PASS. The build is READY.
- Access surfaces are settled in the access guide (`~/workspace/your_files/ietf-access.md`): the plain-text file, the keyed SQ endpoint on logos-prime (tenant `p=ietf`), and the machine-readable index (`ietf-source.json`).

One audit fact is itself positioning evidence: the first build's reference extractor silently under-extracted RFCXML-v3-era RFCs while labelling them "parsed." An independent audit leg caught it, the extractor was fixed, a sanity gate was added so a silent empty extraction can no longer be labelled parsed, and the re-audit passed at 96,168 edges. A corpus whose seller can show a defect found, fixed, and re-audited is a different product from a corpus whose seller has never looked.

## The rights analysis — what a buyer would actually pay for

This section is the load-bearing wall of the positioning. It is an analysis, not legal advice; the open items at the end need counsel before any commercial redistribution.

### What the IETF gives away

RFC texts are not Phext's to sell. The IETF Trust's own FAQ states the general rule plainly: reproduction of whole RFCs, including translation, "has been allowed and encouraged" since the beginning of the series, and the IETF Trust and the RFC Editor place no restrictions on it; most RFCs carry the phrase "Distribution of this memo is unlimited." Copyright in RFCs remains with the original authors (or their employers) — the IETF has never required a transfer. The Trust's Legal Provisions grant any person a non-exclusive, royalty-free, worldwide license to copy, publish, display, and distribute IETF RFCs in full and without modification, and to reproduce unmodified portions with attribution and legends preserved. Code Components extracted from RFCs are separately licensed for use in software under the TLP's BSD-style terms.

Sources: IETF Trust FAQ, https://trustee.ietf.org/about/faq/ ; IETF Trust Legal Provisions (summary in search record, https://www.ietf.org/proceedings/72/slides/ipr-2.pdf); RFC 5378 (BCP 78).

### What the IETF does not give away

- **Modification rights in the text.** The TLP does not grant a general right to modify RFC text or create derivative works of it outside the IETF standards process; modification of non-code extracts requires the original authors' permission, per the Trust FAQ. Anyone may redistribute the text; no one may sell a rewritten RFC as an RFC.
- **Currency.** The RFC Editor gives away documents one at a time. It does not give away a maintained, machine-checkable answer to "is this the current text, what obsoleted it, what does it normatively depend on, and what depends on it" across all 10,052 records at once.
- **The implementation graph.** Which open-source project canonically implements which standard, with the evidence (the basis field) for calling it canonical, and the license that governs composing it — that mapping is Phext's work product, not an IETF publication.
- **The computation.** Turning a set of requirements into the subset slice of standards that matter, the libraries and services that satisfy them, a composed baseline, and an explicit list of gaps — that is RFC Box, a program, not a document collection.

### The strongest sentence of this analysis

**Phext cannot sell the RFCs — anyone may copy and distribute them in full, free — Phext can only sell what the IETF does not give away: a verified map of which texts are current, what they depend on, what code canonically implements them, and a machine that turns a requirement into that slice and a working baseline.**

Sell the map and the machine. Never pretend to sell the territory.

### What is NOT sellable, and why

- **The RFC texts themselves, or exclusive access to them.** Freely reproducible by anyone, in full, under the terms above. Any offer that prices the text is pricing a public good and will be found out by the first competent buyer.
- **Copyright in the RFCs.** It sits with the original authors and their employers. Phext holds none of it and cannot license what it does not hold.
- **Official status or IETF endorsement.** Phext is not the IETF, the RFC Editor, or a delegate of either. The corpus is a third-party structure over IETF material and must be described as exactly that.
- **Modified RFC text presented as the RFC.** Not licensed, and corrosive to the one asset the corpus has: byte-level trust (each scroll carries the SHA-256 of the text as fetched).
- **The canonical implementations themselves.** OpenSSL and the other 87 projects are their authors' work under their own licenses. What Phext sells is the resolution — which project, on what basis, under which license — and the composition service. The license field in each implementation record exists because composition is a legal act, not only a technical one.
- **Standards outside the IETF series.** The corpus contains no ISO, ASTM, IEEE, or TIA documents. Those bodies sell their texts (see the price anchors in offers.md); nothing in our positioning may imply coverage we do not have. Where a buyer's requirement names a non-IETF standard, RFC Box's correct output is a gap, not a substitute.
- **Certainty the basis does not support.** "Canonical implementation" is a curation judgment recorded with its basis, not an IETF designation, except where a basis explicitly cites one. Selling the judgment as a designation would be a false claim about our own metadata — the MOAT discipline forbids it internally; positioning must not reintroduce it externally.
- **Support and availability commitments beyond our capacity.** Phext, Inc. runs at roughly $5,000/year of burn with no funding (verified house fact). Whether we can honour any given SLA is Unknown until someone scopes the staffing. Positioning promises the artifact and its audit trail, not an operations desk.

### Open rights items (Unknown until counsel answers)

- Whether wrapping RFC text in phext scroll structure with Phext headers counts as distribution "without modification" under the TLP, or as a derivative presentation — the texts themselves are unaltered and hash-verified per scroll, but the legal characterization of the container is Unknown.
- Pre-2008 RFCs sit under earlier IETF policy documents (RFCs 2026, 3978, 4748 era terms), not RFC 5378 alone. The corpus spans the whole series; a commercial redistribution review has to cover the older terms too. Status: Unknown, flagged, not waved through.
- The implementation graph records facts and links about third-party projects. Facts are not copyrightable in the US as a general principle, but the selection-and-arrangement and database-rights position outside the US is Unknown without advice.

## Segments, ranked by revenue fit

Ranking criterion: severity of the problem × existing budget for this class of spend × reachability for a zero-revenue, no-funding seller × how directly our receipts answer the buyer's objection. Each segment states the buyer's problem in the buyer's own plain terms (a paraphrase of the problem as buyers describe it in the market, not a fabricated quotation or testimonial — we have no customers and claim none).

### 1. Engineering teams implementing protocol-heavy products

**Their problem, in their words:** "We have to build to the standards, the standards reference each other, half of what search returns is the superseded version, and every engineer on the team is working from a different pile of PDFs." For teams in radio, telecom, and adjacent regulated work — the S/MIME + TLS + AES-256 composition case is the type example, and TIA-standard work sits in the same posture — the failure mode is concrete: implement against an obsoleted RFC, miss a normative dependency, or pick a library whose license cannot ship in the product.

**What changes for them:** RFC Box takes their requirements and returns the slice that matters — current texts only, supersession already resolved — then the libraries and services that meet those standards, license-checked at resolution time, composed with phcc into a baseline their business logic builds on. Gaps come back as explicit build requirements instead of being discovered in certification.

**Evidence we can show:** the corpus counts and hash above; three audit legs PASS, including the Leg 3 defect-and-fix record; 6,888 implementation links across 88 projects with a basis and a license on each; the access guide's worked composition pattern. What we cannot yet show: a completed composition inside a customer's product. Status of that proof: Unknown — it does not exist yet, and this segment is where it has to be produced first.

**Why first:** this is the only segment that already spends money on standards access as a category (see offers.md: single ISO PDFs at $199–$385, an ASTM volume subscription listed at $1,281.69/year, an org-wide ASTM Compass contract at $51,850/year). The budget line exists; we do not have to invent it.

### 2. Agent builders who need ground-truth standards access

**Their problem, in their words:** "My agent answers protocol questions from model memory. It sounds right. Sometimes it is citing a standard that was obsoleted years ago, and nothing in the output tells me which times are which."

**What changes for them:** a keyed SQ endpoint their agent can actually call — index first, select by coordinate, cite the coordinate and the scroll's SHA-256. Answers become checkable against the text instead of plausible. The access guide documents the consumption pattern for Claude, ChatGPT, Grok, and local agents separately, because each platform's limits differ.

**Evidence we can show:** the endpoint is live and gated, verified end-to-end 2026-11-18 with a byte-identical `get` round-trip; the index loads first so no platform has to pour 538 MiB into a context window. Phext's own agents already work this way internally (see segment 4).

**Why second, not first:** the fit is exact and the adoption friction is lowest — this is Boardy's channel and the accelerator's population — but willingness to pay is unproven and likely thinner per seat than segment 1. Hypothesis, not measurement: agent builders are design partners and distribution first, revenue second. They are also the segment most likely to find our errors, which is worth money we do not have to spend on QA.

### 3. Internal use — Phext's own infrastructure reference

Not a buyer. Ranked here because on use-fit alone it outranks every external segment, and because it is the evidence engine for segments 1 and 2.

**The problem, in our words:** Phext builds protocol software — SQ, phext-lattice, mail and hosting layers — and its agents were resolving standards questions from model recollection, four substrates with four recollections.

**What changed:** one addressable corpus every Phext agent reads from; protocol work moves from recall to verification against the text, at a coordinate, with supersession status attached.

**Evidence:** the corpus exists because internal demand built it in one day; the access guide's "local agent" pattern is the internal pattern written down. Every internal use is a receipt the external segments can be shown. Revenue from this segment: $0, by definition. Its value is that segments 1 and 2 are buying something the seller demonstrably uses to build its own infrastructure — the strongest claim a zero-revenue seller can make without inventing a customer.

### 4. AI platforms whose models recall standards from memory

**Their problem, in their words:** "Our model hallucinates citations and states obsoleted protocol behaviour with full confidence, and our enterprise customers are starting to test for it."

**What would change for them:** a licensed local copy of the corpus and index inside their retrieval stack, with the update service keeping supersession current — ground truth their models retrieve instead of recall.

**Evidence we can show:** the same artifact, audits, and hashes. Nothing else. We have no load testing at platform scale, no SLA, and no support organization; the honest Unknown list for this segment is longer than the evidence list.

**Why last despite the largest budgets:** the sales cycle is enterprise-length, the diligence will (correctly) probe exactly the capacities we cannot yet demonstrate, and a failed platform deployment would burn the track record the other segments are building. Keep as inbound-only: if a platform comes to us, the licensed-copy offer in offers.md is the shape. Do not chase this segment in the first year. Hypothesis about timing, stated as such.

### Cut from the candidate list

- **Individual developers and hobbyists as a paid segment.** The RFC texts are free and these users know it; per-seat revenue would be small and support cost per dollar high. They are served by the zero-price option in offers.md, where they function as distribution and error-finding, not as a segment.
- **Standards bodies and the IETF itself.** They are the source, not a customer, and any appearance of selling their own texts back to them would be both wrong on the rights analysis above and reputationally fatal.
- **Compliance and certification firms.** Plausible on paper — Unknown in fact. We have no evidence about their buying process for IETF material specifically, and their core spend is on paywalled SDO texts we do not carry. Not ranked rather than ranked on a guess.
