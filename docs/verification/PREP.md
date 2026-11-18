# ietf.phext — Watch integrity & access leg (task 25.6.9/8.6.10/6.6.9)
Prepared 2026-11-18 08:33 CST, before build landing. Owner: Watch (PH: DevSecOps), tasking from Orion.

## State at prep time
- ~/workspace/phext/ietf.phext and ietf-source.json do NOT exist yet; tasks.phext status: "in progress — build dispatched 2026-11-18 (background coordinator, ranch-first)". Leg is in prep-and-standby per Orion's point 3.

## Leg 1 — corpus integrity sample
Universe established independently by Watch from the live RFC Editor index
(rfc-index.xml fetched 2026-11-18, 13,758,089 B, copy in this dir):
- 10,052 RFC entries, RFC 1 through RFC 10094.

Sample method (deterministic, fixed before seeing the build — no cherry-picking):
- Sort all RFC numbers; divide into 25 equal strata; take the element at each stratum center.
- Force-include the first (RFC 1) and last (RFC 10094) entries.
- Result: 27 RFCs — 1, 202, 604, 1006, 1408, 1810, 2212, 2614, 3016, 3418, 3820, 4222, 4624, 5026, 5429, 5831, 6233, 6635, 7037, 7439, 7841, 8243, 8645, 9047, 9449, 9854, 10094. (sample.json in this dir.)

Check procedure per sampled RFC, run when the build lands:
1. Read the recorded SHA-256 for that RFC from ietf-source.json.
2. Independently fetch https://www.rfc-editor.org/rfc/rfcNNNN.txt (fresh, Watch's own fetch).
3. SHA-256 the fetched bytes; compare to the recorded hash. PASS/FAIL per RFC, reported individually — one mismatch is a finding.
4. If a sampled RFC has no text online: the check inverts — the build's entry must be annotated text-unavailable and ietf-source.json must carry NO hash for it (a hash there would mean fabricated content).

## Leg 2 — hosting (ADJUSTED by Orion 2026-11-18 08:33 CST: hosting is Watch's to STAND UP, not just probe)
- When the build lands AND Orion's coordinator verification passes: Watch hosts ietf.phext via SQ on a ranch node of Watch's pick from the eligible set (GB/s class per the Nov 8/9 I/O sweep — 3.5–5.2 GB/s nodes; exclude the ~300 MB/s class: rock-mountain, scroll-central, quacked-hill).
- Watch = primary host; Rogue (PH: Evolution) holds the mirror-node leg — coordinate so mirror ≠ primary node.
- MIRROR DESIGNATED (Rogue, 2026-11-18 08:41 CST): free-fall (100.74.202.8) — 736 GB free, 22 GiB RAM avail, SQ v0.6.0 at ~/.cargo/bin/sq (invoke by full path; not on non-login PATH), no daemon running, held ready. Primary must NOT be free-fall.
- Primary eligibility (fleet truth, Rogue's sweep via Orion 2026-11-18): GB/s-class measured nodes only; EXCLUDE free-fall (mirror), rock-mountain/scroll-central/quacked-hill (~300 MB/s), jacked-kite/kaleidoscope-one/moon-beam/west-lincoln/yeet-max (login OK direct but lived-in/resident-sensitive, disk class unmeasured — do not place hosting), great-white (genuine key refusal), orchid-flow (NFS + fetch load), aletheia-core (hub/coordinator). Candidates: logos-prime, halycon-vector — BOTH already run `sq host 1337` for an existing workload, so survey ports on the chosen node before standing up; ietf.phext gets its own port.
- Fleet probe hygiene: use FLEET_DIRECT=1 or durable-set IPs; the hub-nested SSH route manufactures phantom key-refusals (five nodes were misrecorded as refused because of it).
- Verify status/TOC/select end-to-end; report the endpoint to Orion for Reed's access doc.
- Deploy/publication: NOT today — publicity hold governs; public endpoint only if Will lifts it for this item. Staged readiness only.

## Leg 3 — hosting exposure probe plan (run when SQ hosting stands up)
1. Bind check on the hosting node itself: `ss -tlnp` for the SQ port — record whether it binds the tailnet IP only or 0.0.0.0.
2. Tailnet reachability: from a second ranch node, fetch the SQ API (status/TOC) over the tailnet address — expect success.
3. Off-tailnet reachability: from awsbox (public internet host, not a tailnet peer for the ranch LAN) and/or this VM's public egress, attempt the same address/port — expect refusal/timeout. Report what was actually probed from where.
4. Auth shape: unauthenticated read attempt (no API key) vs key-bearing read — record whether reads are gated, and whether status/TOC metadata leaks without a key.
5. Leak surface: confirm the SQ instance serves only ietf.phext (no directory listing, no other hosted phexts exposed by the same daemon).

## Reporting
Results go to Orion in the PH: R&D thread with per-RFC pass/fail and probe evidence, and are appended to task 25.6.9/8.6.10/6.6.9 in tasks.phext.
