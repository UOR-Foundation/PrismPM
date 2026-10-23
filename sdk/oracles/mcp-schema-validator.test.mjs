// Executable regression for the MCP 2026-07-28 schema oracle: the pinned
// specification's own examples are accepted, and each planted defect in a
// recorded transcript is rejected with a diagnostic that names it.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import Ajv2020 from 'ajv/dist/2020.js';

const here = fileURLToPath(new URL('.', import.meta.url));
const corpus = process.env.PRISMPM_MCP_SCHEMA_CORPUS
  ?? join(here, '../../standards/oracles/mcp-schema-2026-07-28');
const schemaPath = join(corpus, 'schema.json');
const validator = process.env.PRISMPM_MCP_SCHEMA_VALIDATOR ?? join(here, 'mcp-schema-validator.mjs');
const scratch = mkdtempSync(join(tmpdir(), 'mcp-schema-'));

const example = (definition, file) =>
  JSON.parse(readFileSync(join(corpus, 'examples', definition, file), 'utf8'));
const toolsRequest = { ...example('ListToolsRequest', 'list-tools-request.json'), id: 1 };
const toolsResponse = { ...example('ListToolsResultResponse', 'list-tools-result-response.json'), id: 1 };
const callRequest = { ...example('CallToolRequest', 'call-tool-request.json'), id: 2 };
const callResponse = { ...example('CallToolResultResponse', 'call-tool-result-response.json'), id: 2 };
const cancelled = example('CancelledNotification', 'user-requested-cancellation.json');

const c2s = (message) => ({ direction: 'client-to-server', message });
const s2c = (message) => ({ direction: 'server-to-client', message });
const transcript = (messages, overrides = {}) =>
  ({ version: 'prismpm/mcp-transcript/1', spec: '2026-07-28', messages, ...overrides });

let counter = 0;
function run(document) {
  const path = join(scratch, `transcript-${counter++}.json`);
  writeFileSync(path, typeof document === 'string' ? document : JSON.stringify(document));
  return spawnSync(process.execPath, [validator, schemaPath, path], { encoding: 'utf8' });
}
function rejected(document, needle) {
  const result = run(document);
  assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
  assert.ok(result.stderr.includes(needle), result.stderr);
}

test('every example published with the pinned schema validates against its definition', () => {
  const ajv = new Ajv2020({ allErrors: true, strict: false, validateFormats: false });
  ajv.addSchema(JSON.parse(readFileSync(schemaPath, 'utf8')), 'mcp');
  let count = 0;
  for (const definition of readdirSync(join(corpus, 'examples'))) {
    const validate = ajv.getSchema(`mcp#/$defs/${definition}`);
    assert.ok(validate, `${definition} is a schema definition`);
    for (const file of readdirSync(join(corpus, 'examples', definition))) {
      assert.ok(validate(example(definition, file)), `${definition}/${file}: ${JSON.stringify(validate.errors)}`);
      count += 1;
    }
  }
  assert.ok(count >= 100, `${count} examples`);
});

test('a conforming exchange is accepted and its messages counted', () => {
  const result = run(transcript([
    c2s(toolsRequest), s2c(toolsResponse), c2s(callRequest), s2c(callResponse), c2s(cancelled),
  ]));
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout);
  assert.equal(report.valid, true);
  assert.equal(report.messages, 5);
});

test('a result without the resultType discriminator is rejected at the response definition', () => {
  const { resultType, ...bare } = callResponse.result;
  assert.equal(resultType, 'complete');
  rejected(transcript([c2s(callRequest), s2c({ ...callResponse, result: bare })]), 'CallToolResultResponse');
});

test('a method the specification does not define is rejected', () => {
  rejected(transcript([c2s({ jsonrpc: '2.0', id: 1, method: 'tools/invent', params: {} })]), 'tools/invent');
});

test('a request without the protocol-version metadata is rejected', () => {
  const params = { ...toolsRequest.params, _meta: {} };
  rejected(transcript([c2s({ ...toolsRequest, params })]), 'ListToolsRequest');
});

test('a response with no open request, a server request and a stale declaration are rejected', () => {
  rejected(transcript([s2c(toolsResponse)]), 'answers no open');
  rejected(transcript([s2c({ jsonrpc: '2.0', id: 9, method: 'roots/list' })]), 'server-to-client JSON-RPC request');
  rejected(transcript([c2s(toolsRequest)], { spec: '2025-11-25' }), '2026-07-28');
});

test('a malformed or unusable input is refused without reaching the schema', () => {
  rejected('not json', 'not JSON');
  rejected(transcript([]), 'between 1 and');
  const missing = spawnSync(process.execPath, [validator, join(scratch, 'absent.json'), join(scratch, 'transcript-0.json')], { encoding: 'utf8' });
  assert.equal(missing.status, 2);
});
