# _shared — scholar-ops Rulebook

Single source of the rules every mode obeys. When a mode says "per `_shared.md`", this is the definition. Never redefine these values inside a mode.

---

## The UNKNOWN convention (INV-06)

A datum not found in the listing or `config/profile.yml` is written exactly `UNKNOWN` (in reports/blocks) or `unknown` (in the tracker deadline cell). Never blank, never guessed. This applies to deadlines, funding amounts, eligibility criteria, and profile facts alike.

---

## Hard eligibility gates (INV-04, INV-05)

Gates run **before** any scoring. Each gate resolves to one of:

- **PASS** — the listing's requirement is met by a profile fact.
- **FAIL** — the requirement is not met. **Any FAIL → verdict INELIGIBLE, stop before scoring.**
- **UNKNOWN** — evidence ambiguous or the profile datum is missing. Does not stop evaluation; flagged in the report and verdict line with ` ⚠`.

Standard gates (evaluate each the listing states; skip gates the listing does not mention):

| Gate | PASS when |
|------|-----------|
| Nationality / citizenship | one of `identity.nationality` is eligible (not on an exclusion list, on an inclusion list if one exists) |
| Age | derived age at deadline within the listing's limit (`identity.birth_year`) |
| Degree level | `education.degrees` contains the required prior level |
| Field restriction | one of `target.fields` / a held degree field matches the listing's allowed fields |
| GPA minimum | `education.degrees[].gpa` meets the stated minimum (normalize scales) |
| Language certificate | a `languages[]` entry has the required cert + minimum score, `status: held` (or `planned` before deadline → UNKNOWN, not PASS) |
| Residency | `identity.residence_country` satisfies any residency requirement |
| Enrollment status | matches any "must be enrolled / must not yet be enrolled" requirement |
| Other binary criterion | any additional hard requirement the listing states |

**Every gate row quotes the listing's requirement verbatim.** Compare that quote against the named profile fact.

---

## Fit scoring (INV-04 runs only after all gates PASS/UNKNOWN)

Score each dimension 0–5 (0 = fails the intent, 5 = ideal). Weighted composite on a 0–5 scale.

| Dimension | Weight | 5 means | 0 means |
|-----------|-------:|---------|---------|
| Funding coverage | 20 | full tuition + stipend + travel | token amount vs need |
| Eligibility margin | 15 | comfortably clears every gate | scrapes minimums / UNKNOWN gates |
| Field match | 15 | exact target field | tangential to targets |
| Deadline feasibility | 15 | ample time to assemble all docs | deadline imminent, docs missing |
| Selectivity (odds) | 10 | wide intake / favorable odds | single award, thousands apply |
| Application effort | 10 | reuses ready docs | many bespoke essays + tests |
| Career value | 10 | strong program + prestige for target path | weak signal |
| Constraints fit | 5 | matches bond/mode/relocation prefs | conflicts with a hard constraint |

**Composite** = Σ(score × weight) / 100, two decimals, 0.00–5.00.

**Letter grade:** A ≥ 4.5 · B ≥ 4.0 · C ≥ 3.0 · D ≥ 2.0 · F < 2.0.

Every dimension needs a one-line rationale. A score without a rationale is invalid output.

---

## Verdict (BR-01, INV-09)

Assign in this order:

1. Liveness failed (dead/closed/deadline passed) → **DEAD** (no report; tracker row only).
2. Any gate FAIL → **INELIGIBLE** (stub report: Blocks A + B + F only).
3. Deadline already passed → never APPLY; cap at SKIP.
4. Otherwise by composite: **APPLY** ≥ 4.0 · **MAYBE** 3.0–3.9 · **SKIP** < 3.0.

Append ` ⚠` to the verdict for each UNKNOWN gate or legitimacy flag.

Verdict line format:
`{BADGE} {VERDICT} · {score}/5 ({grade}) · deadline {value} ({marker} {n} days) · report: {path}`

Badges & markers are defined in `docs/07_visual_identity.md`: 🟢 APPLY · 🟡 MAYBE · 🔴 SKIP · ⛔ INELIGIBLE · 💀 DEAD; deadline 🔥 <7d · ⚠ <14d · ✗ PASSED · ∞ rolling · ? unknown.

---

## Research budgets (INV-08, BR-04)

Per evaluation: **≤ 3 WebSearch queries** (selectivity/odds only) and **≤ 2 Playwright navigations** (liveness fallback). When exhausted, mark remaining data unavailable — do not keep searching. WebSearch queries MUST NOT contain profile data (INV-12).

---

## Tracker row contract (INV-02) — `data/scholarships.md`

One markdown table, exactly these columns:

```markdown
| Name | Provider | Level | Country | Deadline | Score | Verdict | Status | Report | URL |
|------|----------|-------|---------|----------|-------|---------|--------|--------|-----|
```

| Column | Domain |
|--------|--------|
| Name | free text |
| Provider | free text |
| Level | `bachelor \| masters \| phd \| exchange` (comma-join if multiple) |
| Country | host country, or `various` |
| Deadline | `YYYY-MM-DD \| rolling \| unknown` — required on every row |
| Score | `0.00`–`5.00`, or `—` for INELIGIBLE/DEAD |
| Verdict | `APPLY \| MAYBE \| SKIP \| INELIGIBLE \| DEAD` (+ ` ⚠` flags) |
| Status | closed vocabulary below |
| Report | repo-relative path, or `—` |
| URL | absolute URL — **unique key**; one row per URL |

Rows kept deadline-ascending; `rolling`/`unknown` last. Each evaluation appends or updates exactly one row.

### Status vocabulary (BR-06) — closed set

`found → evaluated → preparing → applied → awaiting → interview → won | lost`, plus `dead`.

Update statuses only on the Seeker's instruction. Never delete a row unless the Seeker names it.

---

## Deadline rules (BR-03)

- Warning threshold: **14 days**. Wherever a deadline is shown, flag `⚠` if < 14 days, `🔥` if < 7 days, `✗ PASSED` if in the past.
- Days-remaining computed from today to the deadline date.
- `rolling` → `∞`, `unknown` → `?`.

---

## Legitimacy signals (INV-10, BR-05) — Block F, always present

| Signal | Flag when |
|--------|-----------|
| Application fee | any fee required to apply → **prominent red flag** (fees never block evaluation but always surface in Block F and the verdict line) |
| Domain officiality | apply/host domain does not match the named provider, or is a generic aggregator posing as the source |
| Guaranteed award | "everyone wins", "guaranteed", pay-to-win language |
| Data harvesting | requests sensitive data (bank details, passport scan) disproportionate to a normal application stage |

---

## Slugs

Report filenames: `reports/{provider-slug}-{name-slug}.md`. Slug = lowercase ASCII, hyphen-separated (`DAAD EPOS` → `daad-epos`).
