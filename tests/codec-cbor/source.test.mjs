import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {test} from 'node:test';

const root = new URL('../../', import.meta.url);
const pins = new Map([
  [8949, 'f1164a5b31a39350ad46abe29b83575eb933ca6c45366989c118b6b1058a214a'],
  [8610, '3713f2a50e23a2bea0a6147ad6c4433605a00c3d00a868bfa86a42b204997089'],
  [3629, 'a2a3a39457d30420c812f87a38cc2b9194832f6982ef17e619de7d816879d6a6'],
]);

test('exact RFC authority bytes are retained, including their notices', () => {
  for (const [rfc, hash] of pins) {
    const bytes = readFileSync(new URL(`tests/fixtures/codec/cbor/rfc${rfc}.txt`, root));
    assert.equal(createHash('sha256').update(bytes).digest('hex'), hash);
    assert.match(bytes.toString(), /Copyright/);
  }
});

test('primitive implementation is authored LexLean, with typed cursor results', () => {
  const source = readFileSync(new URL(
    'stdlib/src/Foundation/Codec/Cbor/V1/Primitive.lex.tex', root), 'utf8');
  const line = source.split('\n').find(line => line.startsWith('\\semanticdata{'));
  const model = JSON.parse(line.slice(14, -1));
  assert.equal(model.spec, 'lexlean/semantic-module/1');
  const declarations = new Map(model.declarations.map(row => [row.name, row]));
  assert.equal(declarations.size, model.declarations.length);
  for (const name of ['CborLimits', 'CborDecoded', 'CborArrayHead']) {
    assert.equal(declarations.get(name)?.kind, 'structure', name);
  }
  for (const name of ['CborValue', 'CborError']) {
    assert.equal(declarations.get(name)?.kind, 'inductive', name);
  }
  for (const name of ['readCborPrimitive', 'readCborArrayHead',
    'writeCborPrimitive', 'writeCborArrayHead', 'finishCborCursor',
    'canonicalCborPrimitiveBytes']) {
    const declaration = declarations.get(name);
    assert.equal(declaration?.kind, 'definition', name);
    assert.ok(declaration.body);
  }
  assert.match(source, /\\importmodule\{Foundation\.Codec\}/);
  assert.match(source, /\\importmodule\{Foundation\.Bytes\}/);
});
