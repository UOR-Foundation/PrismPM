import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, mkdtempSync, writeFileSync, readFileSync, realpathSync, rmSync, chmodSync, symlinkSync, unlinkSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import test from 'node:test';
import {credentialHelperRunner} from './metadata-helper.mjs';

function helper(t, program, {header = '#!/usr/bin/python3 -I', extra = []} = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'sdk-credential-helper-'));
  t.after(() => rmSync(directory, {recursive:true, force:true}));
  const executable = join(directory, 'docker-credential-fixture');
  const bytes = Buffer.from(header + '\n' + program + '\n');
  writeFileSync(executable, bytes, {mode:0o700});
  const row = {command:'docker-credential-fixture', executable, sha256:createHash('sha256').update(bytes).digest('hex')};
  const runtime = ['python3'].map(command => {
    const executable = realpathSync('/usr/bin/python3');
    return {command, executable, sha256:createHash('sha256').update(readFileSync(executable)).digest('hex')};
  });
  return {row, directory, run:credentialHelperRunner([row,...runtime,...extra])};
}
const invoke = (fixture, signal = new AbortController().signal) => fixture.run('docker-credential-fixture','ghcr.io',signal);

test('real verified helper receives get plus server stdin and returns bounded credentials', async t => {
  const fixture = helper(t, `import sys, json
assert len(sys.argv) == 2 and sys.argv[1] == 'get'
assert sys.stdin.read() == 'ghcr.io\\n'
print(json.dumps({'Username':'fixture','Secret':'synthetic-secret'}))`);
  assert.deepEqual(await invoke(fixture), {Username:'fixture', Secret:'synthetic-secret'});
});

test('helper failures and malformed output never expose secret diagnostics', async t => {
  for (const program of [
    `print('synthetic-secret')`,
    `import sys; sys.stderr.write('synthetic-secret'); sys.exit(2)`,
    `import sys; print('credentials not found in native keychain\\nsynthetic-secret'); sys.exit(1)`,
    `print('synthetic-secret' * 10000)`,
    `import sys; sys.stderr.write('synthetic-secret' * 10000)`,
  ]) await assert.rejects(invoke(helper(t, program)), error => !String(error.stack).includes('synthetic-secret'));
  const missing = helper(t, `import sys; print('credentials not found in native keychain'); sys.exit(1)`);
  assert.equal(await invoke(missing), null);
});

test('changed, writable, absent and duplicate SDK helper identities refuse execution', async t => {
  const fixture = helper(t, `print('{}')`);
  writeFileSync(fixture.row.executable, '#!/usr/bin/python3 -I\nprint(1)\n');
  await assert.rejects(invoke(fixture), /execution or cleanup failed/);
  chmodSync(fixture.row.executable, 0o777);
  await assert.rejects(invoke(fixture), /execution or cleanup failed/);
  for (const commands of [[], [fixture.row, fixture.row]])
    await assert.rejects(credentialHelperRunner(commands)('docker-credential-fixture','ghcr.io',new AbortController().signal), /uniquely present/);
});

test('sealed helper execution rejects self-modification and strips interpreter override variables', async t => {
  const fixture = helper(t, `import os, sys, json, errno
try:
    with open(sys.argv[0], 'wb') as stream: stream.write(b'replacement')
    sys.exit(9)
except OSError as error:
    assert error.errno == errno.EPERM
assert not any(key in os.environ for key in ['NODE_OPTIONS','LD_PRELOAD','PYTHONPATH'])
print(json.dumps({'Username':'sealed','Secret':'synthetic'}))`);
  const saved = Object.fromEntries(['NODE_OPTIONS','LD_PRELOAD','PYTHONPATH'].map(key => [key,process.env[key]]));
  try {
    process.env.NODE_OPTIONS = '--require=/must-not-load'; process.env.LD_PRELOAD = '/must-not-load'; process.env.PYTHONPATH = '/must-not-load';
    assert.deepEqual(await invoke(fixture), {Username:'sealed',Secret:'synthetic'});
  } finally {for (const [key,value] of Object.entries(saved)) {if (value === undefined) delete process.env[key]; else process.env[key] = value;}}
});

test('escaped-session descendants are reaped with inherited and closed output pipes', async t => {
  for (const pipes of ['inherit','ignore']) {
    const directory = mkdtempSync(join(tmpdir(),'sdk-helper-descendant-'));
    t.after(() => rmSync(directory,{recursive:true,force:true}));
    const pidFile = join(directory,'pid');
    const fixture = helper(t, `import os, time, json
pid = os.fork()
if pid == 0:
    os.setsid()
    ${pipes === 'ignore' ? 'os.close(1); os.close(2)' : 'pass'}
    time.sleep(60)
    os._exit(0)
with open(${JSON.stringify(pidFile)}, 'w') as stream: stream.write(str(pid))
print(json.dumps({'Username':'fixture','Secret':'synthetic'}))`);
    const controller = new AbortController(), start = performance.now();
    const timer = setTimeout(() => controller.abort(),500);
    try {
      if (pipes === 'inherit') await assert.rejects(invoke(fixture,controller.signal), /deadline exceeded/);
      else assert.deepEqual(await invoke(fixture,controller.signal), {Username:'fixture',Secret:'synthetic'});
    } finally {clearTimeout(timer);}
    assert(performance.now() - start < 3000);
    const pid = Number(readFileSync(pidFile,'utf8'));
    assert.throws(() => process.kill(pid,0), {code:'ESRCH'}, 'owned escaped descendant must be gone before settlement');
  }
});

test('deadline kills an actual stalled helper and inherited-pipe descendant', async t => {
  const fixture = helper(t, `import os, time
os.fork()
time.sleep(60)`);
  const controller = new AbortController(), start = performance.now();
  const timer = setTimeout(() => controller.abort(), 100);
  try {await assert.rejects(invoke(fixture, controller.signal), /deadline exceeded/);}
  finally {clearTimeout(timer);}
  assert(performance.now() - start < 2000, 'helper descendants must not retain output pipes after timeout');
});

test('shebang alias replacement cannot change the verified canonical interpreter', async t => {
  const directory = mkdtempSync(join(tmpdir(),'sdk-interpreter-alias-'));
  t.after(() => rmSync(directory,{recursive:true,force:true}));
  const alias = join(directory,'interpreter'); symlinkSync('/usr/bin/python3',alias);
  const fixture = helper(t, `import json; print(json.dumps({'Username':'canonical','Secret':'synthetic'}))`, {header:'#!' + alias});
  const pending = invoke(fixture);
  unlinkSync(alias); symlinkSync('/usr/bin/false',alias);
  assert.deepEqual(await pending,{Username:'canonical',Secret:'synthetic'});
});

test('env interpreter dispatch uses the verified selection despite a conflicting real PATH command', async t => {
  const extra = [['env','/usr/bin/env'],['false','/usr/bin/python3']].map(([command,path]) => {
    const executable = realpathSync(path);
    return {command,executable,sha256:createHash('sha256').update(readFileSync(executable)).digest('hex')};
  });
  const fixture = helper(t, `import json; print(json.dumps({'Username':'selected','Secret':'synthetic'}))`, {header:'#!/usr/bin/env false',extra});
  // /usr/bin/false really exists and would exit 1; it must not win PATH search.
  assert.deepEqual(await invoke(fixture),{Username:'selected',Secret:'synthetic'});
});

test('bare Python helper shebangs cannot execute planted HOME user-site hooks', async t => {
  const directory = mkdtempSync(join(tmpdir(),'sdk-helper-user-site-'));
  t.after(() => rmSync(directory,{recursive:true,force:true}));
  const version = execFileSync('/usr/bin/python3',['-I','-c','import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")'],
    {encoding:'utf8',timeout:5000}).trim();
  const site = join(directory,'.local/lib/python' + version + '/site-packages'), marker = join(directory,'unverified-startup');
  mkdirSync(site,{recursive:true});
  writeFileSync(join(site,'usercustomize.py'), 'open(' + JSON.stringify(marker) + ', "w").write("unverified startup")\n');
  const previous = process.env.HOME;
  try {
    process.env.HOME = directory;
    // Establish that the fixture really is active without isolation.
    execFileSync('/usr/bin/python3',['-c','pass'],{env:process.env,timeout:5000});
    assert(existsSync(marker)); unlinkSync(marker);
    const fixture = helper(t, `import json; print(json.dumps({'Username':'isolated','Secret':'synthetic'}))`, {header:'#!/usr/bin/python3'});
    assert.deepEqual(await invoke(fixture),{Username:'isolated',Secret:'synthetic'});
    assert(!existsSync(marker), 'unverified Python user-site startup must not execute');
  } finally {if (previous === undefined) delete process.env.HOME; else process.env.HOME = previous;}
});
