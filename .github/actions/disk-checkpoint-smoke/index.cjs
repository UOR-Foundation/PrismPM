const fs = require('node:fs');
const path = require('node:path');
const {spawn} = require('node:child_process');
const {performance} = require('node:perf_hooks');
const {pathToFileURL} = require('node:url');

// This action receives the normal Actions runtime environment directly. It
// never prints or exports it, or forwards uploader output to the job log.
(async () => {
  const root = process.env.GITHUB_WORKSPACE;
  const temp = process.env.RUNNER_TEMP;
  if (!root || !temp) throw Error('missing runner context');
  const directory = path.join(temp, 'checkpoint-survival-smoke');
  const uploader = path.join(temp, 'checkpoint-survival-uploader.js');
  fs.mkdirSync(directory, {mode: 0o700});
  const allowed = ['PATH', 'HOME', 'ACTIONS_RUNTIME_TOKEN', 'ACTIONS_RUNTIME_URL', 'ACTIONS_RESULTS_URL',
    'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_REPOSITORY', 'GITHUB_SERVER_URL', 'GITHUB_WORKSPACE', 'RUNNER_TRACKING_ID'];
  const env = Object.fromEntries(allowed.filter(key => typeof process.env[key] === 'string').map(key => [key, process.env[key]]));
  const child = spawn(process.execPath, [path.join(root, 'scripts/ci-disk-checkpoint.mjs'), '_monitor', directory, uploader],
    {cwd: root, env, stdio: 'ignore', detached: true});
  let exited = false;
  const closed = new Promise(resolve => { child.once('error', () => { exited = true; resolve({code: null, signal: null}); });
    child.once('close', (code, signal) => { exited = true; resolve({code, signal}); }); });
  try {
    const deadline = performance.now() + 60_000;
    let status;
    while (performance.now() < deadline && !exited) {
      try { status = JSON.parse(fs.readFileSync(path.join(directory, 'status.json'), 'utf8')); } catch {}
      if (status?.attempts === 1) break;
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    if (status?.attempts !== 1 || status?.lastUpload !== 'uploaded') throw Error('checkpoint upload was not confirmed');
    // Kill the owned producer after the action has confirmed finalization.
    // The next job must retrieve its artifact after this job has ended.
    if (exited || child.exitCode !== null || child.signalCode !== null) throw Error('producer exited before termination');
    if (!process.kill(-child.pid, 'SIGKILL')) throw Error('termination failed');
    const terminated = await closed;
    if (terminated.signal !== 'SIGKILL') throw Error('producer termination not confirmed');
    // Exercise the same pinned uploader with runtime credentials deliberately
    // absent. Its output is discarded and only the closed status is checked.
    const {uploadOnce} = await import(pathToFileURL(path.join(root, 'scripts/ci-disk-checkpoint.mjs')).href);
    const negativePath = path.join(directory, 'negative.json');
    const negativeOutput = path.join(directory, 'negative-output');
    fs.writeFileSync(negativePath, '{}', {mode: 0o600, flag: 'wx'});
    fs.writeFileSync(negativeOutput, '', {mode: 0o600, flag: 'wx'});
    const noRuntime = {...env};
    delete noRuntime.ACTIONS_RUNTIME_TOKEN; delete noRuntime.ACTIONS_RUNTIME_URL; delete noRuntime.ACTIONS_RESULTS_URL;
    const negative = await uploadOnce({entry: uploader, path: negativePath, output: negativeOutput,
      name: `vv-disk-denied-${process.env.GITHUB_RUN_ID}`, env: noRuntime, timeoutMs: 10_000});
    if (negative.status !== 'failed') throw Error('missing-credential probe did not fail closed');
  } finally {
    if (!exited && child.exitCode === null && child.signalCode === null) { try { process.kill(-child.pid, 'SIGKILL'); } catch {} await closed; }
  }
})().catch(() => { process.stderr.write('Checkpoint survival smoke failed.\n'); process.exitCode = 1; });
