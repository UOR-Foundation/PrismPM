// Every code raised in sdk/npm/prism-code/src and crates/prismpm/src against ERRORS.md and model/errors.toml (#448).
// `xtask validate` gates the Rust side on every change. This also runs when the JavaScript side changes, and holds its unregistered codes to a written list.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { errors } from '../src/errors-namespace.mjs';

const repo = (path) => fileURLToPath(new URL(`../../../../${path}`, import.meta.url));
const PP = /\bPP\d{4}\b/g;

const filesUnder = (dir, extensions) =>
  readdirSync(dir, { withFileTypes: true, recursive: true })
    .filter((entry) => entry.isFile() && extensions.some((extension) => entry.name.endsWith(extension)))
    .map((entry) => join(entry.parentPath, entry.name));

const mentionedPP = (dir, extensions) => {
  const mentioned = new Map();
  for (const file of filesUnder(dir, extensions)) {
    for (const [code] of readFileSync(file, 'utf8').matchAll(PP)) if (!mentioned.has(code)) mentioned.set(code, file);
  }
  return mentioned;
};

const modelCodes = new Set([...readFileSync(repo('model/errors.toml'), 'utf8').matchAll(/^code = "(PP\d{4})"\s*$/gm)].map((match) => match[1]));
const errorsMd = readFileSync(repo('ERRORS.md'), 'utf8');
const documentedPP = new Set([...errorsMd.matchAll(/^## `(PP\d{4})`/gm)].map((match) => match[1]));

/** A code counts as registered once ERRORS.md, generated from model/*.toml, lists it in any namespace. */
const isRegistered = (code) => errorsMd.includes(`\`${code}\``);

const withoutComments = (text) => text.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');

/**
 * The codes the package sets on a `PrismCodeError` or passes as `code`. A code built
 * from a variable, such as `export.${check}`, is listed by its prefix, as `export.*`.
 * src/vendor is the copy of @prism-code/verify, whose only registered codes are its PP codes.
 */
function raisedCodes() {
  const raised = new Set();
  const dir = fileURLToPath(new URL('../src/', import.meta.url));
  for (const file of filesUnder(dir, ['.mjs', '.js'])) {
    if (/[\\/]vendor[\\/]/.test(file)) continue;
    const text = withoutComments(readFileSync(file, 'utf8'));
    const patterns = [/\bfail\(\s*(['"`])([^'"`]+)\1/g, /new PrismCodeError\(\s*(['"`])([^'"`]+)\1/g, /new [A-Z]\w*Error\([^;]{0,300}?\bcode: (')([^']+)'/g];
    for (const pattern of patterns) for (const match of text.matchAll(pattern)) raised.add(match[2].replace(/\$\{[^}]+\}/g, '*'));
  }
  return raised;
}

const sorted = (values) => [...values].sort();

test('ERRORS.md and model/errors.toml list the same PP codes', () => {
  assert.ok(modelCodes.size > 0);
  assert.deepEqual(sorted(documentedPP), sorted(modelCodes));
});

test('every PP code in crates/prismpm/src is registered', () => {
  const mentioned = mentionedPP(repo('crates/prismpm/src'), ['.rs', '.js', '.mjs']);
  assert.ok(mentioned.size > 0);
  for (const [code, file] of mentioned) assert.ok(modelCodes.has(code), `${file} names ${code}, which model/errors.toml does not register`);
});

test('every PP code in sdk/npm/prism-code/src is registered', () => {
  const mentioned = mentionedPP(fileURLToPath(new URL('../src/', import.meta.url)), ['.mjs', '.js']);
  assert.ok(mentioned.size > 0);
  for (const [code, file] of mentioned) assert.ok(modelCodes.has(code), `${file} names ${code}, which model/errors.toml does not register`);
});

test('the unregistered codes of the package are exactly the written list', () => {
  const listed = JSON.parse(readFileSync(new URL('./fixtures/unregistered-codes.json', import.meta.url), 'utf8'));
  const advice =
    'A code the package raises is registered in model/*.toml and listed in ERRORS.md. Until #448 settles where, a new one joins test/fixtures/unregistered-codes.json, and a code that was registered or removed leaves it.';
  assert.deepEqual(sorted(Object.values(errors.codes).filter((code) => !isRegistered(code))), sorted(listed.classCodes), advice);
  assert.deepEqual(sorted([...raisedCodes()].filter((code) => !isRegistered(code))), sorted(listed.raisedCodes), advice);
});
