#!/usr/bin/env node
// deadline-check.mjs — list tracker deadlines sorted, warn on near/passed dates.
// Zero dependencies, no network, no AI. (FR-082)
// Exit: 0 ok · 1 tracker missing · 2 >=1 unparseable deadline cell.

import { readFileSync, existsSync } from "node:fs";

const TRACKER = "data/scholarships.md";
const argv = process.argv.slice(2);
const daysFlagIdx = argv.indexOf("--days");
const WARN_DAYS = daysFlagIdx !== -1 ? parseInt(argv[daysFlagIdx + 1], 10) : 14;

if (!existsSync(TRACKER)) {
  console.error(`✗ ${TRACKER} not found. Evaluate a scholarship first: /scholar-ops <url>`);
  process.exit(1);
}

// --- parse the one markdown table into row objects (tolerant of padding) -----
function parseTable(text) {
  const lines = text.split(/\r?\n/).filter((l) => l.trim().startsWith("|"));
  if (lines.length < 2) return [];
  const cells = (l) => l.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map((c) => c.trim());
  const header = cells(lines[0]).map((h) => h.toLowerCase());
  const rows = [];
  for (let i = 1; i < lines.length; i++) {
    if (/^[-\s|:]+$/.test(lines[i])) continue; // separator row
    const c = cells(lines[i]);
    const row = {};
    header.forEach((h, j) => (row[h] = c[j] ?? ""));
    rows.push(row);
  }
  return rows;
}

const rows = parseTable(readFileSync(TRACKER, "utf8"));
const today = new Date();
today.setHours(0, 0, 0, 0);

const DAY = 86400000;
let unparseable = 0;
const dated = [];
const special = []; // rolling / unknown

for (const r of rows) {
  const d = (r.deadline ?? "").trim();
  const name = r.name || "(unnamed)";
  const status = r.status || "";
  if (d === "rolling" || d === "unknown") {
    special.push({ marker: d === "rolling" ? "∞" : "?", d, name, status, days: Infinity });
    continue;
  }
  if (!/^\d{4}-\d{2}-\d{2}$/.test(d)) {
    console.error(`✗ unparseable deadline "${d}" for ${name}`);
    unparseable++;
    continue;
  }
  const date = new Date(d + "T00:00:00");
  const days = Math.round((date - today) / DAY);
  dated.push({ d, name, status, days });
}

dated.sort((a, b) => a.days - b.days);

const markerFor = (days) => {
  if (days < 0) return "✗ PASSED";
  if (days < 7) return "🔥";
  if (days < WARN_DAYS) return "⚠";
  return "  ";
};

console.log(`Deadlines (warn < ${WARN_DAYS} days) — ${dated.length + special.length} tracked\n`);
for (const r of dated) {
  const dd = r.days < 0 ? `${r.days}d` : `${r.days}d left`;
  console.log(`${markerFor(r.days).padEnd(8)} ${r.d}  ${String(dd).padEnd(9)} ${r.name}  [${r.status}]`);
}
for (const r of special) {
  console.log(`${r.marker.padEnd(8)} ${r.d.padEnd(12)} ${"".padEnd(9)} ${r.name}  [${r.status}]`);
}

const urgent = dated.filter((r) => r.days >= 0 && r.days < WARN_DAYS).length;
const passed = dated.filter((r) => r.days < 0).length;
console.log(`\n${urgent} within ${WARN_DAYS} days · ${passed} passed`);

process.exit(unparseable > 0 ? 2 : 0);
