// Private construction smoke only; actual Binary/Library owners still execute
// their complete generated-source/runtime-audit and installed SDK acceptance.
import assert from 'node:assert/strict';
import {readSemantic, writeSemantic, toNotation, runtimeAudit} from './semantic-source.mjs';

assert.equal(process.getuid(), 1000, 'unprivileged installed consumer required');
const value = {spec: 'lexlean/semantic-module/2', declarations: []};
const source = '\\begin{semanticmodule}\n\\semanticdata{' + JSON.stringify(value) + '}\n\\end{semanticmodule}\n';
assert.deepEqual(readSemantic(source), value);
assert.deepEqual(readSemantic(writeSemantic(source, value)), value);
assert.deepEqual(readSemantic(toNotation(source)), value);
assert.deepEqual(runtimeAudit([]), []);
console.log(JSON.stringify({scope: 'private-installed-helper-dispatch-only',
  operations: ['read', 'write', 'to-notation', 'runtime-audit'], status: 'passed',
  unclaimed: ['runtime-import-closure-acceptance', 'public-source-interface', 'SDK-qualification']}));
