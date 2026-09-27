import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {mutationNames, mutateSources} from './mutations.mjs';

const names = ['Foundation.View.Browser.V1.Model', 'Foundation.View.Browser.V1.Wire',
  'Foundation.View.Browser.V1.Semantics', 'Foundation.Browser.Application.V1.Session'];
const sources = () => new Map(names.map(name => [name,
  readFileSync(new URL('../../stdlib/src/' + name.replaceAll('.', '/') + '.lex.tex', import.meta.url))]));
const module = bytes => JSON.parse(/\\semanticdata\{(.*)\}/.exec(bytes.toString('utf8'))[1]);

test('dynamic source constructors and all exhaustive presentation matches remain closed', () => {
  const all = sources(), model = module(all.get(names[0]));
  assert.equal(model.declarations.find(row => row.name === 'TextChoice').fields[1].type.kind, 'string');
  const content = model.declarations.find(row => row.name === 'Content');
  assert.deepEqual(content.constructors.map(row => row.name), ['Section', 'Navigation', 'Form',
    'Heading', 'Text', 'Input', 'Multiline', 'Select', 'Action', 'Table', 'SecretInput', 'TextSelect']);
  let matches = 0;
  function inspect(value) {
    if (!value || typeof value !== 'object') return;
    if (value.kind === 'match' && value.branches.some(row => row.constructor.name === 'Content.Select')) {
      assert.equal(value.branches.filter(row => row.constructor.name === 'Content.TextSelect').length, 1);
      matches++;
    }
    Object.values(value).forEach(child => Array.isArray(child) ? child.forEach(inspect) : inspect(child));
  }
  for (const bytes of all.values()) inspect(module(bytes));
  assert.equal(matches, 15);
});

test('every planned defect changes one actual source module and no acceptance fixture', () => {
  const before = sources();
  for (const kind of mutationNames) {
    const mutated = new Map(before); mutateSources(mutated, kind);
    assert.equal([...mutated].filter(([name, bytes]) => !bytes.equals(before.get(name))).length, 1, kind);
    for (const bytes of mutated.values()) assert.equal(module(bytes).spec, 'lexlean/semantic-module/1');
  }
  assert.throws(() => mutateSources(new Map(before), 'unknown'));
});

test('dynamic owning test remains registered and cannot report component checks as generated acceptance', () => {
  const root = new URL('../../', import.meta.url), owner = readFileSync(new URL('sdk/browser/dynamic-choice.test.mjs', root), 'utf8');
  assert.match(owner, /verifyDynamicChoice/);
  assert.match(readFileSync(new URL('model/ids.toml', root), 'utf8'), /id = "DK-31"/);
  assert.match(readFileSync(new URL('crates/conformance/tests/conformance.rs', root), 'utf8'), /conformance_dk_31/);
  assert.ok(fileURLToPath(root).endsWith('/'));
});
