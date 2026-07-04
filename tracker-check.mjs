#!/usr/bin/env node
// tracker-check.mjs — tracker hygiene: duplicate URLs, bad statuses, malformed rows.
// Zero dependencies, no network, no AI. (FR-083)
// --fix normalizes status spelling/case only; never deletes or reorders rows.
// Exit: 0 clean · 1 findings reported.

import { readFileSync, writeFileSync, existsSync } from "node:fs";

const TRACKER = "data/scholarships.md";
const FIX = process.argv.includes("--fix");

const STATUSES = ["found", "evaluated", "preparing", "applied", "awaiting", "interview", "won", "lost", "dead"];
const COLUMNS = 10; // Name|Provider|Level|Country|Deadline|Score|Verdict|Status|Report|URL

if (!existsSync(TRACKER)) {
  console.error(`✗ ${TRACKER} not found.`);
  process.exit(1);
}

const raw = readFileSync(TRACKER, "utf8");
const lines = raw.split(/\r?\n/);
const findings = [];
const cells = (l) => l.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map((c) => c.trim());

// locate table rows (skip header + separator)
let header = null, statusCol = -1, urlCol = -1;
const seenUrls = new Map();
let fixes = 0;

for (let i = 0; i < lines.length; i++) {
  const line = lines[i];
  if (!line.trim().startsWith("|")) continue;
  if (/^\s*\|[-\s|:]+\|?\s*$/.test(line)) continue; // separator
  const c = cells(line);
  if (!header) {
    header = c.map((h) => h.toLowerCase());
    statusCol = header.indexOf("status");
    urlCol = header.indexOf("url");
    if (c.length !== COLUMNS) findings.push(`header has ${c.length} columns, expected ${COLUMNS}`);
    continue;
  }
  // data row
  if (c.length !== COLUMNS) {
    findings.push(`row ${i + 1}: ${c.length} columns, expected ${COLUMNS} → "${(c[0] || "").slice(0, 30)}"`);
    continue;
  }
  // duplicate URL (INV-02 unique key)
  if (urlCol !== -1) {
    const url = c[urlCol];
    if (url && url !== "—") {
      if (seenUrls.has(url)) findings.push(`row ${i + 1}: duplicate URL ${url} (also row ${seenUrls.get(url)})`);
      else seenUrls.set(url, i + 1);
    }
  }
  // status vocabulary
  if (statusCol !== -1) {
    const s = c[statusCol];
    const norm = s.toLowerCase().trim();
    if (!STATUSES.includes(norm)) {
      findings.push(`row ${i + 1}: status "${s}" not in vocabulary (${STATUSES.join(", ")})`);
    } else if (s !== norm && FIX) {
      c[statusCol] = norm;
      lines[i] = "| " + c.join(" | ") + " |";
      fixes++;
    }
  }
}

if (!header) {
  console.log("No table found — tracker is empty. Nothing to check.");
  process.exit(0);
}

if (FIX && fixes > 0) {
  writeFileSync(TRACKER, lines.join("\n"));
  console.log(`✓ normalized ${fixes} status value(s).`);
}

if (findings.length === 0) {
  console.log(`✓ tracker clean — ${seenUrls.size} unique rows, statuses valid.`);
  process.exit(0);
}

for (const f of findings) console.log(`✗ ${f}`);
console.log(`\n${findings.length} finding(s).${FIX ? "" : "  Re-run with --fix to normalize status spelling."}`);
process.exit(1);
