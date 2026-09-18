import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { hostname } from 'node:os';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));
const helper = '/usr/local/bin/prismpm-devcontainer-exec';
const initializer = '/usr/local/bin/prismpm-devcontainer-init';
const docker = (args, options = {}) => execFileSync('docker', args, {
  encoding: 'utf8', timeout: 45_000, stdio: ['pipe', 'pipe', 'pipe'], ...options,
}).trim();
const run = (container, user, args) => docker(['exec', '--user', user, container, ...args]);
const denied = (container, user, args, pattern) => {
  assert.throws(() => run(container, user, args), error => {
    assert.equal(error.status, 1);
    assert.match(error.stderr.toString(), pattern);
    return true;
  });
};

// The test uses the current devcontainer's already-built, content-addressed
// image, not an unpinned pull or the SDK under publication test.
function createContainer(socket, remap = false) {
  const image = docker(['inspect', hostname(), '--format', '{{.Image}}']);
  assert.match(image, /^sha256:[0-9a-f]{64}$/);
  const container = docker([
    'create', '--entrypoint', '/bin/sh',
    ...(socket ? ['--mount', 'type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock'] : []),
    image, '-c',
    `if [ -f /tmp/prismpm-test-restart ]; then ${initializer} true; fi; exec sleep infinity`,
  ]);
  assert.match(container, /^[0-9a-f]{64}$/);
  try {
    docker(['cp', resolve(root, '.devcontainer/docker-ready.sh'), `${container}:${helper}`]);
    docker(['cp', resolve(root, '.devcontainer/docker-entrypoint.sh'), `${container}:${initializer}`]);
    docker(['start', container]);
    run(container, 'root', ['chmod', '0755', helper, initializer]);
    if (remap) {
      // Dev Containers remaps the image account to the checkout owner's IDs.
      // Reproduce that without rebuilding or modifying the running container.
      run(container, 'root', ['sh', '-eu', '-c', `
        identity=23571
        while getent group "$identity" >/dev/null || getent passwd "$identity" >/dev/null; do
          identity=$((identity + 1))
        done
        groupmod --gid "$identity" "$(id -gn vscode)"
        usermod --uid "$identity" --gid "$identity" vscode
        test "$(id -u vscode)" = "$identity"
        test "$(id -g vscode)" = "$identity"
      `]);
      assert.throws(() => run(container, 'root', ['getent', 'group', '1000']));
    }
    if (socket) {
      // Ensure the test models the race even if the host socket happens to
      // share the image user's primary GID. Changes stay in this container.
      run(container, 'root', ['sh', '-eu', '-c', `
        gid=$(stat -c %g /var/run/docker.sock)
        primary=$(id -g vscode)
        if [ "$gid" = "$primary" ]; then
          groupadd prismpm-test-primary
          primary=$(getent group prismpm-test-primary | cut -d: -f3)
        fi
        usermod --gid "$primary" --groups "" vscode
      `]);
    }
    return container;
  } catch (error) {
    docker(['rm', '--force', container]);
    throw error;
  }
}

function pendingShell(container, command, args = []) {
  const child = spawn('docker', [
    'exec', '--interactive', '--user', 'vscode', container,
    '/bin/sh', '-c', `printf 'ready:%s\\n' "$(id -G)"; ${command}`, '--', ...args,
  ], { stdio: ['pipe', 'pipe', 'pipe'] });
  let stdout = '';
  let stderr = '';
  let closed = false;
  const stderrWaiters = [];
  let readyResolve;
  let readyReject;
  const ready = new Promise((resolveReady, rejectReady) => {
    readyResolve = resolveReady;
    readyReject = rejectReady;
  });
  child.stdout.on('data', chunk => {
    stdout += chunk;
    if (stdout.includes('\n')) readyResolve(stdout.split('\n')[0]);
  });
  child.stderr.on('data', chunk => {
    stderr += chunk;
    for (const waiter of stderrWaiters) {
      if (stderr.includes(waiter.fragment)) waiter.resolve();
    }
  });
  const finished = new Promise((resolveFinished, rejectFinished) => {
    child.once('error', error => { readyReject(error); rejectFinished(error); });
    child.once('close', code => {
      closed = true;
      for (const waiter of stderrWaiters) {
        waiter.reject(new Error(`shell exited before ${waiter.fragment}: ${stderr}`));
      }
      if (!stdout.includes('\n')) readyReject(new Error(`shell exited before readiness: ${stderr}`));
      resolveFinished({ code, stdout, stderr });
    });
  });
  const timer = setTimeout(() => {
    readyReject(new Error('Docker test shell timed out'));
    child.kill();
  }, 40_000);
  finished.finally(() => clearTimeout(timer));
  const waitForStderr = fragment => {
    if (stderr.includes(fragment)) return Promise.resolve();
    if (closed) return Promise.reject(new Error(`shell exited before ${fragment}: ${stderr}`));
    return new Promise((resolve, reject) => stderrWaiters.push({fragment, resolve, reject}));
  };
  return { child, ready, finished, waitForStderr };
}

// Freeze the actual group databases between usermod's separate file updates.
// Only a disposable container is changed; the host socket stays untouched.
function addGroupMember(container, database, group) {
  assert.ok(['/etc/group', '/etc/gshadow'].includes(database));
  run(container, 'root', ['node', '-e', `
    const assert = require('node:assert/strict'), fs = require('node:fs');
    const [path, group] = process.argv.slice(1);
    let found = false;
    const lines = fs.readFileSync(path, 'utf8').split('\\n').map(line => {
      const fields = line.split(':');
      if (fields[0] !== group) return line;
      assert.equal(fields.length, 4);
      const members = fields[3].split(',').filter(Boolean);
      assert.ok(!members.includes('vscode'), 'fixture member is already present');
      fields[3] = [...members, 'vscode'].join(','); found = true;
      return fields.join(':');
    });
    assert.ok(found, 'fixture group is absent');
    fs.writeFileSync(path, lines.join('\\n'));
  `, database, group]);
}

const probe = `
  const {execFileSync} = require('node:child_process');
  console.log(JSON.stringify({
    uid: process.getuid(), groups: process.getgroups(), args: process.argv.slice(1),
    docker: execFileSync('docker', ['version', '--format', '{{.Server.Version}}'], {encoding:'utf8'}).trim()
  }));
`;

test('devcontainer lifecycle waits before attach and wraps the actual fetch command', () => {
  const config = JSON.parse(readFileSync(resolve(root, '.devcontainer/devcontainer.json')));
  assert.equal(config.remoteUser, 'vscode');
  assert.equal(config.overrideCommand, false);
  assert.equal(config.waitFor, 'postStartCommand');
  assert.deepEqual(config.postCreateCommand.slice(0, 3), [helper, '/bin/sh', '-c']);
  assert.equal(config.postCreateCommand[3],
    'toolchain=$(tr -d \'\\n\' < lean-toolchain); if ! elan toolchain list | grep -Fqx "$toolchain"; then elan toolchain install "$toolchain"; fi; lean --version; ./scripts/fetch.sh');
  assert.deepEqual(config.postStartCommand, [helper, '/usr/bin/true']);
  assert.match(readFileSync(resolve(root, '.devcontainer/Dockerfile'), 'utf8'),
    /COPY --chmod=0755 \.devcontainer\/docker-ready\.sh \/usr\/local\/bin\/prismpm-devcontainer-exec/);
});

async function checkSocket(t, remap) {
  const container = createContainer(true, remap);
  try {
    const socketGid = Number(run(container, 'root', ['stat', '-c', '%g', '/var/run/docker.sock']));
    const uid = Number(run(container, 'vscode', ['id', '-u']));
    assert.notEqual(uid, 0);
    const socketBefore = run(container, 'root', ['stat', '-c', '%u:%g:%a', '/var/run/docker.sock']);
    const args = ['plain', '', 'two words', "a'b\"c", '$(touch /tmp/prismpm-injected)',
      '`touch /tmp/prismpm-injected`', 'line\nbreak\n'];

    await t.test('actual stale process fails without refresh and succeeds with it, retaining UID and argv', async () => {
      const stale = pendingShell(container, 'read start; exec docker version --format "{{.Server.Version}}"');
      const repaired = pendingShell(container, 'exec "$@"', [helper, 'node', '-e', probe, ...args]);
      const readiness = await Promise.all([stale.ready, repaired.ready]);
      for (const line of readiness) {
        assert(!line.slice('ready:'.length).split(' ').map(Number).includes(socketGid));
      }
      run(container, 'root', [initializer, 'true']);
      stale.child.stdin.end('\n');
      repaired.child.stdin.end();
      const oldResult = await stale.finished;
      assert.equal(oldResult.code, 1);
      assert.match(oldResult.stderr, /permission denied/);
      const result = await repaired.finished;
      assert.equal(result.code, 0, result.stderr);
      const actual = JSON.parse(result.stdout.split('\n')[1]);
      assert.equal(actual.uid, uid);
      assert(actual.groups.includes(socketGid));
      assert.deepEqual(actual.args, args);
      assert.match(actual.docker, /^\d+\.\d+\.\d+/);
      run(container, 'root', ['test', '!', '-e', '/tmp/prismpm-injected']);
    });

    await t.test('fresh process and restart remain non-root and Docker-ready', () => {
      const fresh = JSON.parse(run(container, 'vscode', [helper, 'node', '-e', probe]));
      assert.equal(fresh.uid, uid);
      assert(fresh.groups.includes(socketGid));
      assert.throws(() => run(container, 'vscode', [helper, '/bin/sh', '-c', 'exit 17']),
        error => error.status === 17);
      run(container, 'root', ['usermod', '--groups', '', 'vscode']);
      run(container, 'root', ['touch', '/tmp/prismpm-test-restart']);
      docker(['restart', '--time', '1', container]);
      const restarted = JSON.parse(run(container, 'vscode', [helper, 'node', '-e', probe]));
      assert.equal(restarted.uid, uid);
      assert(restarted.groups.includes(socketGid));
      assert.equal(run(container, 'root', ['stat', '-c', '%u:%g:%a', '/var/run/docker.sock']), socketBefore);
    });

    await t.test('group visibility before shadow authorization cannot execute the user command', async () => {
      run(container, 'root', ['usermod', '--groups', '', 'vscode']);
      const group = run(container, 'root', ['getent', 'group', String(socketGid)]).split(':')[0];
      const deniedProbe = pendingShell(container, 'read start; exec sg "$1" -c true', [group]);
      const repaired = pendingShell(container, 'read start; exec "$@"', [
        helper, '/bin/sh', '-c', 'printf x >> /tmp/prismpm-skew-command; exec "$@"', '--',
        'node', '-e', probe, ...args,
      ]);
      const failingCommand = pendingShell(container, 'read start; exec "$@"', [
        helper, '/bin/sh', '-c', 'printf x >> /tmp/prismpm-skew-failure; exit 17',
      ]);
      for (const line of await Promise.all([deniedProbe.ready, repaired.ready, failingCommand.ready])) {
        assert(!line.slice('ready:'.length).split(' ').map(Number).includes(socketGid));
      }
      addGroupMember(container, '/etc/group', group);
      assert(run(container, 'root', ['id', '-G', 'vscode']).split(' ').map(Number).includes(socketGid));
      deniedProbe.child.stdin.end('\n');
      const deniedResult = await deniedProbe.finished;
      assert.equal(deniedResult.code, 1);
      assert.match(deniedResult.stderr, /Invalid password/);
      repaired.child.stdin.end('\n');
      failingCommand.child.stdin.end('\n');
      await Promise.all([repaired, failingCommand].map(shell =>
        shell.waitForStderr('waiting for socket group authorization')));
      run(container, 'root', ['test', '!', '-e', '/tmp/prismpm-skew-command']);
      run(container, 'root', ['test', '!', '-e', '/tmp/prismpm-skew-failure']);
      addGroupMember(container, '/etc/gshadow', group);
      const result = await repaired.finished;
      assert.equal(result.code, 0, result.stderr);
      const actual = JSON.parse(result.stdout.split('\n')[1]);
      assert.equal(actual.uid, uid);
      assert(actual.groups.includes(socketGid));
      assert.deepEqual(actual.args, args);
      assert.match(actual.docker, /^\d+\.\d+\.\d+/);
      assert.equal(run(container, 'vscode', ['cat', '/tmp/prismpm-skew-command']), 'x');
      assert.equal((await failingCommand.finished).code, 17);
      assert.equal(run(container, 'vscode', ['cat', '/tmp/prismpm-skew-failure']), 'x');
      assert.equal(run(container, 'root', ['stat', '-c', '%u:%g:%a', '/var/run/docker.sock']), socketBefore);
    });

    await t.test('missing shadow authorization times out without executing the command or prompting', async () => {
      run(container, 'root', ['usermod', '--groups', '', 'vscode']);
      const group = run(container, 'root', ['getent', 'group', String(socketGid)]).split(':')[0];
      const blocked = pendingShell(container, 'read start; exec "$@"', [
        helper, 'touch', '/tmp/prismpm-unauthorized',
      ]);
      assert(!(await blocked.ready).slice('ready:'.length).split(' ').map(Number).includes(socketGid));
      addGroupMember(container, '/etc/group', group);
      const started = Date.now();
      blocked.child.stdin.end('\n');
      const result = await blocked.finished;
      assert.equal(result.code, 1);
      assert.match(result.stderr, /timed out waiting for socket group authorization/);
      assert.doesNotMatch(result.stderr, /Password:|Invalid password/);
      assert(Date.now() - started >= 28_000);
      assert(Date.now() - started < 40_000);
      run(container, 'root', ['test', '!', '-e', '/tmp/prismpm-unauthorized']);
      assert.equal(run(container, 'root', ['stat', '-c', '%u:%g:%a', '/var/run/docker.sock']), socketBefore);
    });

    await t.test('missing initialization fails within its deadline without executing the command', () => {
      run(container, 'root', ['usermod', '--groups', '', 'vscode']);
      const started = Date.now();
      denied(container, 'vscode', [helper, 'touch', '/tmp/prismpm-uninitialized'], /timed out waiting/);
      assert(Date.now() - started >= 28_000);
      assert(Date.now() - started < 40_000);
      run(container, 'root', ['test', '!', '-e', '/tmp/prismpm-uninitialized']);
      denied(container, 'root', [helper, 'touch', '/tmp/prismpm-root'], /non-root remote user/);
      run(container, 'root', ['test', '!', '-e', '/tmp/prismpm-root']);
      denied(container, 'vscode', [helper], /a command is required/);
    });
  } finally {
    docker(['rm', '--force', container]);
    assert.throws(() => docker(['inspect', container]));
  }
}

for (const remap of [false, true]) {
  const identity = remap ? 'remapped remote user' : 'image remote user';
  test(`real Docker socket (${identity}): stale shell, fresh shell, restart and bounded failure`,
    { timeout: 90_000 }, t => checkSocket(t, remap));
  test(`missing mounted socket fails closed (${identity})`, () => {
    const container = createContainer(false, remap);
    try {
      denied(container, 'vscode', [helper, 'touch', '/tmp/prismpm-no-socket'], /mounted Docker socket is missing/);
      run(container, 'root', ['test', '!', '-e', '/tmp/prismpm-no-socket']);
    } finally {
      docker(['rm', '--force', container]);
      assert.throws(() => docker(['inspect', container]));
    }
  });
}
