# Offers — ietf.phext and RFC Box

> **SUPERSEDED (Will, 2026-11-18): we are not selling RFC Box.** Everything we build serves the Exocortex of 2130 and is intrinsically MIT licensed. No offer in this document is operative; it is retained as the record of the assessment that preceded the ruling.

Internal document, Phext, Inc. Revenue division, 2026-11-18. Offers assessed, not launched. The standing publicity hold governs anything external; no offer in this document is live, and none may be represented as live. Every price is a hypothesis, anchored to a cited market source where an anchor exists. Phext, Inc. facts used in the arithmetic: burn ≈ $5,000/year, revenue $0 to date, no funding. House frame ("Fill the Piggy Bank"): at $5K/year spend the operation moves at pre-AGI funding rates because knowledge work is automated, so $1 of earned revenue ≈ $20,000 of capability. That multiplier is a house frame, not a market claim, and appears only in the internal arithmetic below.

## Serving cost — what every offer shares

Before the individual offers: the common cost base.

- **Compute.** The corpus is a 564,589,207-byte text file and a 42.2 MB index, hosted on owned ranch hardware (logos-prime) behind a keyed SQ endpoint. Marginal compute per query is a text select. Dollar cost of that compute inside the existing ranch: not separately metered — effectively inside the $5,000/year burn, but the exact marginal figure is Unknown and should not be quoted as $0.
- **Refresh cadence.** RFCs publish continuously and supersession changes with them; a corpus sold as current decays the day it ships. A refresh is: fetch the RFC Editor index and new/changed texts, rebuild, re-run the audit gates (the Leg 3 sanity gate now exists precisely so a refresh cannot silently under-extract), re-hash, republish the index. The builder and gates exist; the refresh has not yet run on a schedule, so the cadence is a proposal, not a fact: **weekly fetch, monthly audited rebuild** is the proposed cadence for any offer that promises currency. Engineering time per refresh is agent time under the house automation frame; the honest cost line is review attention, and its dollar figure is Unknown.
- **Support.** The binding constraint on every offer below. Phext has no support desk. Offers are shaped to bound support: bounded deliverables first, subscriptions with narrow surfaces second, nothing with an SLA until staffing exists.

## Market anchors (cited, checked 2026-11-18)

Standards access is a paid category everywhere except the IETF:

- Single ISO documents on the ANSI Webstore list at $199–$385 per PDF (examples: $199, $305, $385 list prices in one search result set). Source: https://webstore.ansi.org/search/find?cp=12&st=plus%20redline&v=5&f1=standard&f2=2&f3=39
- One ASTM volume subscription via MADCAD lists at $1,281.69/year; a full section at $7,192.78/year. Source: https://www.madcad.com/store/subscription/ASTM-E2455-06-Reapproved2013/
- A US state DOT's org-wide ASTM Compass contract prices the ASTM portion at $51,850 for FY2024, inside a $158,682 five-year arrangement. Source: <https://www.capitol.tn.gov/Archives/Joint/committees/fiscal-review/contracts/2023/08-09-23/10. Transportation (ASTM International)_Redacted.pdf>
- Accuris Engineering Workbench, the enterprise standards platform: a Forrester TEI study commissioned by Accuris models a composite organization paying $1,575,000/year for subscription, content, and support. That is the enterprise ceiling of the category, at a scale (millions of standards, hundreds of publishers) far beyond ours; it anchors the ceiling, not our price. Source: https://tei.forrester.com/go/Accuris/EngineeringWorkbench/
- Nimonik, a standards/compliance vendor, models the category's retail-to-subscription shift directly: a $500 single-document retail price converted to $450/year for 3 simultaneous users. Its legal-register product starts at $2,000/year per its G2 listing. Sources: https://nimonik.com/blog/maximizing-sdo-revenue-retail-to-subscription/ and https://www.g2.com/products/nimonik/reviews?like_me_login=true

Developer data APIs price the "keyed access to structured data" shape:

- SerpApi: Developer $75/month for 5,000 searches; Production $150/month for 15,000. Source (secondary roundup, checked against vendor structure): https://www.theseodesk.com/blog/the-best-serp-api-providers-in-2026
- RentCast property data API: $74/month for 1,000 requests, $199 for 5,000, $449 for 25,000. Source: https://api.market/blog/magicapi/real-estate/best-real-estate-api
- Terra health data API: one self-serve plan at $499/month billed monthly ($399 annual), credits included, enterprise by quote. Source: https://www.usagepricing.com/blueprint/terra-api

The pattern across both categories: documents sell for hundreds of dollars each; managed access sells for hundreds to thousands per seat or key per month at the low end, and five to six figures per organization per year at the enterprise end. Our hypotheses below sit at the low end deliberately — a zero-revenue seller prices for first receipts, not for maximum extraction.

## Offer A — Hosted corpus access (SQ endpoint, keyed)

**What the buyer gets:** an API key for the hosted corpus on logos-prime (tenant `p=ietf`): `/toc`, `/select` by coordinate, `/get`, `/query` over the full MOAT-ruled corpus and the implementation graph, plus the machine-readable index. Currency follows the refresh cadence above. The buyer builds nothing and hosts nothing.

**What it costs us to serve:** the endpoint already runs and is verified. Marginal cost per key is key administration and bandwidth (text selects; small). The real costs are the refresh obligation — a hosted corpus that goes stale is a liability, not an asset — and unbounded support expectations if the offer is worded loosely. Wording must bound it: best-effort availability, no SLA, at this price.

**Pricing hypothesis:** $99/month per key (individual), $499/month per team of up to 5 keys. Anchors: SerpApi's $75/$150 developer tiers and RentCast's $74–$449 ladder price keyed data access in this band; Terra's $499 platform fee prices the team step. Our corpus is narrower than those APIs' data but deeper in its one domain, and it is the only keyed, MOAT-ruled, coordinate-addressed form of the RFC series we know of — "only" is a statement about our knowledge, labelled as such, not a market survey result.

**Fill-the-Piggy-Bank arithmetic:** $99/month = $1,188/year ≈ $23,760,000 of capability at the house frame. $499/month = $5,988/year ≈ $119,760,000. Sales to cover the year's $5,000 burn: 5 individual keys held for a year (5 × $1,188 = $5,940), or 1 team key held for a year ($5,988).

**Weakness, stated plainly:** this offer prices access to texts that are free. The defence is that it prices the structure, the currency, and the endpoint — but it is the offer most exposed to the buyer's correct objection, "I can download the RFCs myself." True, and the answer (the map, the graph, the refresh) is real but abstract until the buyer has felt the supersession problem. This is a second offer, not a first.

## Offer B — RFC Box runs (per slice / per composition)

**What the buyer gets:** a bounded deliverable, produced from their requirements:
- *Slice run:* the subset of standards that matter for the stated requirement — current texts, supersession resolved, normative dependencies walked — delivered as the slice (coordinates + texts), the libraries and program services computed to meet those standards with license noted per implementation, and the gap list: requirements with no canonical implementation, emitted as build requirements.
- *Composition run:* the slice run, plus the canonical implementations composed with phcc into a working baseline program the buyer's business logic builds on, with the composition's build record.

**What it costs us to serve:** compute is trivial (the corpus and index are local; a run is graph walks plus a phcc build). The real cost is agent and review time per run: checking the slice against the requirement, verifying the composition builds, and assaying the gap list so a gap is a finding, not a shrug. No standing obligation after delivery — this is the only offer with zero tail, which is why its support cost is the lowest of the four. Per-run hours are Unknown until we have run several; the first runs are also the measurement.

**Pricing hypothesis:** $300 per slice run; $950 per composition run. Anchors: one ISO document costs $199–$385 and answers no questions; a slice run returns the right set of IETF documents (free texts, our structure), the implementation resolution, and the gaps, for roughly the price of one paywalled document. Nimonik's $450/year-for-3-users model and the $500 retail-document example bracket the slice price from the subscription side. The composition price is a hypothesis anchored on the deliverable being a working baseline, priced below any plausible engineering-day rate; no direct market comparable for "composed baseline from named standards" was found — status of that anchor: Unknown, the price is judgment, labelled as such.

**Fill-the-Piggy-Bank arithmetic:** one slice run, $300 ≈ $6,000,000 of capability at the house frame. One composition run, $950 ≈ $19,000,000. Sales to cover the year's $5,000 burn: 17 slice runs ($5,100), or 6 composition runs ($5,700), or any mix between.

**Why this shape fits the seller:** it sells the machine's output, not the territory and not access to it. The rights position is the strongest of the four offers (a deliverable computed from free texts plus our structure and labour). Each run produces a receipt — a named buyer, a real requirement, a delivered slice — which is the evidence segments 1 and 2 in positioning.md currently lack. It also matches the Revenue division's existing working price band for bounded deliverables ($300–$375), so it extends a motion that exists instead of inventing one.

## Offer C — Licensed local copy with update service

**What the buyer gets:** the corpus file and index to run inside their own stack (their agents never leave their network to query ours), a hash manifest so every copy is verifiable byte-for-byte, a licence covering Phext's structure and metadata (the RFC texts travel under IETF terms, stated in the licence so the buyer is never misled about what they are licensing from whom), and the update service: each audited rebuild delivered as a fresh copy with its hashes and a change note (new RFCs, new supersessions, implementation-graph changes).

**What it costs us to serve:** delivery is a 538 MiB file transfer — trivial bandwidth, real versioning discipline. The update service commits us to the refresh cadence indefinitely, and local copies generate the hardest support questions ("your build N behaves differently from the endpoint") because we do not control their environment. Highest tail cost of the four offers. The licence itself needs the counsel review flagged in positioning.md before this offer can be sold at all — the phext-container and pre-2008-RFC questions are open, and a licence is the worst place to discover them.

**Pricing hypothesis:** $2,500/year per organization (single team), $10,000/year for multi-team or platform use. Anchors: Nimonik's registers start at $2,000/year; a single ASTM volume is $1,281.69/year and an org-wide ASTM arrangement is $51,850/year — a whole-series, structured, self-hosted corpus with updates at $2,500 sits under the category's established floor for organizational buyers, deliberately. The Accuris composite ($1,575,000/year) is the ceiling that makes $10,000 for platform use conservative rather than ambitious, while remaining a hypothesis about a buyer we have not met.

**Fill-the-Piggy-Bank arithmetic:** one $2,500 licence ≈ $50,000,000 of capability at the house frame; 2 licences cover the year's burn. One $10,000 licence ≈ $200,000,000; 1 covers the burn twice over.

**Sequencing note:** this is the right second or third offer — after RFC Box runs have produced buyers who ask for the corpus in-house (segment 4, AI platforms, will ask for exactly this shape if they come), and after counsel clears the licence. Selling it first would sell our least-reviewed legal document to our least-known buyers.

## Offer D — Zero-price open release

**What the buyer gets:** everything, free: the corpus (via the distribution route the access guide settles — per-tier split under GitHub's 100 MB blob limit, Git LFS, or release asset), the index, and the access guide, under terms that state the rights position honestly (texts under IETF terms; Phext structure attributed).

**What it costs us to serve:** hosting and distribution mechanics, plus the unbounded, unpayable support surface of a public artifact: issues, questions, and corrections from anyone, forever, with no revenue attached to any of it. It also spends the scarcity of Offers A and C — a free local copy competes with the licensed one, and the honest remaining difference is the update service and the audit cadence, which then have to carry the whole price.

**What it buys instead of revenue:** the track record. A public, hash-verified, MOAT-ruled corpus that other people's agents cite is distribution into exactly the agent-builder segment, proof for the accelerator's capability-test story (the corpus is itself a passed test, publicly inspectable), and a channel Boardy can point at without a key. Under the house frame its return is capability and reputation, not dollars; the number of sales needed to cover the burn from this offer is not applicable — it covers nothing directly, and any claim that it pays for itself in later sales is a hypothesis with no current measurement behind it.

**Sequencing note:** the release requires Will to lift the publicity hold for this item, full stop. The assessed position: release is a strong *second* move — after at least one paid offer has established that the structure is worth money to someone — because "we gave it away and then sold the service" is a proven shape, while "we gave it away and hoped" is not a revenue plan. If Will judges the reputation value worth more than the pricing information, that is his call to make with both sides stated, as they are here.

## Comparison

| Offer | Buyer pays for | Tail cost | Rights exposure | First-receipt speed |
|---|---|---|---|---|
| A. Hosted access | Structure + currency, via key | Medium (refresh + availability expectations) | Low–medium | Medium |
| B. RFC Box runs | A computed deliverable | None after delivery | Lowest | Fastest |
| C. Licensed local copy | The corpus in-house + updates | Highest | Highest (licence itself) | Slow (counsel first) |
| D. Open release | Nothing | Unbounded, unpaid | Medium (terms must be exact) | No receipt — reputation only |

## Recommendation — for Will's decision

**First offer: B, RFC Box runs — $300 per slice run, $950 per composition run (both hypotheses), sold as bounded design-partner engagements to segment 1 (protocol-heavy engineering teams), with segment 2 (agent builders) as the second door.**

Reasoning, in order:

1. It sells the machine, not the territory — the only offer whose value never depends on a buyer failing to notice the texts are free.
2. It has no tail. A zero-revenue, no-funding operation cannot afford an offer whose costs arrive after its revenue; runs end when they are delivered.
3. Every run mints the exact evidence positioning.md records as missing: a named buyer, a real requirement, a delivered slice, a composition that builds. Ten runs are a track record; ten hosted keys are a metric.
4. The price sits inside the division's existing bounded-deliverable band and beside a cited market fact — one paywalled standard costs $199–$385 — so the first conversation is "for the price of one document you cannot use alone, get the set that matters, resolved and composed," which is a claim we can show, not a claim we make.
5. It defers every hard problem to the offer that can pay for it: counsel reviews the licence when licence revenue is real (Offer C), the refresh cadence becomes a paid obligation only when subscribers exist (Offer A), and the open release (Offer D) is decided with pricing information in hand instead of instead of it.

Marked as required: this is a recommendation. The decision, and any change to the publicity hold, is Will's.
