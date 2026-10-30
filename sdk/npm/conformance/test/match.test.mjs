// The expectation matcher: equality, subset objects, arrays by length, the six operators and the mismatch paths.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { OPERATORS, matches, mismatches, validatePattern } from '../src/match.mjs';

test('a primitive matches by equality, null included', () => {
  assert.ok(matches(1, 1));
  assert.ok(matches('a', 'a'));
  assert.ok(matches(true, true));
  assert.ok(matches(null, null));
  assert.ok(!matches(1, '1'));
  assert.ok(!matches(null, undefined));
  assert.deepEqual(mismatches(1, 2), [{ path: '$', expected: '1', actual: '2' }]);
});

test('an object matches when every member it names matches; members it does not name are free', () => {
  assert.ok(matches({ a: 1 }, { a: 1, b: 2 }));
  assert.ok(!matches({ a: 1, c: 3 }, { a: 1, b: 2 }));
  assert.deepEqual(mismatches({ a: { b: 1 } }, { a: { b: 2 } }), [{ path: '$.a.b', expected: '1', actual: '2' }]);
  assert.deepEqual(mismatches({ a: 1 }, { b: 1 }), [{ path: '$.a', expected: '1', actual: 'absent' }]);
  assert.deepEqual(mismatches({ a: 1 }, 'not an object'), [{ path: '$', expected: 'an object', actual: '"not an object"' }]);
});

test('an array matches element by element with the same length', () => {
  assert.ok(matches([1, { a: 1 }], [1, { a: 1, b: 2 }]));
  assert.ok(!matches([1], [1, 2]));
  assert.ok(!matches([], [1]));
  assert.ok(matches([], []));
  assert.deepEqual(mismatches([1, 2], [1, 3]), [{ path: '$[1]', expected: '2', actual: '3' }]);
  assert.deepEqual(mismatches([1], { 0: 1 }), [{ path: '$', expected: 'an array of 1', actual: '{"0":1}' }]);
});

test('$type distinguishes string, number, boolean, object, array and null', () => {
  const cases = [
    ['string', 'a'],
    ['number', 1],
    ['boolean', false],
    ['object', {}],
    ['array', []],
    ['null', null],
  ];
  for (const [type, value] of cases) {
    assert.ok(matches({ $type: type }, value), type);
    for (const [other, otherValue] of cases) if (other !== type) assert.ok(!matches({ $type: type }, otherValue), `${type} against ${other}`);
  }
});

test('$min and $max bound a number and refuse anything else', () => {
  assert.ok(matches({ $min: 1 }, 1));
  assert.ok(matches({ $min: 1 }, 7));
  assert.ok(!matches({ $min: 1 }, 0));
  assert.ok(!matches({ $min: 1 }, '7'));
  assert.ok(matches({ $max: 8 }, 8));
  assert.ok(!matches({ $max: 8 }, 9));
  assert.ok(matches({ $min: 1, $max: 8 }, 4));
  assert.ok(!matches({ $min: 1, $max: 8 }, 9));
});

test('$regex matches a string against a regular expression', () => {
  assert.ok(matches({ $regex: '^blake3:[0-9a-f]{64}$' }, `blake3:${'ab'.repeat(32)}`));
  assert.ok(!matches({ $regex: '^blake3:' }, 'sha256:x'));
  assert.ok(!matches({ $regex: '^1$' }, 1));
});

test('$length applies a pattern to the length of a string or an array', () => {
  assert.ok(matches({ $length: 3 }, 'abc'));
  assert.ok(matches({ $length: { $min: 1 } }, [1]));
  assert.ok(!matches({ $length: { $min: 1 } }, []));
  assert.ok(!matches({ $length: 1 }, 1));
  assert.deepEqual(mismatches({ $length: 2 }, [1]), [{ path: '$.length', expected: '2', actual: '1' }]);
});

test('$some is true when at least one element of an array matches', () => {
  assert.ok(matches({ $some: { check: 'policy.level' } }, [{ check: 'proof.statement' }, { check: 'policy.level', code: 'PP5601' }]));
  assert.ok(!matches({ $some: { check: 'policy.level' } }, [{ check: 'proof.statement' }]));
  assert.ok(matches({ $some: 'holo_run' }, ['holo_alloc', 'holo_run']));
  assert.ok(!matches({ $some: 'holo_run' }, 'holo_run'));
  assert.ok(!matches({ $some: 1 }, []));
});

test('operators combine in one pattern, and every mismatch is reported with its path', () => {
  assert.ok(matches({ $length: { $min: 1 }, $some: { a: 1 } }, [{ a: 1 }]));
  const found = mismatches({ exit: 0, result: { level: 'P0', reasons: { $length: 1 } } }, { exit: 1, result: { level: 'P1', reasons: [] } });
  assert.deepEqual(
    found.map((m) => m.path),
    ['$.exit', '$.result.level', '$.result.reasons.length'],
  );
});

test('validatePattern refuses an unknown operator, a mixed object and a wrong argument', () => {
  assert.deepEqual(OPERATORS, ['$type', '$min', '$max', '$regex', '$length', '$some']);
  assert.throws(() => validatePattern({ $every: 1 }), /unknown operator \$every/);
  assert.throws(() => validatePattern({ $min: 1, a: 1 }), /mixes operators/);
  assert.throws(() => validatePattern({ $type: 'integer' }), /\$type takes one of/);
  assert.throws(() => validatePattern({ $min: '1' }), /\$min takes a number/);
  assert.throws(() => validatePattern({ $regex: '(' }), /Invalid regular expression/);
  assert.throws(() => validatePattern({ a: [{ $nope: 1 }] }), /\$\.a\[0\]: unknown operator/);
  validatePattern({ a: [{ $type: 'number', $min: 0 }], b: { $length: { $min: 1 }, $some: { c: { $regex: 'x' } } } });
});

test('mismatches against an operator at a nested path name that path', () => {
  assert.deepEqual(mismatches({ a: { b: { $type: 'string' } } }, { a: { b: 1 } }), [{ path: '$.a.b', expected: 'a string', actual: '1' }]);
  assert.deepEqual(mismatches({ a: { $some: 1 } }, { a: 'x' }), [{ path: '$.a', expected: 'an array', actual: '"x"' }]);
});
