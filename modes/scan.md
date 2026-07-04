# Mode: scan — Portal Scanner (Discovery)

Bounded discovery of new candidate scholarships. Sweeps the sources in `portals.yml`, filters against the profile, and feeds survivors into the inbox `data/pipeline.md`. It **never evaluates** (INV-14) — evaluation is a separate, Seeker-initiated run.

Read `_shared.md` and `config/profile.yml` first.

## Boundaries (INV-13)

`portals.yml` is the **complete world**. Scan only its `sources`, `extra_queries`, and `tracked_portals`. This is a fixed sweep, not open-ended research. Do not follow the discovery into an evaluation, and do not spawn sub-agents in MVP.

## Two levels

### Level 1 — WebSearch (cheap)

Queries are **composed from the profile**, not hand-written. You do not read query strings out of `portals.yml` — you build them from `config/profile.yml` crossed with the `sources` list, using the `query.template`.

**Compose:**
1. Take `query.template` (default `"{funding} {level} scholarship {field} {year} site:{site}"`).
2. Expand the placeholders:
   - `{level}` → each of `target.levels`.
   - `{field}` → each of `target.fields`.
   - `{funding}` → `"fully funded"` when `finances.funding_need` is `full`, else empty.
   - `{year}` → the current year and the next (from today's date).
   - `{site}` → each `sources[].site`. When the site is empty, drop the trailing `site:` token entirely (open web search).
3. Produce the cross-product of levels × fields × sources, dedupe identical strings, and **cap at `query.max_queries`** (default 8). When capping, prefer covering each field at least once before repeating a field.
4. Append every string in `extra_queries` verbatim (these are run as-is and do **not** count toward, but are bounded by common sense alongside, `max_queries`).

**INV-12 — the query firewall.** A composed query may contain **only** level, field, funding, region, and year terms. Never a name, birth year, GPA, email, or any other personal datum. `identity.nationality` is opt-in: include a region term derived from it **only if** `query.include_nationality` is `true`; default is off, keeping citizenship out of search-engine logs.

Run each query. Collect candidate scholarship links from the results.

### Level 2 — Playwright (tracked portals)
For each `enabled: true` entry in `tracked_portals`, navigate the URL and read listed scholarships. Skip portals already well covered by Level 1 results. If a portal is unreachable, skip it and log an error line — do not retry endlessly (EXC-07).

## Filter (FR-063)

Before a candidate reaches the inbox, keep it only if **all** hold, judged from the listing snippet/page:
- **Level** intersects `target.levels`.
- **Field** is relevant to `target.fields`.
- **Nationality** is not obviously excluded for `identity.nationality`.
- **Deadline** is not already passed.
- Does not contain any `filters.exclude_keywords`.

Respect `filters.max_results_per_source`. When a datum (e.g. deadline) is not visible in the snippet, keep the candidate — the full check happens at evaluation.

## Deduplicate + append (INV-15)

For each survivor, add to `data/pipeline.md` **only if** the URL is absent from both the pipeline and the tracker (`data/scholarships.md`) URL column. Skip duplicates silently, counting them (EXC-09).

Entry format:
```markdown
- [ ] <url> | <portal name> scan YYYY-MM-DD | deadline <value if known>
```

## Log (FR-064)

Append one line per portal to `data/scan-history.tsv` (create with a header if absent):
```
timestamp	portal	found	added	duplicates	errors
```

## Summary + handoff

Report per portal: found / added / duplicates / errors, and the total newly queued. Then stop (INV-14). Tell the Seeker to run `/scholar-ops pipeline` to evaluate the new entries. **Do not evaluate them now.**
