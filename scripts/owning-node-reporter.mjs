// Acceptance infrastructure, not an application runtime. Node's automatic
// empty-file wrapper is not a registered test and has no per-file summary.
import {tap} from 'node:test/reporters';

export default async function* owningTap(source) {
  const files = [];
  async function* tracked() {
    for await (const event of source) {
      if (event.type === 'test:summary' && event.data.file !== undefined) {
        if (files.length >= 64) throw Error('owning test file limit');
        const {file, success, counts} = event.data;
        files.push({file, success, tests: counts.tests, passed: counts.passed,
          failed: counts.failed, cancelled: counts.cancelled, skipped: counts.skipped,
          todo: counts.todo, topLevel: counts.topLevel, suites: counts.suites});
      }
      yield event;
    }
  }
  yield* tap(tracked());
  for (const file of files) {
    yield '# prismpm-owning-file ' + JSON.stringify(file) + '\n';
  }
}
