import assert from 'node:assert/strict';

// node:test records a failed subtest without rejecting t.test(). Do not let
// subsequent browser checks consume a missing or unaccepted prerequisite.
export async function prerequisite(t, name, verify) {
  let completed = false, result;
  await t.test(name, async child => {
    result = await verify(child);
    completed = true;
  });
  assert.ok(completed, name + ': prerequisite did not complete');
  return result;
}
