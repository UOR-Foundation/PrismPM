/**
 * `prism-code setup [--agent <id>] [--dry-run]` and
 * `prism-code open <holo://build?agent=<id>&q=<sentence>> [--dry-run]` (experimental).
 */
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { formatSteps, hostContext, openLink } from './open.mjs';
import { SKILLS_INDEX, applySetup, fetchSkills, formatSetup, hostSetupContext, planSetup } from './setup.mjs';
import { applyMachine, formatMachine, hostMachineContext, machineFailed, machineReport, planMachine } from './setup-machine.mjs';

const HANDLER = join(dirname(fileURLToPath(import.meta.url)), 'handler.mjs');

function options(argv, { values = [], flags = [] }) {
  const positional = [];
  const named = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const eq = a.indexOf('=');
    const name = a.startsWith('--') ? (eq > 0 ? a.slice(2, eq) : a.slice(2)) : null;
    if (name === null) positional.push(a);
    else if (values.includes(name)) named[name] = eq > 0 ? a.slice(eq + 1) : argv[++i];
    else if (flags.includes(name) && eq < 0) named[name] = true;
    else return { problem: `--${name} is not an option here` };
    if (values.includes(name) && (typeof named[name] !== 'string' || named[name] === '')) return { problem: `--${name} takes a value` };
  }
  return { positional, named };
}

/**
 * `setup [--agent <id>] [--image <ref>] [--no-pull] [--agents-only | --machine-only] [--json] [--dry-run]`.
 * The machine half runs first and is applied first, so the agent half's write of
 * `config.json` sees the Docker record. @returns {Promise<number>} the exit code
 */
export async function runSetup(argv, usage, context = hostSetupContext(), machineContext = hostMachineContext({ env: context.env, platform: context.platform, configDir: context.configDir })) {
  const { problem, positional, named } = options(argv, { values: ['agent', 'image'], flags: ['dry-run', 'no-pull', 'agents-only', 'machine-only', 'json'] });
  if (problem) return usage(problem);
  if (positional.length > 0) return usage('setup takes no arguments');
  const dryRun = named['dry-run'] === true;
  const machine = named['agents-only'] ? null : planMachine({ image: named.image, pull: !named['no-pull'] }, machineContext);
  if (machine && !dryRun) applyMachine(machine);
  const agentsHalf = !named['machine-only'];
  // The skills are read from uor.foundation even with --dry-run: reading changes nothing here.
  const skills = agentsHalf ? await fetchSkills(context.env.PRISM_SKILLS_INDEX || SKILLS_INDEX) : undefined;
  const plan = agentsHalf ? planSetup({ agent: named.agent, skills }, context) : { agents: [], handler: { status: 'not-found', steps: [] }, preferred: { agent: null, steps: [] } };
  if (!dryRun && agentsHalf) applySetup(plan);
  const rows = [...plan.agents, plan.handler, ...(plan.skills ? [plan.skills] : [])];
  const failed = (machine ? machineFailed(machine) : false) || rows.some((row) => row.status === 'failed');
  if (named.json) {
    const row = ({ agent, status, detail }) => ({ agent, status, detail: detail ?? null });
    const report = {
      schema: 'prism/setup-report/1',
      dryRun,
      machine: machine ? machineReport(machine) : null,
      agents: plan.agents.map(row),
      handler: { status: plan.handler.status, detail: plan.handler.detail ?? null },
      skills: plan.skills ? { status: plan.skills.status, detail: plan.skills.detail ?? null } : null,
      preferredAgent: plan.preferred.agent,
      failed,
    };
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    return failed ? 1 : 0;
  }
  if (machine) process.stdout.write(formatMachine(machine, { dryRun }));
  if (agentsHalf) process.stdout.write(formatSetup(plan, { dryRun }));
  return failed ? 1 : 0;
}

/** @returns {Promise<number>} the exit code */
export async function runOpen(argv, usage, context = hostContext({ handler: HANDLER })) {
  const { problem, positional, named } = options(argv, { flags: ['dry-run'] });
  if (problem) return usage(problem);
  if (positional.length !== 1) return usage('open takes one holo://build link');
  // A refused link throws a PrismCodeError open.*, which the command line prints and exits 2 for.
  const plan = await openLink(positional[0], context, { dryRun: named['dry-run'] === true });
  process.stdout.write(named['dry-run'] ? `${formatSteps(plan.steps)}\n` : `${plan.summary}\n`);
  return 0;
}
