// Isolate each actual maximum; retain only hashes and numeric observations.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {executeWasm} from './runtime.mjs';
const args = process.argv.slice(2); assert.equal(args.length, 3);
const [wasm, request, response] = args.map(path => readFileSync(path));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
console.log(JSON.stringify({...executeWasm(wasm, [{id: 'actual projection maximum', request, response}]),
  request: digest(request), response: digest(response)}));
