#!/usr/bin/env node
/**
 * prism-conformance: the Prism runtime conformance suite, from the command line.
 *
 *   prism-conformance run --runtime <name> [--entrypoint <arg>]... [--report <file>] [--scenario <id>]... [--keep]
 *                                   run the suite against a runtime through its entrypoint; exit 0 when no
 *                                   scenario failed, errored or passed against its expected-failure declaration
 *   prism-conformance matrix <report>... (--out <file> | --check <file>)
 *                                   render COMPLIANCE.md from the reports, or fail when the committed file is stale
 *   prism-conformance gate [--spec <constants.mjs>]
 *                                   fail unless every holo format revision is named by a scenario that loads it
 *   prism-conformance list          every scenario, expanded over the reference archives
 *
 * Exit codes: 0 ok, 1 the suite, the gate or the stale check failed, 2 the
 * suite's own data or a runtime file is invalid, 64 usage.
 */
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { ConformanceError, expandScenarios, failed, gate, loadFeatures, loadScenarios, referenceArchives, readJsonReport, renderMatrix, current, runSuite, sorted } from '../src/cli.mjs';

const require = createRequire(import.meta.url);
const { version } = require('../package.json');

const USAGE = `prism-conformance ${version}

Usage:
  prism-conformance run --runtime <name> [--entrypoint <arg>]... [--report <file>] [--scenario <id>]... [--keep]
  prism-conformance matrix <report>... (--out <file> | --check <file>)
  prism-conformance gate [--spec <constants.mjs>]
  prism-conformance list
  prism-conformance --version | --help

Options for run:    --runtime <name>       a runtime of runtimes.json: prism-code, prismpm, prism-python
                    --entrypoint <arg>     one element of the command that runs the runtime's shim; repeat for each
                                           element. Replaces the entrypoint runtimes.json registers
                    --report <file>        write the prism/conformance-report/1 there
                    --scenario <id>        run only this scenario, or this expanded run; repeatable
                    --keep                 keep the scratch directory and print its path
Options for matrix: --out <file>           write the rendered matrix there
                    --check <file>         render and compare; exit 1 when the file differs
Options for gate:   --spec <constants.mjs> read FORMAT_REVISIONS from this file instead of @prism-code/spec
`;

function usage(message) {
  if (message) process.stderr.write(`${message}\n\n`);
  process.stderr.write(USAGE);
  return 64;
}

const VALUE_OPTIONS = ['runtime', 'entrypoint', 'report', 'scenario', 'out', 'check', 'spec'];
const REPEATED = ['entrypoint', 'scenario'];

function parse(argv) {
  const positional = [];
  const options = {};
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg.startsWith('--')) {
      positional.push(arg);
      continue;
    }
    const name = arg.slice(2);
    if (VALUE_OPTIONS.includes(name)) {
      const value = argv[i + 1];
      if (value === undefined) return { error: `--${name} takes a value` };
      i += 1;
      if (REPEATED.includes(name)) (options[name] ??= []).push(value);
      else if (Object.hasOwn(options, name)) return { error: `--${name} is given twice` };
      else options[name] = value;
    } else if (name === 'keep') options.keep = true;
    else return { error: `unknown option --${name}` };
  }
  return { positional, options };
}

function describeRow(row) {
  const mark = row.status === 'passed' || row.status === 'xfail' || row.status === 'not-applicable' ? 'ok' : 'not ok';
  const lines = [`${mark} ${row.id} ${row.status}${row.reason ? ` (${row.reason})` : ''}`];
  for (const failure of row.failures ?? []) {
    if (failure.error) {
      lines.push(`    step ${failure.step}: ${failure.error}`);
      if (failure.stderr) lines.push(...failure.stderr.trimEnd().split('\n').map((l) => `      ${l}`));
    } else lines.push(`    step ${failure.step}: ${failure.path}: expected ${failure.expected}, found ${failure.actual}`);
  }
  return `${lines.join('\n')}\n`;
}

function commandRun(positional, options) {
  if (positional.length > 0) return usage(`run takes no positional argument, found ${positional.join(' ')}`);
  if (!options.runtime) return usage('run takes --runtime <name>');
  const scratch = mkdtempSync(join(tmpdir(), 'prism-conformance-'));
  try {
    const report = runSuite({
      runtime: options.runtime,
      entrypoint: options.entrypoint ?? null,
      only: options.scenario ?? null,
      scratch,
      log: (row) => process.stdout.write(describeRow(row)),
    });
    const s = report.summary;
    process.stdout.write(`\n${report.runtime.name}${report.runtime.version ? ` ${report.runtime.version}` : ''}: ${s.passed} passed, ${s.failed} failed, ${s.xfail} expected to fail, ${s['unexpected-pass']} unexpectedly passed, ${s.error} errored, ${s['not-applicable']} not applicable\n`);
    if (options.report) {
      writeFileSync(resolve(options.report), `${JSON.stringify(sorted(report), null, 2)}\n`);
      process.stdout.write(`report written to ${options.report}\n`);
    }
    return failed(report) ? 1 : 0;
  } finally {
    if (options.keep) process.stdout.write(`scratch kept at ${scratch}\n`);
    else rmSync(scratch, { recursive: true, force: true });
  }
}

function commandMatrix(positional, options) {
  if (positional.length === 0) return usage('matrix takes at least one report');
  if ((options.out && options.check) || (!options.out && !options.check)) return usage('matrix takes --out <file> or --check <file>');
  const reports = {};
  for (const path of positional) {
    const report = readJsonReport(resolve(path));
    if (Object.hasOwn(reports, report.runtime.name)) return usage(`two reports name runtime ${report.runtime.name}`);
    reports[report.runtime.name] = report;
  }
  const rendered = renderMatrix({ reports });
  if (options.out) {
    writeFileSync(resolve(options.out), rendered);
    process.stdout.write(`matrix written to ${options.out}\n`);
    return 0;
  }
  let committed;
  try {
    committed = readFileSync(resolve(options.check), 'utf8');
  } catch (cause) {
    process.stderr.write(`${options.check} cannot be read: ${cause.code ?? cause.message}\n`);
    return 1;
  }
  if (current(rendered, committed)) {
    process.stdout.write(`${options.check} is current\n`);
    return 0;
  }
  process.stderr.write(`${options.check} is stale: it differs from the matrix rendered from these reports. Run \`npm run matrix\` in sdk/npm/conformance and commit the result.\n`);
  return 1;
}

async function commandGate(positional, options) {
  if (positional.length > 0) return usage(`gate takes no positional argument, found ${positional.join(' ')}`);
  const { revisions, problems } = await gate({ specPath: options.spec ? resolve(options.spec) : null });
  if (problems.length > 0) {
    process.stderr.write(`${problems.map((p) => `gate: ${p}`).join('\n')}\n`);
    return 1;
  }
  process.stdout.write(`gate: every format revision (${revisions.join(', ')}) is named by a scenario that loads an archive of it\n`);
  return 0;
}

function commandList(positional) {
  if (positional.length > 0) return usage(`list takes no argument, found ${positional.join(' ')}`);
  const features = loadFeatures();
  const runs = expandScenarios(loadScenarios(undefined, { features }), referenceArchives());
  for (const run of runs) process.stdout.write(`${run.id}\t${run.scenario.feature}\t${run.scenario.cites.join(',')}\t${run.scenario.revision}\n`);
  return 0;
}

async function main(argv) {
  const [command, ...rest] = argv;
  if (!command || command === '--help' || command === '-h' || command === 'help') {
    process.stdout.write(USAGE);
    return command ? 0 : 64;
  }
  if (command === '--version' || command === '-v' || command === 'version') {
    process.stdout.write(`${version}\n`);
    return 0;
  }
  const parsed = parse(rest);
  if (parsed.error) return usage(parsed.error);
  const { positional, options } = parsed;
  switch (command) {
    case 'run':
      return commandRun(positional, options);
    case 'matrix':
      return commandMatrix(positional, options);
    case 'gate':
      return commandGate(positional, options);
    case 'list':
      return commandList(positional);
    default:
      return usage(`unknown command ${command}`);
  }
}

main(process.argv.slice(2)).then(
  (code) => {
    process.exitCode = code;
  },
  (error) => {
    if (error instanceof ConformanceError) {
      process.stderr.write(`${error.code}: ${error.message}\n`);
      process.exitCode = 2;
    } else {
      process.stderr.write(`${error?.stack ?? error}\n`);
      process.exitCode = 70;
    }
  },
);
