# Mode: onboarding — Build the Profile

The **only** writer of `config/profile.yml` (INV-01). A conversational interview that produces a valid profile. Read `config/profile.example.yml` for the exact schema you must produce.

## When this runs

- The Seeker invokes `/scholar-ops onboarding`, or
- Any profile-dependent mode found `config/profile.yml` missing or unparseable and routed here (FR-014).

## If a profile already exists (FR-025)

Do **not** overwrite. Offer a field-by-field update. Preserve every field the Seeker does not mention — never drop existing data.

## How to ask — use the picker (AskUserQuestion)

Drive the interview with the **`AskUserQuestion` picker**, not prose walls of numbered questions. For any field whose value comes from a fixed set — an enum, yes/no, or multi-pick — present it as picker options. **Batch up to 4 related selectors into one `AskUserQuestion` call** so each step is a single clean screen the Seeker arrows through. The picker always adds an **Other** free-text choice, so a Seeker can still type an answer you didn't list — use the exact enum values from the schema as the options.

Use plain free-text prompts only for open data that can't be enumerated — names, email, institutions, numbers (GPA, birth year, fee), fields of study, countries. Ask those in short related batches too; never drip one at a time, never dump a giant form.

Cover every required field. Adapt follow-ups to answers. Never invent an answer — a skipped or unknown field becomes a placeholder or `unknown` (INV-06).

## The interview — step by step

1. **Identity** *(free-text batch)* — full name, email, nationality/citizenship(s) (ask explicitly; the most common eligibility wall), country of residence, birth year.
2. **Education** *(free-text + picker)* — for each degree held: field, institution, GPA (+ scale), graduation year (null if in progress). Ask **level** as a picker (`high-school | bachelor | masters | phd`) and **currently enrolled?** as a yes/no picker. No degree yet → ask current enrollment (institution + field of study) + expected graduation.
3. **Target level** *(picker, multi-select)* **— ask explicitly (FR-022). Never assume.** "Which scholarship level(s) are you after?" Options `bachelor | masters | phd | exchange`, multiSelect on. Store in `target.levels` (≥1 required).
4. **Fields of study** *(select-or-type picker, multi-select)* — `AskUserQuestion` caps at 4 options, so don't use a canned list: **seed the options from the degree or current-enrollment field(s) captured in Step 2** plus one broad umbrella (e.g. the parent discipline), multiSelect on, and rely on the picker's **Other** row for anything else. No field captured in Step 2 → ask free-text instead. Store in `target.fields` (≥1 required).
5. **Countries + start** *(select-or-type picker + free-text)* — preferred host countries as a multi-select picker: seed 3 likely hosts (e.g. `Japan | Germany | Sweden`) plus an explicit `Anywhere (no preference)` option, multiSelect on; the **Other** row covers any country not listed. `Anywhere` selected → leave `target.countries` empty (= anywhere). Then free-text: earliest start (YYYY-MM).
6. **Languages** *(free-text, then picker per language)* — first ask free-text: which languages does the Seeker speak? Then for each, one picker call with two selectors: **proficiency** (`native | fluent | intermediate | basic`) and **certificate status** (`held | planned | none`); then free-text certificate name + score (skip if `none`).
7. **Finances + constraints** *(one picker call, 3 selectors)* — **funding need** (`full | partial-ok`), **bond acceptable?** (yes/no), **study mode** (`on-campus | online | either`). Then free-text: max application fee in USD (0 = none), any relocation limits.
8. **Documents** *(picker batch)* — readiness of passport, transcripts, reference_letters, cv, motivation_letter_base, each `ready | in-progress | missing`. Batch 4 selectors per call (2 calls total).
9. **Proof points** *(optional, free-text)* — a few achievements, projects, publications, or leadership roles.

## Writing the profile

- Write `config/profile.yml` conforming exactly to the `profile.example.yml` schema. Use the enums as written (`level`, `proficiency`, `status`, `funding_need`, `study_mode`, document values).
- Required fields must be present: `identity.full_name`, `identity.nationality` (≥1), `identity.birth_year`, `education.degrees` (key present, may be empty), `target.levels` (≥1), `target.fields` (≥1), `languages` (≥1), `finances.funding_need`, all five `documents` keys.
- Anything the Seeker doesn't know → a sensible placeholder or `unknown`, and tell them to fill it later.

## Confirm (FR-024)

Show the resulting `config/profile.yml` and ask the Seeker to confirm or correct it.

## Seed portals (optional)

After the profile is confirmed, offer to set up scan: if `portals.yml` does not exist, offer to create it by copying `portals.example.yml`. Explain what you are seeding and what you are not:

- You seed the **sources** (the sites and tracked portals to sweep) and the query **template** — the *where*.
- You do **not** write query strings. Scan composes each Level-1 query from this profile (`target.levels` × `target.fields`) at run time, so the queries follow the profile automatically. The Seeker only maintains the site list.
- Leave `query.include_nationality` at its default `false` (INV-12 — keeps citizenship out of search logs); mention it as an opt-in.

Do not overwrite an existing `portals.yml`. This is the only file besides `config/profile.yml` onboarding may create, and only on the Seeker's yes.

Then suggest next steps: `/scholar-ops <url>` to evaluate a scholarship, or `/scholar-ops scan` to discover some. Optionally suggest `node doctor.mjs` to verify setup.

Reminder: both `config/profile.yml` and `portals.yml` are gitignored and never leave the machine (INV-12).
