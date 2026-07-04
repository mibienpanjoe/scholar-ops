#!/usr/bin/env node
// doctor.mjs — validate scholar-ops setup. Zero dependencies, no network, no AI.
// Exit 0 if all REQUIRED checks pass; 1 otherwise. (FR-081)

import { readFileSync, existsSync } from "node:fs";

const results = []; // { ok:boolean, required:boolean, msg:string }
const pass = (msg) => results.push({ ok: true, required: true, msg });
const fail = (msg) => results.push({ ok: false, required: true, msg });
const warn = (msg) => results.push({ ok: true, required: false, msg, warn: true });

// --- minimal YAML-lite reader: enough to validate our profile schema ---------
// Handles: nested maps (2-space indent), block sequences (- item), inline flow
// lists [a, b], scalars, quoted strings, comments, blank lines. Not a full YAML.
function parseYaml(text) {
  const lines = text.split(/\r?\n/);
  const root = {};
  // stack of { indent, container }
  const stack = [{ indent: -1, container: root }];

  const scalar = (raw) => {
    let v = raw.trim();
    if (v === "" || v === "~" || v === "null") return null;
    if (v === "true") return true;
    if (v === "false") return false;
    if (/^\[.*\]$/.test(v)) {
      const inner = v.slice(1, -1).trim();
      if (inner === "") return [];
      return inner.split(",").map((s) => scalar(s));
    }
    if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) {
      return v.slice(1, -1);
    }
    if (/^-?\d+$/.test(v)) return parseInt(v, 10);
    return v;
  };

  for (let raw of lines) {
    const noComment = raw.replace(/\s+#.*$/, "").replace(/^#.*$/, "");
    if (noComment.trim() === "") continue;
    const indent = noComment.length - noComment.trimStart().length;
    const content = noComment.trim();

    while (stack.length > 1 && indent <= stack[stack.length - 1].indent) stack.pop();
    const parent = stack[stack.length - 1].container;

    if (content.startsWith("- ")) {
      const item = content.slice(2).trim();
      if (!Array.isArray(parent)) continue; // malformed; skip defensively
      if (/^[\w-]+\s*:/.test(item)) {
        // sequence of maps: "- key: value"
        const obj = {};
        parent.push(obj);
        const colon = item.indexOf(":");
        const k = item.slice(0, colon).trim();
        const val = item.slice(colon + 1).trim();
        if (val === "") {
          const child = {};
          obj[k] = child;
          stack.push({ indent, container: obj }); // subsequent keys belong to obj
        } else {
          obj[k] = scalar(val);
          stack.push({ indent, container: obj });
        }
      } else {
        parent.push(scalar(item));
      }
      continue;
    }

    // "key: value" or "key:"
    const colon = content.indexOf(":");
    if (colon === -1) continue;
    const key = content.slice(0, colon).trim();
    const val = content.slice(colon + 1).trim();
    if (Array.isArray(parent)) continue;
    if (val === "") {
      // container follows; decide list vs map by peeking is hard, default map,
      // but if next non-blank line at deeper indent starts with "-", make a list.
      const child = detectChild(lines, lines.indexOf(raw), indent);
      parent[key] = child;
      stack.push({ indent, container: child });
    } else {
      parent[key] = scalar(val);
    }
  }
  return root;
}

function detectChild(lines, idx, indent) {
  for (let i = idx + 1; i < lines.length; i++) {
    const l = lines[i].replace(/\s+#.*$/, "").replace(/^#.*$/, "");
    if (l.trim() === "") continue;
    const ind = l.length - l.trimStart().length;
    if (ind <= indent) break;
    return l.trim().startsWith("- ") ? [] : {};
  }
  return {};
}

// --- checks ------------------------------------------------------------------
function checkNode() {
  const major = parseInt(process.versions.node.split(".")[0], 10);
  major >= 18 ? pass(`Node ${process.versions.node} (>= 18)`)
              : fail(`Node ${process.versions.node} is below required 18`);
}

const REQUIRED = [
  ["identity.full_name", (p) => p?.identity?.full_name],
  ["identity.nationality (>=1)", (p) => Array.isArray(p?.identity?.nationality) && p.identity.nationality.length >= 1],
  ["identity.birth_year", (p) => Number.isInteger(p?.identity?.birth_year)],
  ["education.degrees (key present)", (p) => p?.education && "degrees" in p.education],
  ["target.levels (>=1)", (p) => Array.isArray(p?.target?.levels) && p.target.levels.length >= 1],
  ["target.fields (>=1)", (p) => Array.isArray(p?.target?.fields) && p.target.fields.length >= 1],
  ["languages (>=1)", (p) => Array.isArray(p?.languages) && p.languages.length >= 1],
  ["finances.funding_need", (p) => !!p?.finances?.funding_need],
  ["documents (5 keys)", (p) => p?.documents &&
    ["passport", "transcripts", "reference_letters", "cv", "motivation_letter_base"].every((k) => k in p.documents)],
];

function checkProfile() {
  if (!existsSync("config/profile.yml")) {
    fail("config/profile.yml missing — run: /scholar-ops onboarding");
    return;
  }
  let profile;
  try {
    profile = parseYaml(readFileSync("config/profile.yml", "utf8"));
  } catch (e) {
    fail(`config/profile.yml did not parse: ${e.message}`);
    return;
  }
  pass("config/profile.yml exists and parsed");
  for (const [label, test] of REQUIRED) {
    let ok = false;
    try { ok = !!test(profile); } catch { ok = false; }
    ok ? pass(`  field ${label}`) : fail(`  missing/invalid: ${label}`);
  }
}

function checkPortals() {
  if (!existsSync("portals.yml")) {
    warn("portals.yml missing — scan mode needs it (copy portals.example.yml)");
    return;
  }
  try {
    parseYaml(readFileSync("portals.yml", "utf8"));
    pass("portals.yml exists and parsed");
  } catch (e) {
    fail(`portals.yml did not parse: ${e.message}`);
  }
}

function checkDirs() {
  for (const d of ["data", "reports"]) {
    existsSync(d) ? pass(`directory ${d}/ exists`) : fail(`directory ${d}/ missing`);
  }
}

function checkGitignore() {
  if (!existsSync(".gitignore")) { warn(".gitignore missing — user data is not protected"); return; }
  const gi = readFileSync(".gitignore", "utf8");
  const need = ["config/profile.yml", "portals.yml", "data/", "reports/"];
  const missing = need.filter((n) => !gi.includes(n));
  missing.length === 0
    ? pass(".gitignore covers the user layer")
    : fail(`.gitignore missing rules for: ${missing.join(", ")}`);
}

// --- run ---------------------------------------------------------------------
checkNode();
checkProfile();
checkPortals();
checkDirs();
checkGitignore();

let failed = 0;
for (const r of results) {
  const mark = r.ok ? (r.warn ? "⚠" : "✓") : "✗";
  console.log(`${mark} ${r.msg}`);
  if (!r.ok) failed++;
}
console.log(failed === 0 ? "\nAll required checks passed." : `\n${failed} required check(s) failed.`);
process.exit(failed === 0 ? 0 : 1);
