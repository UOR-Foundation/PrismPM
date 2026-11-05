/**
 * The command line of the authoring front door. Experimental: every flag here
 * exists only behind `--unstable-author`, and `build` without it is unchanged.
 */
import { resolve } from 'node:path';
import { canonicalJson } from '../archive.mjs';
import { templateFiles, DEFAULT_TEMPLATE, TEMPLATES } from '../init.mjs';
import { DEFAULT_ADAPTER, processAgent } from './agent.mjs';
import { authorBuild } from './build.mjs';
import { checkAuthoring } from './check.mjs';
import { formatAuthoring } from './report.mjs';
import { dockerToolchain } from './toolchain.mjs';

const BUILD_OPTIONS = ['unstable-author', 'check', 'no-cache', 'author-command', 'author-model', 'author-timeout', 'author-max-tokens', 'max-iterations', 'project'];
export const AUTHOR_CHECK_OPTIONS = ['unstable-author', 'project', 'reference'];
const decoder = new TextDecoder();

/**
 * The project every generated model starts from: the reviewed `init` template,
 * without its `prismpm.toml`, which lives at the project root. The agent is
 * shown the project file and the entry module, and the other paths by name.
 */
export function modelScaffold(template = DEFAULT_TEMPLATE) {
  const files = templateFiles(template);
  files.delete('prismpm.toml');
  const shown = [...files.keys()].filter((path) => path === 'lexlean.toml' || /^src\/[^/]+\.lex\.tex$/.test(path));
  return {
    files,
    context: {
      template,
      description: TEMPLATES[template].description,
      paths: [...files.keys()],
      files: Object.fromEntries(shown.map((path) => [path, decoder.decode(files.get(path))])),
    },
  };
}

function wholeNumber(options, name, fallback, usage) {
  if (options[name] === undefined) return { value: fallback };
  const value = Number(options[name]);
  if (!Number.isInteger(value) || value < 1) return { refused: usage(`--${name} takes a positive whole number`) };
  return { value };
}

/** `build --unstable-author`. Returns an exit code, or `{ delegate: true }` for a project with no spec files. */
export async function runAuthorBuild({ positional, options }, usage) {
  const unknown = Object.keys(options).find((name) => !BUILD_OPTIONS.includes(name));
  if (unknown) return usage(`build --unstable-author has no option --${unknown}`);
  if (positional.length > 0) return usage('build --unstable-author takes no positional argument; use --project <dir>');
  for (const name of ['author-command', 'author-model', 'author-timeout', 'author-max-tokens', 'max-iterations', 'project']) {
    if (Object.hasOwn(options, name) && (typeof options[name] !== 'string' || options[name] === '')) return usage(`--${name} takes a value`);
  }
  const maxIterations = wholeNumber(options, 'max-iterations', 10, usage);
  if (maxIterations.refused !== undefined) return maxIterations.refused;
  const maxTokens = wholeNumber(options, 'author-max-tokens', 2_000_000, usage);
  if (maxTokens.refused !== undefined) return maxTokens.refused;
  const timeout = wholeNumber(options, 'author-timeout', 600, usage);
  if (timeout.refused !== undefined) return timeout.refused;

  const root = resolve(options.project ?? '.');
  const scaffold = modelScaffold();
  const command = options['author-command'];
  const agent = processAgent({ ...(command ? { command, adapter: 'command' } : { adapter: DEFAULT_ADAPTER }), timeoutMs: timeout.value * 1000 });
  const result = await authorBuild({
    root,
    agent,
    toolchain: dockerToolchain(root),
    scaffold: scaffold.files,
    scaffoldContext: scaffold.context,
    model: options['author-model'] ?? null,
    maxIterations: maxIterations.value,
    maxTokens: maxTokens.value,
    noCache: options['no-cache'] === true,
    checkOnly: options.check === true,
  });
  if (result.delegate) {
    process.stderr.write('no intent.md or examples/: building the hand-written model as usual\n');
    return result;
  }
  (result.code === 0 ? process.stdout : process.stderr).write(result.text);
  return result.code;
}

/** The authoring half of `check <holo> --unstable-author`. Returns `{ passed, text, json }`. */
export async function runAuthorCheck(holo, options) {
  const outcome = await checkAuthoring(holo, { root: resolve(options.project ?? '.'), reference: options.reference ?? null });
  return { passed: outcome.passed, text: formatAuthoring(outcome), json: canonicalJson(outcome) };
}
