#!/usr/bin/env node
//
// Checks that src/kitty_ss.rs and src/title_ss.rs's byte arrays still match
// what WASM-4's png2src tool generates from their source PNGs
// (kitty-ss.png and kitty_title.png). Catches a PNG edit that never got run
// back through png2src (or vice versa) -- see the "Assets" section of
// README.md for the full regeneration procedure and its gotchas.
//
// Usage: node tools/verify-assets.js

const { execFileSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const ROOT = path.join(__dirname, "..");

function extractBytes(rustSource) {
  const m = rustSource.match(/\[u8;\s*\d+\]\s*=\s*\[([\s\S]*?)\];/);
  if (!m) throw new Error("couldn't find a `[u8; N] = [...]` array");
  return m[1]
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
    .map((s) => parseInt(s, 16));
}

function png2src(pngPath) {
  const outPath = path.join(
    fs.mkdtempSync(path.join(os.tmpdir(), "verify-assets-")),
    "out.rs"
  );
  execFileSync(
    "npx",
    ["--yes", "-p", "wasm4", "w4", "png2src", "--rust", pngPath, "--output", outPath],
    { cwd: ROOT, stdio: "inherit" }
  );
  const bytes = extractBytes(fs.readFileSync(outPath, "utf8"));
  fs.rmSync(path.dirname(outPath), { recursive: true });
  return bytes;
}

// `renameTo` works around png2src deriving Rust constant names from the
// input filename: title_ss.rs was first generated from a file named
// output_onlinepngtools.png, so that's the name we must feed png2src today
// to reproduce the same bytes (the constant names themselves don't matter
// for this check, only the byte array does).
function check(name, pngRelPath, rustRelPath, renameTo) {
  const pngPath = path.join(ROOT, pngRelPath);
  let sourcePng = pngPath;
  let tmpDir;
  if (renameTo) {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "verify-assets-src-"));
    sourcePng = path.join(tmpDir, renameTo);
    fs.copyFileSync(pngPath, sourcePng);
  }

  const fresh = png2src(sourcePng);
  if (tmpDir) fs.rmSync(tmpDir, { recursive: true });

  const committed = extractBytes(fs.readFileSync(path.join(ROOT, rustRelPath), "utf8"));

  const matches =
    fresh.length === committed.length && fresh.every((b, i) => b === committed[i]);

  if (!matches) {
    console.error(
      `✘ ${name}: ${rustRelPath} does not match what png2src generates from ${pngRelPath}`
    );
    console.error(`  See the "Assets" section of README.md to regenerate it.`);
    return false;
  }
  console.log(`✔ ${name}: ${rustRelPath} matches ${pngRelPath}`);
  return true;
}

const results = [
  check("kitty spritesheet", "kitty-ss.png", "src/kitty_ss.rs"),
  check("title spritesheet", "kitty_title.png", "src/title_ss.rs", "output_onlinepngtools.png"),
];

process.exit(results.every(Boolean) ? 0 : 1);
