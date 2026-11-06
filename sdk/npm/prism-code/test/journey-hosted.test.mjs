// The journey's proof on the hosted verifier: no Docker, no image. A stand-in server answers; nothing leaves the machine.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { hostedClient } from '../src/hosted.mjs';
import { compile } from '../src/journey/session.mjs';
import { names } from '../src/journey/render.mjs';
import { appDir, readSession, writeApp } from '../src/journey/project.mjs';
import { fromHosted, isWarning, prove, proveWhere } from '../src/journey/engine.mjs';

const ANSWERS = {
  purpose: 'Payment approval: decide whether a payment request is approved.',
  fields: 'role: clerk|manager, amount: number at least 1, limit: number',
  outcomes: 'approved, refused',
  rules: 'when role is manager then approved\nwhen amount is at most limit then approved\notherwise refused',
  properties: 'never when role is clerk and amount > limit then approved\nalways when role is manager then approved',
  examples: 'clerk 250 1000 -> approved\nclerk 1001 1000 -> refused',
};
const PROFILE = { id: 'p1', name: 'prism-app', roots: ['src'], packages: [{ package: 'lexlean.core' }, { package: 'lexlean.std.nat' }, { package: 'prism.arch' }] };
const MODULES = ['Foundation.Bytes', 'Foundation.View.Text.V1.Model', 'PaymentApproval', 'PaymentApprovalProofs'];

function app() {
  const dir = appDir(mkdtempSync(join(tmpdir(), 'prism-journey-hosted-')));
  return { dir, session: { ...readSession(dir), spec: compile({ ...ANSWERS, review: 'yes' }).spec, axioms: {}, status: {} } };
}

/** A stand-in server whose verify answers come from `answers`, one per call. */
function server(answers) {
  const verifies = [];
  let id = 0;
  const fetch = async (_url, { body }) => {
    const request = JSON.parse(body);
    const tool = request.params.name;
    let result;
    if (tool === 'lexlean_profiles') result = { structuredContent: { profiles: [PROFILE] } };
    else if (tool === 'lexlean_draft_write') result = { structuredContent: { draft: `prism-app/${++id}` } };
    else if (tool === 'lexlean_verify') {
      verifies.push(request.params.arguments);
      result = answers.shift();
    }
    return { status: 200, text: async () => JSON.stringify({ jsonrpc: '2.0', id: request.id, result }) };
  };
  return { verifies, client: hostedClient({ url: 'https://lexlean.example.test/', fetch, sleep: async () => {}, retryMs: 0 }) };
}

const verified = (modules = MODULES) => ({ taskId: 't', status: 'completed', result: { content: [{ type: 'text', text: JSON.stringify({ attestation_id: 'c'.repeat(64), modules }) }], structuredContent: { state: 'verified' } } });
const failed = (diagnostics) => ({ taskId: 't', status: 'completed', result: { content: [{ type: 'text', text: JSON.stringify({ command: 'verify', diagnostics }) }], structuredContent: { state: 'failed' } } });
const WARNING = { code: 'LLV7002', severity: 'error', message: 'Lean rejected `PrismPaymentApproval.PaymentApproval` (warning): This simp argument is unused:\n  Nat.beq_eq', primary: { path: 'src/PaymentApproval.lex.tex', line_start: 3 } };

test('where the proof runs: PRISM_CODE_TIER chooses, and a diagnostic of the hosted verifier reads like a local one', () => {
  const { dir, session } = app();
  assert.equal(proveWhere(dir, session, { env: { PRISM_CODE_TIER: 'hosted' } }), 'hosted');
  assert.equal(proveWhere(dir, session, { env: { PRISM_CODE_TIER: 'local' } }), 'local');
  assert.deepEqual(fromHosted({ code: 'LLV7002', severity: 'error', message: 'm', primary: { path: 'src/A.lex.tex', line_start: 77, column_start: 1 } }), { severity: 'error', code: 'LLV7002', message: 'm', span: { file: 'src/A.lex.tex', line: 77, column: 1 } });
  assert.equal(fromHosted({ code: 'X', message: 'm' }).span, null);
  assert.equal(isWarning(fromHosted(WARNING)), true, 'Lean warnings arrive as LLV7002 errors and are not failures');
  assert.equal(isWarning({ code: 'LLV7002', severity: 'error', message: 'Lean rejected `A` (error): unsolved goals' }), false);
});

test('a hosted proof names every module, the proofs module included, and records where it was proved', async () => {
  const { dir, session } = app();
  const { verifies, client } = server([verified()]);
  const lines = [];
  const r = await prove(dir, session, { where: 'hosted', client, log: (l) => lines.push(l) });
  assert.equal(r.ok, true, r.message);
  assert.equal(r.where, 'hosted');
  assert.equal(r.attestation, 'c'.repeat(64));
  assert.match(r.message, /^Lean proved all 2 properties for every request on the hosted verifier, in \d+ s\.$/);
  assert.deepEqual(verifies[0].files.sort(), ['src/Foundation/Bytes.lex.tex', 'src/Foundation/View/Text/V1/Model.lex.tex', 'src/PaymentApproval.lex.tex', 'src/PaymentApprovalProofs.lex.tex']);
  assert.match(lines[0], /hosted verifier https:\/\/lexlean\.example\.test\/, so the SDK image is not needed yet\. The files under prism\/src\/ are sent, unlisted but not private\./);
  assert.equal(session.status.proved.where, 'hosted');
  assert.equal(session.status.proved.record, `.lexlean/hosted/${'c'.repeat(64)}.json`);
  assert.ok(existsSync(join(dir, session.status.proved.record)));
  assert.deepEqual(Object.values(session.status.properties), ['proved', 'proved']);
});

test('planted: a verified answer without the proofs module is no proof', async () => {
  const { dir, session } = app();
  const { client } = server([verified(['Foundation.Bytes', 'PaymentApproval'])]);
  const r = await prove(dir, session, { where: 'hosted', client });
  assert.equal(r.ok, false);
  assert.match(r.message, /not the entrypoint modules Foundation\.View\.Text\.V1\.Model, PaymentApprovalProofs; nothing about this project was proved/);
  assert.equal(session.status.proved, undefined);
});

test('a wrong axiom set is corrected and proved again; a Lean warning beside it does not stop that', async () => {
  const { dir, session } = app();
  const axiom = { code: 'LLV7005', severity: 'error', message: '`PrismPaymentApproval.PaymentApproval.decide` violates its exact axiom policy: observed [propext]', primary: { path: 'src/PaymentApproval.lex.tex', line_start: 10 } };
  const { verifies, client } = server([failed([axiom, WARNING]), verified()]);
  const r = await prove(dir, session, { where: 'hosted', client });
  assert.equal(r.ok, true, r.message);
  assert.equal(verifies.length, 2);
  assert.equal(session.axioms.decide, '[propext]');
});

test('planted: Lean refusing a property marks that property failing, found by the line the verifier names', async () => {
  const { dir, session } = app();
  writeApp(dir, session);
  const n = names(session.spec);
  const id = session.spec.properties[0].id;
  const proofs = readFileSync(join(dir, 'src', `${n.proofs}.lex.tex`), 'utf8').split('\n');
  const line = proofs.findIndex((l, i) => /^theorem\{/.test(l) && (proofs[i + 1] ?? '').includes(n.properties[id].theorem)) + 1;
  assert.ok(line > 0, 'the proofs module holds the theorem');
  const refused = { code: 'LLV7002', severity: 'error', message: `Lean rejected \`${n.prefix}.${n.proofs}\` (error): unsolved goals`, primary: { path: `src/${n.proofs}.lex.tex`, line_start: line } };
  const { client } = server([failed([refused, WARNING])]);
  const r = await prove(dir, session, { where: 'hosted', client, overwrite: true });
  assert.equal(r.ok, false);
  assert.equal(session.status.properties[id], 'failing');
  assert.match(r.message, new RegExp(`${id}`));
  assert.match(readFileSync(join(dir, '.prism-last.log'), 'utf8'), new RegExp(`error\\[LLV7002\\]: .*\\n {2}--> src/${n.proofs}\\.lex\\.tex:${line}:`));
});
