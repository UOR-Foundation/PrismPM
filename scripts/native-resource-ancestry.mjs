// Offline commit ancestry only. This is not a source/build acceptance receipt.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {lstatSync, realpathSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {performance} from 'node:perf_hooks';
import {fileURLToPath} from 'node:url';

export const diagnosticInitialAncestor = 'c30276799786c1c9093d4740f41d6376180b8567';
const sourceRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const environment = Object.freeze({PATH: '/usr/bin:/bin', LANG: 'C', LC_ALL: 'C',
  GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_SYSTEM: '/dev/null', GIT_CONFIG_GLOBAL: '/dev/null',
  GIT_NO_REPLACE_OBJECTS: '1', GIT_NO_LAZY_FETCH: '1', GIT_OPTIONAL_LOCKS: '0',
  GIT_TERMINAL_PROMPT: '0', GIT_ALLOW_PROTOCOL: '', GIT_PAGER: 'cat', PAGER: 'cat'});

export function verifyDiagnosticAncestor(revision, repositoryRoot = sourceRoot, budgetMs = 10000) {
  assert.equal(typeof revision, 'string'); assert.match(revision, /^[0-9a-f]{40}$/);
  assert.notEqual(revision, diagnosticInitialAncestor, 'diagnostic source must be a strict descendant');
  assert(Number.isSafeInteger(budgetMs) && budgetMs > 0 && budgetMs <= 10000, 'ancestry budget must narrow the ten-second limit');
  const deadline = performance.now() + budgetMs;
  const root = resolve(repositoryRoot);
  assert.equal(realpathSync(root), root, 'ancestry checkout must not be aliased');
  assert(lstatSync(root).isDirectory(), 'ancestry checkout directory required');
  const git = args => {
    const remaining = Math.floor(deadline - performance.now());
    assert(remaining > 0, 'offline ancestry proof deadline exhausted');
    const output = execFileSync('/usr/bin/git', ['--no-replace-objects', '--no-pager', '--no-optional-locks',
      '-C', root, '-c', 'core.commitGraph=false', '-c', 'protocol.allow=never',
      '-c', 'core.hooksPath=/dev/null', '-c', 'core.fsmonitor=false', '-c', 'gc.auto=0',
      '-c', 'maintenance.auto=false', ...args], {env: environment, encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'], maxBuffer: 65536, timeout: remaining, killSignal: 'SIGKILL'});
    assert(performance.now() < deadline, 'offline ancestry proof deadline exhausted');
    return output;
  };
  const path = output => {
    assert(output.endsWith('\n') && output.length > 1 && !/[\0\r\n]/.test(output.slice(0, -1)), 'one local Git path required');
    return resolve(root, output.slice(0, -1));
  };
  assert.equal(path(git(['rev-parse', '--show-toplevel'])), root, 'exact ancestry checkout root required');
  const common = path(git(['rev-parse', '--git-common-dir']));
  for (const grafts of new Set([join(common, 'info/grafts'), path(git(['rev-parse', '--git-path', 'info/grafts']))])) {
    assert.equal(lstatSync(grafts, {throwIfNoEntry: false}), undefined, 'legacy grafts are forbidden in diagnostic ancestry');
  }
  assert.equal(git(['rev-parse', '--is-shallow-repository']), 'false\n', 'complete offline commit history required');
  assert.equal(git(['rev-parse', '--verify', 'HEAD']), revision + '\n', 'actual checkout HEAD differs from diagnostic source');
  assert.equal(git(['cat-file', '-t', diagnosticInitialAncestor]), 'commit\n', 'pinned diagnostic ancestor must be a commit');
  assert.equal(git(['cat-file', '-t', revision]), 'commit\n', 'diagnostic source must be a commit');
  assert.equal(git(['merge-base', '--is-ancestor', diagnosticInitialAncestor, revision]), '', 'offline strict ancestry required');
  return {before: diagnosticInitialAncestor, after: revision, strict_ancestor: true};
}
