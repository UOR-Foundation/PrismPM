import assert from 'node:assert/strict';
import test from 'node:test';
import {localModuleInputs} from './local-module-inputs.mjs';

const capture = files => localModuleInputs('/owned', ['tests/a.mjs'], path => {
  assert.ok(Object.hasOwn(files, path), 'missing local module ' + path);
  return Buffer.from(files[path]);
});
test('actual ESM parser captures reexports, multiline imports and cycles without executing source', () => {
  const files = {'tests/a.mjs': 'import\n {x} from "./b.mjs"; export * from "../sdk/c.mjs"; throw Error("must not execute");',
    'tests/b.mjs': 'import "node:fs"; import "./a.mjs"; export const x=1;',
    'sdk/c.mjs': 'export const c=2;'};
  assert.deepEqual([...capture(files).keys()], ['sdk/c.mjs', 'tests/a.mjs', 'tests/b.mjs']);
});
test('closure rejects missing dependencies, escapes, remote/package imports and malformed modules', () => {
  for (const source of ['import "./missing.mjs";', 'import "../../escape.mjs";',
    'import "https://example.invalid/module.mjs";', 'import "package";', 'export const =;',
    'im' + 'port("./dynamic.mjs");', 'im' + 'port/* comment */("./dynamic.mjs");'])
    assert.throws(() => capture({'tests/a.mjs': source}));
});
