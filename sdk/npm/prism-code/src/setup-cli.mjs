/**
 * `prism-code setup [--agent <id>] [--dry-run]` and
 * `prism-code open <holo://build?agent=<id>&q=<sentence>> [--dry-run]` (experimental).
 */
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { formatSteps, hostContext, openLink } from './open.mjs';
import { applySetup, bundledSkills, formatSetup, hostSetupContext, planSetup } from './setup.mjs';

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

/** @returns {Promise<number>} the exit code */
export async function runSetup(argv, usage, context = hostSetupContext()) {
  const { problem, positional, named } = options(argv, { values: ['agent'], flags: ['dry-run'] });
  if (problem) return usage(problem);
  if (positional.length > 0) return usage('setup takes no arguments');
  const plan = planSetup({ agent: named.agent, skills: bundledSkills() }, context);
  const dryRun = named['dry-run'] === true;
  if (!dryRun) applySetup(plan);
  process.stdout.write(formatSetup(plan, { dryRun }));
  return [...plan.agents, plan.handler, plan.skills].some((row) => row.status === 'failed') ? 1 : 0;
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
