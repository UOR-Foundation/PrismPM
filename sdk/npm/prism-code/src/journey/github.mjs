/**
 * What a journey app needs to build on GitHub Actions with no Docker here.
 *
 * The template's `build.yml` builds an app in `prism/` when the commit holds
 * `prism/prism.toml`, `prism/prismpm.lock` and `prism/standards.lock`, and it
 * reads the SDK image from `prismpm.lock` at the repository root. Pinning the
 * app to an image (`bind`) used to need the image itself, but both locks are
 * a function of the image alone: every app pinned to the default image holds
 * the same bytes. So they are fetched from the template at a pinned commit and
 * checked against pinned SHA-256 digests, beside the workflow, rather than
 * shipped in the package (about 330 KB).
 *
 * The repository is created only when the caller names it, after the user has
 * approved it: creating a public repository is outward facing.
 */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { parseRepository, tokenSource } from '../github.mjs';
import { isBound } from './project.mjs';
import { DEFAULT_SDK_IMAGE } from './sdk.mjs';

export const TEMPLATE_REPOSITORY = 'UOR-Foundation/template';
/** The template commit the files below are read from. Its prismpm.lock pins DEFAULT_SDK_IMAGE. */
export const TEMPLATE_COMMIT = 'df1973257ecd10dbdedd6f00061ac34ca6ac129b';
/** Each file a GitHub build needs, by its path in the template, with the SHA-256 it must have. */
export const PINNED_FILES = Object.freeze({
  'prismpm.lock': '3773d32926962ef5d2556b74e0df9b34bdaf72353022f3256ebaa04a4ba8db7f',
  'standards.lock': '54448d21cb1ed3a4af826aa44991e9839b51d97b8975a5b125927c78af922b46',
  '.github/workflows/build.yml': '756521d5db16c8ea4457a8acdce81d8e656aa602b6f658350f6f303c90de7e00',
});
export const RAW_URL = 'https://raw.githubusercontent.com';

const sha256Hex = (bytes) => createHash('sha256').update(bytes).digest('hex');
const git = (cwd, args, run) => run('git', ['-C', cwd, ...args], { encoding: 'utf8', windowsHide: true, timeout: 120000 });
const ok = (result) => !result.error && result.status === 0;
const said = (result) => String(result.stderr || result.stdout || result.error?.message || '').trim().split('\n').slice(-3).join(' ');

/** The bytes of a pinned template file, refused unless they have the pinned digest. */
export async function fetchPinned(path, { fetch = globalThis.fetch } = {}) {
  const want = PINNED_FILES[path];
  if (!want) throw new Error(`${path} is not a pinned template file`);
  const url = `${RAW_URL}/${TEMPLATE_REPOSITORY}/${TEMPLATE_COMMIT}/${path}`;
  let response;
  try {
    response = await fetch(url);
  } catch (error) {
    throw new Error(`${url} did not answer: ${error.cause?.code ?? error.message}`);
  }
  if (!response.ok) throw new Error(`${url} answered HTTP ${response.status}`);
  const bytes = new Uint8Array(await response.arrayBuffer());
  const got = sha256Hex(bytes);
  if (got !== want) throw new Error(`${url} has SHA-256 ${got}, not the pinned ${want}`);
  return bytes;
}

/** The top of the git repository holding `dir`, or null. */
export function gitRoot(dir, { run = spawnSync } = {}) {
  const top = git(dir, ['rev-parse', '--show-toplevel'], run);
  return ok(top) ? resolve(String(top.stdout).trim()) : null;
}

/** The github.com repository `origin` names in `root`, or null. */
export function originRepository(root, { run = spawnSync } = {}) {
  const url = git(root, ['remote', 'get-url', 'origin'], run);
  return ok(url) ? parseRepository(String(url.stdout).trim()) : null;
}

/** The signed-in GitHub login, from gh, or null. */
export function signedInLogin({ run = spawnSync } = {}) {
  const result = run('gh', ['api', 'user', '--jq', '.login'], { encoding: 'utf8', windowsHide: true, timeout: 30000 });
  const login = ok(result) ? String(result.stdout).trim() : '';
  return /^[A-Za-z0-9-]+$/.test(login) ? login : null;
}

/** Whether a build can go to GitHub Actions: a token is there to start and read it. */
export const canBuildOnGitHub = ({ env = process.env, run = spawnSync } = {}) => tokenSource(env, run) !== null;

/** The repository to suggest for an app: `<login>/prism-<name>`. */
export const suggestedRepository = (login, spec) => `${login ?? 'OWNER'}/prism-${spec.name}`;

/**
 * Write what the build needs and is missing: the app's two locks when it is
 * not pinned yet, `prismpm.lock` at the root, which the workflow reads, and
 * the workflow. Never overwrites. Returns the paths written, relative to `root`.
 */
export async function readyForGitHub({ root, dir, session, fetch = globalThis.fetch, log = () => {} }) {
  const wanted = [];
  if (!isBound(dir)) {
    const image = session.image ?? DEFAULT_SDK_IMAGE;
    if (image !== DEFAULT_SDK_IMAGE) throw new Error(`the app is to be pinned to ${image}, and only ${DEFAULT_SDK_IMAGE} can be pinned without Docker`);
    for (const file of ['standards.lock', 'prismpm.lock']) if (!existsSync(join(dir, file))) wanted.push([file, join(dir, file)]);
  }
  if (!existsSync(join(root, 'prismpm.lock'))) wanted.push(['prismpm.lock', join(root, 'prismpm.lock')]);
  if (!existsSync(join(root, '.github', 'workflows', 'build.yml'))) wanted.push(['.github/workflows/build.yml', join(root, '.github', 'workflows', 'build.yml')]);
  if (wanted.length === 0) return [];
  log(`Adding what the GitHub build needs, from ${TEMPLATE_REPOSITORY} at ${TEMPLATE_COMMIT.slice(0, 12)}, each file checked against its pinned SHA-256...`);
  const cache = new Map();
  const written = [];
  for (const [source, target] of wanted) {
    if (!cache.has(source)) cache.set(source, await fetchPinned(source, { fetch }));
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, cache.get(source));
    written.push(relative(root, target).split('\\').join('/'));
  }
  return written;
}

/**
 * Commit Prism's own paths, and nothing else, in `root`. A new repository is
 * created first. Returns `{ committed, sha }`; throws with git's reason.
 */
export function commitApp({ root, paths, message, run = spawnSync }) {
  if (!existsSync(join(root, '.git')) && gitRoot(root, { run }) === null) {
    const init = git(root, ['init', '-b', 'main'], run);
    if (!ok(init)) throw new Error(`git init failed: ${said(init)}`);
  }
  const add = git(root, ['add', '--', ...paths], run);
  if (!ok(add)) throw new Error(`git add failed: ${said(add)}`);
  const staged = git(root, ['diff', '--cached', '--quiet', '--', ...paths], run);
  let committed = false;
  if (staged.status === 1) {
    const commit = git(root, ['commit', '--quiet', '-m', message, '--', ...paths], run);
    if (!ok(commit)) {
      const hint = /user\.(name|email)|identity/i.test(said(commit)) ? ' Set your name and email with git config --global user.name and user.email, then build again.' : '';
      throw new Error(`git commit failed: ${said(commit)}.${hint}`);
    }
    committed = true;
  }
  const head = git(root, ['rev-parse', 'HEAD'], run);
  if (!ok(head)) throw new Error(`git rev-parse HEAD failed: ${said(head)}`);
  return { committed, sha: String(head.stdout).trim() };
}

/**
 * Put `root` on GitHub as `repository`: create it public with gh and push, or,
 * when it exists already and is empty of this history, add it as origin and push.
 */
export function publishRepository({ root, repository, run = spawnSync }) {
  const repo = parseRepository(repository);
  if (!repo) throw new Error(`${JSON.stringify(repository)} is not owner/name`);
  const view = run('gh', ['repo', 'view', repo.full, '--json', 'name'], { encoding: 'utf8', windowsHide: true, timeout: 60000 });
  if (view.error?.code === 'ENOENT') throw new Error('gh is not on the PATH; install the GitHub CLI (https://cli.github.com) and run gh auth login');
  if (ok(view)) {
    const add = git(root, ['remote', 'add', 'origin', `https://github.com/${repo.full}.git`], run);
    if (!ok(add)) throw new Error(`git remote add origin failed: ${said(add)}`);
    const push = git(root, ['push', '--quiet', '-u', 'origin', 'HEAD:main'], run);
    if (!ok(push)) throw new Error(`git push to ${repo.full} failed: ${said(push)}`);
    return { repository: repo.full, created: false };
  }
  const create = run('gh', ['repo', 'create', repo.full, '--public', '--source', root, '--remote', 'origin', '--push'], { encoding: 'utf8', windowsHide: true, timeout: 180000 });
  if (!ok(create)) throw new Error(`gh repo create ${repo.full} failed: ${said(create)}`);
  return { repository: repo.full, created: true };
}
