# Fleet Truth + Mirror — Rogue (PH: Evolution), 2026-11-18

Reports composed for the ietf.phext One Day Build (task 25.6.9/8.6.10/6.6.9), under Will's directive that all principals support Orion (PH: R&D) and push their changes to this repo. Both reports were measured on 2026-11-18; nothing here is carried over from earlier records without re-measurement.

---

## Part 1 — Fleet truth (probe window 08:34–08:53 CST, 2026-11-18)

29 ranch hosts are registered on the tailnet. Measured today:

**19 OPERATIONAL · 1 KEY-REFUSED · 9 OFFLINE**

Operational (19): aletheia-core, ashfall-haven, elven-path, free-fall, halycon-vector, jacked-kite, kaleidoscope-one, lighthouse-omen, logos-prime, moon-beam, ni-knight, orchid-flow, plain-view, quacked-hill, rock-mountain, scroll-central, uncanny-valley, west-lincoln, yeet-max.

Key-refused (1): great-white (100.124.146.124) — tailnet online, direct-dial SSH returns Permission denied (publickey,password). This is the only genuine refusal in the fleet.

Offline (9): aurora-continuum, best-willow, chief-of-staff, chrysalis-hub, chyrsalis-hub, delta-wood, phext, splinter, victory-galleon — tailnet LastSeen timestamps byte-identical to the 2026-11-17 census; no change in either direction in this class.

### Correction to the 2026-11-17 census — the hub-nested probe artifact

Last night's census classified six machines as key-refused. Five of them — **jacked-kite, kaleidoscope-one, moon-beam, west-lincoln, yeet-max** — were a measurement artifact, and the census record has been corrected.

Cause: for IPs outside its durable set, `fleet-ssh.sh` routes hub-nested through aletheia-core — and the hub's own SSH key is refused by every ranch peer (standing fact since 2026-11-12). Probes down that route report phantom `Permission denied` regardless of the target's actual state. Dialed direct (FLEET_DIRECT=1), all five accept login: 22–36 day uptimes, near-zero load, 4–5 logged-in users each.

Consequences:
- The fleet is healthier than the 2026-11-17 record said: 19 SSH-usable machines, not 14.
- The fleet key-grant recovery item shrinks from six machines to one (great-white).
- The five flipped nodes are **lived-in and resident-sensitive**, and their disk class is **UNMEASURED** — they were never in the 2026-11-08 I/O sweep (uncanny-valley's sweep row failed with the same artifact; its disk class is also unmeasured). Do not place load on them by default.
- Against Will's count of 25 machines vs 29 registrations: if the chrysalis-hub/chyrsalis-hub pair is one box registered twice (−1) and chief-of-staff + phext are not physical ranch machines (−2), the physical count is 26, leaving a residual of 1 unexplained from remote evidence.

### Fleet constraints for build work

1. **Use FLEET_DIRECT=1** (or durable-set IPs) for any fleet probe or deploy. The default hub-nested route manufactures false key-refusals.
2. **Resident-sensitive five:** jacked-kite, kaleidoscope-one, moon-beam, west-lincoln, yeet-max — operational but lived-in and I/O-unmeasured; not default load targets.
3. **Port 1337 is occupied** by existing `sq host 1337` daemons on halycon-vector and logos-prime, serving workloads that are not ours to disturb. Survey before assuming a free SQ port on any node; ietf.phext hosting gets its own port wherever it lands.
4. **orchid-flow** is the fleet NFS server and is still carrying the Lastbook fetch (108 of 109 files verified; the 387.9 GB `Lastbook-All.zip` still pulling at the source's per-IP cap). Fleet-wide transfers share the uplink with it.
5. Quacked Hill is a resident agent's home (Corus) — probe hygiene only, nothing installed, nothing left running.

---

## Part 2 — Mirror designation and completion

### Designation (measured 2026-11-18 morning)

**Mirror host: free-fall (100.74.202.8)** — designated from the GB/s-class operational set, with exclusions applied: the ~300 MB/s class (rock-mountain, scroll-central, quacked-hill), key-refused great-white, orchid-flow (NFS + Lastbook), and aletheia-core (hub/coordinator).

- Disk: 736 GB available on the root LV (935 GB total, 18% used). I/O class 4.5 GB/s write / 3.2 GB/s read (2026-11-08 sweep).
- RAM: 25 GiB total, 22 GiB available — best headroom of the candidates measured (halycon-vector and logos-prime each had 1.6 TB disk free but only 10 GiB RAM available, plus the port-1337 occupants noted above).
- SQ: binary present at `~/.cargo/bin/sq`, invoked and verified (v0.6.0 usage response; the binary has no `--version`). No install was needed. Note: sq is not on the non-login PATH — invoke as `~/.cargo/bin/sq`. No SQ daemon was running at designation; none has been started by this seat — standing SQ up is Watch's (PH: DevSecOps) call, and primary hosting is a different node by protocol (one node, one role).

### Mirror completion (2026-11-18 09:42 CST)

Destination: **free-fall:/home/wbic16/ietf-mirror/**

| File | Size (B) | sha256 (measured on free-fall) |
|---|---|---|
| ietf.phext | 564,160,499 | `5c576b3737c65d12b24bc1dbb8329639326838c986ebf8882bdbb095541ac045` |
| ietf-source.json | 37,446,757 | `ef3f94b1a1a3d42ea4dfefb5de44f8031620eb7e24e3538909aaed0d67a34555` |
| ietf.index.json | 1,473,813 | `849e5a442c83c9b5c37942f10b5936148e9afadd99253154f63b7c3a764036e7` |

Total mirrored: 603,081,069 B. The ietf.phext hash matches the build's hash exactly; the two JSON hashes match source measurements taken on plain-view before transfer.

Transfer route: direct node-to-node rsync, plain-view (`~/ietf-build/`) → free-fall over tailnet IPs. Peer SSH between the nodes was accepted in both directions, so the transfer never routed through the Muse VM (standing order: this box is never a bulk relay). The fleet-NFS staging fallback was confirmed mounted on both nodes but went unused. No anomalies; no SQ daemon started on free-fall.

— Rogue (PH: Evolution), 2026-11-18
