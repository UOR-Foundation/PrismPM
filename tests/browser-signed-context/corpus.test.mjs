import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {corpus, context, envelope, publicKey, encode} from './corpus.mjs';
import {mutations, mutateSignedContextSource} from './mutations.mjs';

test('independent signed-context exact wire bounds and complete finite inventory', () => {
  assert.equal(encode(context()).length, 416);
  assert.equal(encode(envelope().slice(0, 3)).length, 485);
  assert.equal(encode(envelope()).length, 551);
  assert.equal(encode([1, 2, context(), publicKey(), envelope()]).length, 1037);
  const rows = corpus();
  assert.equal(rows.filter(row => row.id.startsWith('Truncated')).length, 1037);
  for (const id of ['BindingMismatch5', 'ContextMismatch8', 'ContextMismatch7', 'KeyMismatch',
    'ContextErrorPrecedence', 'SignatureErrorPrecedence', 'FrameMaximum', 'FrameOverflow'])
    assert.ok(rows.some(row => row.id === id), id);
  const names = ['Foundation.Browser.Application.V1.SignedContext', 'Foundation.Browser.Application.V1.SignedContextWire'];
  const original = new Map(names.map(name => [name, readFileSync(new URL('../../stdlib/src/' + name.replaceAll('.', '/') + '.lex.tex', import.meta.url))]));
  const canonical = value => Array.isArray(value) ? value.map(canonical)
    : value && typeof value === 'object'
      ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
  for (const mutation of mutations) {
    const sources = new Map(original); mutateSignedContextSource(sources, mutation.id);
    assert.ok(rows.some(row => row.id === mutation.probe));
    assert.equal([...sources].filter(([name, bytes]) => !bytes.equals(original.get(name))).length, 1);
    for (const bytes of sources.values()) {
      const json = /\\semanticdata\{(.*)\}/.exec(bytes.toString())[1];
      assert.equal(json, JSON.stringify(canonical(JSON.parse(json))), 'canonical actual mutant ' + mutation.id);
    }
  }
});

test('envelope reader binds version separately from context key and signature', () => {
  const canonical = value => Array.isArray(value) ? value.map(canonical)
    : value && typeof value === 'object'
      ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
  for (const name of ['SignedContext', 'SignedContextWire']) {
    const source = readFileSync(new URL('../../stdlib/src/Foundation/Browser/Application/V1/' + name + '.lex.tex', import.meta.url), 'utf8');
    const json = /\\semanticdata\{(.*)\}/.exec(source)[1];
    assert.equal(json, JSON.stringify(canonical(JSON.parse(json))), 'canonical actual semantic source ' + name);
  }
  const text = readFileSync(new URL('../../stdlib/src/Foundation/Browser/Application/V1/SignedContextWire.lex.tex', import.meta.url), 'utf8');
  const model = JSON.parse(/\\semanticdata\{(.*)\}/.exec(text)[1]);
  const reader = model.declarations.find(row => row.name === 'readSignedContextEnvelope');
  const values = [];
  function visit(node) {if (!node || typeof node !== 'object') return;
    if (node.kind === 'record' && node.type.name === 'SignedContextEnvelope') values.push(node);
    for (const value of Object.values(node)) Array.isArray(value) ? value.forEach(visit) : visit(value);
  }
  visit(reader.body); assert.equal(values.length, 1);
  const check = value => assert.deepEqual(value.fields.map(field => [field.field, field.value.field, field.value.value.name]),
    [['context', 'value', 'field1'], ['publicKey', 'value', 'field2'], ['signature', 'value', 'field3']]);
  check(values[0]);
  const shifted = structuredClone(values[0]); shifted.fields[0].value.value.name = 'field0';
  assert.throws(() => check(shifted));
});
