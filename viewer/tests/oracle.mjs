// Compares the viewer's WebAssembly module with oxedi.parse.
//
// Usage: node viewer/tests/oracle.mjs EXPECTED_DIR
//
// EXPECTED_DIR holds the JSON files viewer/tests/expected.py writes, one per
// fixture and sample. Each input is parsed again with the module's `parse`
// and compared: for a file oxedi.parse rejects, the thrown message; otherwise
// the chosen spec version, the segment count, each segment's chain of loop
// instances (its node up to the root) against the loops open when the library
// read it, every table cell by cell, and every diagnostic's text, loop path,
// level, segment, element and component; then, with a fixed key, the masked cells and
// diagnostic texts against oxedi.parse(..., mask=). Prints one line per file and stops at
// the first mismatch, naming where it is and both values.

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { parse } from "../pkg/index.js";

const MASK_KEY = new TextEncoder().encode("oxedi-test-key-0");
const ROOT = fileURLToPath(new URL("../../", import.meta.url));

class Mismatch extends Error {}

function fail(file, what, mine, theirs) {
  throw new Mismatch(
    `${file}: ${what}\n  module:     ${JSON.stringify(mine)}\n  oxedi.parse: ${JSON.stringify(theirs)}`,
  );
}

// A loop name as the core writes it in a path: quoted when it is empty or
// holds '.', '/', '#' or whitespace.
function renderKey(name) {
  return name === "" || /[./#\s]/u.test(name) ? JSON.stringify(name) : name;
}

// The loops from the root down to `node`, as [name, ordinal] pairs.
function chain(view, node) {
  const pairs = [];
  for (let at = node; at !== null; at = view.nodes[at].parent) {
    const { loop_name: name, ordinal } = view.nodes[at];
    if (name !== null) pairs.push([name, ordinal]);
  }
  return pairs.reverse();
}

function renderPath(path) {
  return path.map(([name, ordinal]) => `${renderKey(name)}#${ordinal}`).join("/");
}

function check(expected) {
  const file = expected.file;
  const bytes = new Uint8Array(readFileSync(join(ROOT, file)));
  let view;
  try {
    view = JSON.parse(parse(bytes));
  } catch (err) {
    if (expected.error === undefined) fail(file, "the module rejected the file", err.message, "accepted");
    if (err.message !== expected.error) fail(file, "error text", err.message, expected.error);
    return `ok ${file} rejected`;
  }
  if (expected.error !== undefined) fail(file, "the module accepted the file", "accepted", expected.error);
  if (view.spec_version !== expected.spec_version) {
    fail(file, "spec version", view.spec_version, expected.spec_version);
  }
  if (view.segments.length !== expected.segments) {
    fail(file, "segment count", view.segments.length, expected.segments);
  }
  view.segments.forEach((segment, index) => {
    const loops = chain(view, segment.node);
    const theirLoops = expected.segment_paths[index];
    if (JSON.stringify(loops) !== JSON.stringify(theirLoops)) {
      fail(file, `loops of segment ${index} (${segment.id || "empty"})`, loops, theirLoops);
    }
  });

  const mine = new Map(view.tables.map((table) => [table.name, table]));
  const names = [...mine.keys()].sort();
  const theirs = Object.keys(expected.tables).sort();
  if (JSON.stringify(names) !== JSON.stringify(theirs)) fail(file, "table names", names, theirs);
  let cells = 0;
  for (const name of theirs) {
    const table = mine.get(name);
    const columns = expected.tables[name];
    const columnNames = table.columns.map((column) => column.name);
    if (JSON.stringify(columnNames) !== JSON.stringify(Object.keys(columns))) {
      fail(file, `columns of ${name}`, columnNames, Object.keys(columns));
    }
    for (const column of table.columns) {
      const values = columns[column.name];
      if (column.values.length !== values.length || table.rows !== values.length) {
        fail(file, `rows of ${name}.${column.name}`, column.values.length, values.length);
      }
      for (let row = 0; row < values.length; row += 1) {
        if (column.values[row] !== values[row]) {
          fail(file, `table ${name}, column ${column.name}, row ${row}`, column.values[row], values[row]);
        }
        cells += 1;
      }
    }
  }

  const texts = view.diagnostics.map((diagnostic) => diagnostic.text);
  if (texts.length !== expected.diagnostics.length) {
    fail(file, "diagnostic count", texts.length, expected.diagnostics.length);
  }
  texts.forEach((text, at) => {
    if (text !== expected.diagnostics[at]) fail(file, `diagnostic ${at}`, text, expected.diagnostics[at]);
    const path = renderPath(view.diagnostics[at].path);
    if (path !== expected.paths[at]) fail(file, `path of diagnostic ${at}`, path, expected.paths[at]);
    const { level, segment, element, component } = view.diagnostics[at];
    const location = [Number(level.replace("SNIP ", "")), segment, element, component];
    if (JSON.stringify(location) !== JSON.stringify(expected.locations[at])) {
      fail(file, `[level, segment, element, component] of diagnostic ${at}`, location, expected.locations[at]);
    }
  });

  const masked = JSON.parse(parse(bytes, MASK_KEY));
  const maskedMine = new Map(masked.tables.map((table) => [table.name, table]));
  for (const name of theirs) {
    for (const column of maskedMine.get(name).columns) {
      const values = expected.masked_tables[name][column.name];
      if (JSON.stringify(column.values) !== JSON.stringify(values)) {
        fail(file, `masked table ${name}, column ${column.name}`, column.values, values);
      }
    }
  }
  const maskedTexts = masked.diagnostics.map((diagnostic) => diagnostic.text);
  if (JSON.stringify(maskedTexts) !== JSON.stringify(expected.masked_diagnostics)) {
    fail(file, "masked diagnostics", maskedTexts, expected.masked_diagnostics);
  }
  return `ok ${file} segments=${view.segments.length} tables=${theirs.length} cells=${cells} diagnostics=${texts.length}`;
}

const dir = process.argv[2];
if (!dir) {
  console.error("usage: node viewer/tests/oracle.mjs EXPECTED_DIR");
  process.exit(2);
}
const files = readdirSync(dir).filter((name) => name.endsWith(".json")).sort();
if (files.length === 0) {
  console.error(`oracle: no expected files in ${dir}`);
  process.exit(1);
}
try {
  for (const name of files) {
    console.log(check(JSON.parse(readFileSync(join(dir, name), "utf8"))));
  }
} catch (err) {
  if (!(err instanceof Mismatch)) throw err;
  console.error(`oracle: mismatch in ${err.message}`);
  process.exit(1);
}
console.log(`oracle: ${files.length} files match oxedi.parse`);
