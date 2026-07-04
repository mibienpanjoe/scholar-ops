# Mode: onboarding — Build the Profile

The **only** writer of `config/profile.yml` (INV-01). A conversational interview that produces a valid profile. Read `config/profile.example.yml` for the exact schema you must produce.

## When this runs

- The Seeker invokes `/scholar-ops onboarding`, or
- Any profile-dependent mode found `config/profile.yml` missing or unparseable and routed here (FR-014).

## If a profile already exists (FR-025)

Do **not** overwrite. Offer a field-by-field update. Preserve every field the Seeker does not mention — never drop existing data.

## The interview

Ask conversationally, a few related questions at a time, not as a rigid form. Cover every required field. Adapt follow-ups to answers (e.g. no degree yet → ask current enrollment + expected graduation).

1. **Identity** — full name, email, nationality/citizenship(s) (ask explicitly; this is the most common eligibility wall), country of residence, birth year.
2. **Education** — each degree held: level, field, institution, GPA (+ scale), graduation year. Currently enrolled?
3. **Target level — ask explicitly (FR-022). Never assume.** "Which level(s) of scholarship are you looking for — bachelor, masters, PhD, or exchange/short programs? You can pick more than one." Store the answer in `target.levels`.
4. **Targets** — fields of study, preferred host countries/regions (empty = anywhere), earliest start (YYYY-MM).
5. **Languages** — for each: proficiency, any certificate (IELTS/TOEFL/DELF…) with score, and whether held / planned / none.
6. **Finances** — need full funding or is partial acceptable? Maximum application fee willing to pay (0 = none).
7. **Constraints** — return-home/service bonds acceptable? On-campus, online, or either? Any relocation limits?
8. **Documents** — readiness of passport, transcripts, reference letters, CV, motivation-letter base: `ready | in-progress | missing` each.
9. **Proof points** (optional) — a few achievements, projects, publications, or leadership roles.

## Writing the profile

- Write `config/profile.yml` conforming exactly to the `profile.example.yml` schema. Use the enums as written (`level`, `proficiency`, `status`, `funding_need`, `study_mode`, document values).
- Required fields must be present: `identity.full_name`, `identity.nationality` (≥1), `identity.birth_year`, `education.degrees` (key present, may be empty), `target.levels` (≥1), `target.fields` (≥1), `languages` (≥1), `finances.funding_need`, all five `documents` keys.
- Anything the Seeker doesn't know → a sensible placeholder or `unknown`, and tell them to fill it later.

## Confirm (FR-024)

Show the resulting `config/profile.yml` and ask the Seeker to confirm or correct it. Then suggest next steps: `/scholar-ops <url>` to evaluate a scholarship, or `/scholar-ops scan` to discover some. Optionally suggest `node doctor.mjs` to verify setup.

Reminder: this file is gitignored and never leaves the machine (INV-12).
