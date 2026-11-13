// Planted defects: a gate counts only when a deliberate defect makes it fail.
// Each case plants one defect in an otherwise valid input and names the refusal
// it must get. A defect that passes is a gate that does not hold.
//
// G2 text: the spec is never altered; text a model or document cannot carry
// as written is refused, naming the character, its code point and its path.
// G7 integrity: lives in test/check.test.mjs and test/verify.test.mjs
// (a tampered holo under a P0 policy) and in sdk/npm/verify/test/policy.test.mjs.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { SpecError, validateSpec } from '../src/journey/spec.mjs';
import { renderModel } from '../src/journey/render.mjs';

const VALID = Object.freeze({
  name: 'door',
  title: 'Door',
  purpose: 'Open the door only for a badge on the list.',
  fields: [{ name: 'badge', kind: 'choice', values: ['guest', 'staff'], means: 'Who is at the door.' }],
  outcomes: ['open', 'closed'],
  meanings: { open: 'The door opens.' },
  rules: [{ when: [{ field: 'badge', op: 'is', value: 'staff' }], then: 'open', why: 'Staff may enter.' }],
  otherwise: 'closed',
  otherwiseWhy: 'Everyone else waits.',
  properties: [{ kind: 'never', when: [{ field: 'badge', op: 'is', value: 'guest' }], then: 'open', why: 'Guests never get in.' }],
  examples: [{ request: 'staff', response: 'open' }],
  limits: ['It does not check the badge itself.'],
});

const plant = (edit) => {
  const spec = structuredClone(VALID);
  edit(spec);
  return spec;
};

const refusedWith = (spec, rule, where, codePoint) => {
  assert.throws(
    () => validateSpec(spec),
    (error) => {
      assert.ok(error instanceof SpecError, String(error));
      assert.equal(error.rule, rule);
      assert.equal(error.where, where);
      if (codePoint) assert.ok(error.message.includes(codePoint), error.message);
      return true;
    },
  );
};

test('the unplanted spec passes, unchanged in every text it carries', () => {
  const spec = validateSpec(VALID);
  for (const key of ['title', 'purpose', 'otherwiseWhy']) assert.equal(spec[key], VALID[key]);
  assert.equal(spec.fields[0].means, VALID.fields[0].means);
  assert.equal(spec.rules[0].why, VALID.rules[0].why);
  assert.equal(spec.properties[0].why, VALID.properties[0].why);
  assert.deepEqual(spec.meanings, VALID.meanings);
  assert.deepEqual(spec.limits, VALID.limits);
  assert.ok(renderModel(spec).includes('"Door"'));
});

const DEFECTS = [
  ['a quote in the title', (s) => (s.title = 'Door "A"'), 'text.character', 'title', 'U+0022'],
  ['a backslash in the purpose', (s) => (s.purpose = 'Open the door\\now.'), 'text.character', 'purpose', 'U+005C'],
  ['a U+0007 control character in the purpose', (s) => (s.purpose = 'Open\u0007 the door.'), 'text.character', 'purpose', 'U+0007'],
  ['a non-ASCII letter in the title', (s) => (s.title = 'Tür'), 'text.character', 'title', 'U+00FC'],
  ['a newline in a reason', (s) => (s.rules[0].why = 'Staff\nmay enter.'), 'text.character', 'rules[0].why', 'U+000A'],
  ['a U+2028 line separator in a meaning', (s) => (s.meanings.open = 'The door opens.'), 'text.character', 'meanings.open', 'U+2028'],
  ['a control character in a field meaning', (s) => (s.fields[0].means = 'Who\u0000 is there.'), 'text.character', 'fields[0].means', 'U+0000'],
  ['text not in NFC form', (s) => (s.properties[0].why = 'Guésts never get in.'), 'text.character', 'properties[0].why', null],
  ['a doubled space', (s) => (s.otherwiseWhy = 'Everyone  else waits.'), 'text.space', 'otherwiseWhy', null],
  ['a trailing space', (s) => (s.limits[0] = 'It does not check the badge itself. '), 'text.space', 'limits[0]', null],
  ['a title over 64 characters', (s) => (s.title = 'D'.repeat(65)), 'text.length', 'title', null],
  ['a purpose over 200 characters', (s) => (s.purpose = 'x'.repeat(201)), 'text.length', 'purpose', null],
  ['a reason over 240 characters', (s) => (s.rules[0].why = 'y'.repeat(241)), 'text.length', 'rules[0].why', null],
  ['a control character in what a property says', (s) => (s.properties[0].says = 'Never\u001b open.'), 'text.character', 'properties[0].says', 'U+001B'],
];

for (const [name, edit, rule, where, codePoint] of DEFECTS) {
  test(`G2 planted defect: ${name} is refused as ${rule} at ${where}`, () => refusedWith(plant(edit), rule, where, codePoint));
}

test('G2 planted defect: render refuses a string it would have to change, even past validation', () => {
  const spec = validateSpec(VALID);
  spec.title = 'Door "A"';
  assert.throws(() => renderModel(spec), /render refuses/);
});
