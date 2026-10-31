/**
 * What `build --unstable-author` and `check --unstable-author` print. Reports
 * are in the developer's terms: the sentence and the example file, never the
 * theorem name.
 */
const pad = (rows) => Math.max(...rows.map((row) => row.sentence.length));

function proving(rows) {
  const width = pad(rows);
  const lines = ['Proving'];
  for (const row of rows) {
    lines.push(`  ${row.sentence.padEnd(width)}  ${row.verdict}`);
    if (row.why) lines.push(`    why: ${row.why}`);
    if (row.explanation) lines.push(`    agent: ${row.explanation}`);
  }
  return lines;
}

const replayLine = (replay) => `  replay of examples/ waits for #${replay.issue}; the guest replayed the acceptance vectors instead`;

/** A build that passed. */
export function formatReport({ spec, rows, replayed, replay, source, holo }) {
  const lines = [...proving(rows), '', 'Examples', `  ${replayed} of ${spec.examples.length} replayed through the guest`, replayLine(replay), ''];
  const agent = `${source.agent.adapter}/${source.agent.model}`;
  lines.push(source.cached ? `Model: cached from spec blake3:${spec.digest}, no agent call (written by ${agent})` : `Model: written by ${agent} in ${source.attempts} attempt${source.attempts === 1 ? '' : 's'}${source.tokens ? `, ${source.tokens} tokens` : ''}`);
  lines.push(`Built ${holo}`);
  return `${lines.join('\n')}\n`;
}

const count = (n, noun) => `${n} ${noun}${n === 1 ? '' : 's'}`;

/** A build that failed. Exactly one of `falsified`, `unproved`, `agentError` or `gate` says why. */
export function formatFailure({ spec, falsified, unproved, agentError, gate, attempts, tokens, budget }) {
  const lines = [];
  if (falsified) {
    lines.push('Examples contradict intent.md');
    for (const row of falsified) lines.push(`  ${row.sentence}  FALSE`, `    ${row.example}: ${row.reason}`);
    lines.push(`Build failed: ${count(falsified.length, 'intent sentence')} false for the examples (exit 1). The agent was not called.`);
    return `${lines.join('\n')}\n`;
  }
  if (unproved) {
    const byTheorem = new Map(unproved.map((row) => [row.theorem, row]));
    const rows = spec.intent.filter((row) => byTheorem.has(row.theorem)).map((row) => ({ sentence: row.sentence, verdict: 'UNPROVED', explanation: byTheorem.get(row.theorem).explanation }));
    lines.push(...proving(rows), `Build failed: ${count(rows.length, 'intent sentence')} unproved (exit 1)`);
    return `${lines.join('\n')}\n`;
  }
  if (agentError) {
    lines.push(`Build failed: ${agentError} (exit 1)`);
    return `${lines.join('\n')}\n`;
  }
  if (gate?.rows) lines.push(...proving(gate.rows));
  if (gate?.stage === 'examples' && gate.example) lines.push(`Example failed: ${gate.example}`);
  if (gate?.diagnostics && !gate.rows) lines.push(`${gate.stage} failed:`, ...gate.diagnostics.trimEnd().split('\n').slice(-20).map((line) => `  ${line}`));
  const spent = budget ? ` after ${budget}` : '';
  const unprovedRows = gate?.rows?.filter((row) => row.verdict !== 'checked').length ?? 0;
  const what = unprovedRows > 0 ? `${count(unprovedRows, 'intent sentence')} unproved` : `the model did not pass ${gate?.stage ?? 'the gate'}`;
  lines.push(`Build failed: ${what}${spent}${attempts ? ` (${count(attempts, 'attempt')}${tokens ? `, ${tokens} tokens` : ''})` : ''} (exit 1)`);
  return `${lines.join('\n')}\n`;
}

/** The authoring section of `check --unstable-author`. */
export function formatAuthoring({ checks, rows, comparison, replay }) {
  const lines = ['', 'Authoring:'];
  for (const check of checks) lines.push(`  ${check.passed ? 'ok    ' : 'FAILED'}  ${check.id}  ${check.message}`);
  if (rows) lines.push('', ...proving(rows));
  lines.push('', replayLine(replay));
  if (comparison) {
    lines.push('', 'Against the reference:');
    for (const row of comparison) lines.push(`  ${row.same ? 'identical' : 'DIFFERENT'}  ${row.part}${row.same ? '' : `: ${row.why}`}`);
  }
  return `${lines.join('\n')}\n`;
}
