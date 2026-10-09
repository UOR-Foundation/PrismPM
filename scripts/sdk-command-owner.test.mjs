// Real Linux processes and executed source mutants, not installed-SDK acceptance.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {randomBytes} from 'node:crypto';
import {closeSync, constants, existsSync, mkdtempSync, openSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import test from 'node:test';
import {executeOwnedSdkCommand} from './sdk-command-owner.mjs';
import {executeQualificationProcess} from './sdk-registry-qualification.mjs';

const options = {profile: 'vv', timeout: 3000, limit: 1024};
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const absent = pid => assert(!existsSync(`/proc/${pid}`), 'real fixture must be absent BEFORE safety cleanup');
async function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'prism-sdk-owned-process-'));
  const marker = join(root, 'child.json'), token = randomBytes(16).toString('hex');
  t.after(async () => {
    if (existsSync(marker)) {
      const row = JSON.parse(readFileSync(marker)); assert.equal(row.token, token);
      assert(Number.isSafeInteger(row.pid) && row.pid > 1);
      if (existsSync(`/proc/${row.pid}`)) {
        const command = readFileSync(`/proc/${row.pid}/cmdline`, 'utf8').split('\0');
        assert(command.includes(marker) && command.includes(token), 'only identified test fixture may be cleaned');
        process.kill(row.pid, 'SIGKILL'); const deadline = performance.now() + 5000;
        while (existsSync(`/proc/${row.pid}`) && performance.now() < deadline) await pause(10);
        absent(row.pid);
      }
    }
    rmSync(root, {recursive: true});
  });
  const holder = `require('node:fs').writeFileSync(process.argv[1],JSON.stringify({pid:process.pid,token:process.argv[2]}),{flag:'wx'});`;
  const parent = (tail, detached = true, pipes = false) => `const fs=require('node:fs');const c=require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(holder + tail)},${JSON.stringify(marker)},${JSON.stringify(token)}],{detached:${detached},stdio:${pipes ? "['ignore',1,2]" : "'ignore'"}});const timer=setInterval(()=>{if(fs.existsSync(${JSON.stringify(marker)})){clearInterval(timer);c.unref();process.exit(0)}},5);setTimeout(()=>process.exit(97),2000).unref();`;
  return {root, marker, token, holder, parent, pid: () => JSON.parse(readFileSync(marker)).pid};
}
async function mutant(t, transform) {
  const root = mkdtempSync(join(tmpdir(), 'prism-sdk-owner-source-mutant-'));
  t.after(() => rmSync(root, {recursive: true}));
  const original = readFileSync(new URL('./sdk-command-owner.py', import.meta.url), 'utf8');
  const changed = transform(original); assert.notEqual(changed, original, 'mutant must actually change production source');
  writeFileSync(join(root, 'sdk-command-owner.py'), changed, {flag: 'wx'});
  writeFileSync(join(root, 'sdk-command-owner.mjs'), readFileSync(new URL('./sdk-command-owner.mjs', import.meta.url)), {flag: 'wx'});
  return (await import(pathToFileURL(join(root, 'sdk-command-owner.mjs')).href)).executeOwnedSdkCommand;
}

test('natural trailing escaped work preserves original result and complete output within the original deadline', async t => {
  const f = await fixture(t), tail = "setTimeout(()=>process.stdout.write('TRAILING\\n'),120);";
  const result = await executeOwnedSdkCommand(process.execPath, ['-e', f.parent(tail, true, true)], options);
  assert.equal(result.status, 0); assert.equal(result.signal, null); assert.equal(result.timedOut, false);
  assert.equal(result.stdout.toString(), 'TRAILING\n'); assert.equal(result.retirement.descendants_absent, true);
  assert.equal(result.retirement.supervisor_status, 0); absent(f.pid());
});

test('qualification normal leader exit cannot hide redirected same-group or escaped children', async t => {
  for (const detached of [false, true]) {
    const f = await fixture(t), start = performance.now();
    const result = await executeQualificationProcess(process.execPath, ['-e', f.parent('setTimeout(()=>{},15000);', detached)], {timeout: 300});
    assert.equal(result.status, 0); assert.equal(result.signal, null); assert.equal(result.timedOut, true);
    assert.equal(result.descendants_absent, true); assert.equal(result.orphaned, true);
    assert(performance.now() - start < 5500); absent(f.pid());
  }
});

test('post-leader cancellation adopts late double-forked work without signaling an unrelated sentinel', async t => {
  const f = await fixture(t), sentinel = spawn(process.execPath, ['-e', "process.stdout.write('SENTINEL\\n');setInterval(()=>{},1000)"], {stdio: ['ignore', 'pipe', 'ignore']});
  const sentinelClosed = new Promise(resolve => sentinel.once('close', resolve));
  t.after(async () => {sentinel.kill('SIGKILL'); await sentinelClosed;});
  await new Promise(resolve => sentinel.stdout.once('data', resolve));
  const cancellation = new AbortController();
  const grandchild = f.holder + "process.stdout.write('GRANDCHILD\\n');process.on('SIGTERM',()=>{});setTimeout(()=>{},15000);";
  const middle = `require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(grandchild)},${JSON.stringify(f.marker)},${JSON.stringify(f.token)}],{detached:true,stdio:['ignore',1,2]}).unref();setTimeout(()=>process.exit(0),50);`;
  const parent = `require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(middle)}],{detached:true,stdio:['ignore',1,2]}).unref();process.exit(0);`;
  let timer;
  try {
    const result = await executeQualificationProcess(process.execPath, ['-e', parent], {signal: cancellation.signal,
      onOutput: (_, bytes) => {if (bytes.includes('GRANDCHILD') && !timer) timer = setTimeout(() => cancellation.abort(), 200);}});
    assert.equal(result.status, 0); assert.equal(result.aborted, true); assert.equal(result.descendants_absent, true);
    assert.equal(result.process_retirement.leader_exit_observed, true); absent(f.pid());
    assert.equal(sentinel.exitCode, null); assert.equal(sentinel.signalCode, null); process.kill(sentinel.pid, 0);
  } finally {clearTimeout(timer);}
});

test('concurrent private owners preserve independent real statuses and exhaust both descendant trees', async t => {
  const one = await fixture(t), two = await fixture(t);
  const results = await Promise.all([one, two].map((f, i) => executeOwnedSdkCommand(process.execPath,
    ['-e', f.parent(`setTimeout(()=>{},${100 + i * 50});`).replace('process.exit(0)', `process.exit(${7 + i})`)], options)));
  assert.deepEqual(results.map(r => r.status), [7, 8]);
  for (const [i, result] of results.entries()) {assert.equal(result.retirement.descendants_absent, true); absent([one, two][i].pid());}
});

test('real escaped output overflow is bounded and reaped before the failed result', async t => {
  const f = await fixture(t);
  await assert.rejects(executeOwnedSdkCommand(process.execPath, ['-e', f.parent("setTimeout(()=>{process.stdout.write('X'.repeat(8192));setInterval(()=>{},1000)},100);", true, true)], options), error => {
    assert.match(error.message, /output exceeded/); assert.equal(error.result.stdout.length, 1024);
    assert.equal(error.result.retirement.descendants_absent, true); absent(f.pid()); return true;
  });
});

test('actual subreaper admission refusal prevents the command from running', async t => {
  const f = await fixture(t), run = await mutant(t, s => s.replace('libc.prctl(36, 1, 0, 0, 0) != 0', 'True'));
  await assert.rejects(run(process.execPath, ['-e', f.holder, f.marker, f.token], options), error => {
    assert.equal(error.result.retirement.descendants_absent, false); assert.equal(error.result.status, null); return true;
  });
  assert(!existsSync(f.marker));
});

test('post-spawn observation failure is not repaired by later successful cleanup', async t => {
  const f = await fixture(t), run = await mutant(t, s => s.replace('self.known.add(self.process.pid)', 'self.known.add(self.process.pid)\n            self.child_path = "/proc/nonexistent-sdk-observer"'));
  await assert.rejects(run(process.execPath, ['-e', f.holder + 'setTimeout(()=>{},15000);', f.marker, f.token], {...options, timeout: 300}), error => {
    assert.equal(error.result.retirement.descendants_absent, false, 'failed observation must prohibit a receipt');
    assert.equal(error.result.retirement.supervisor_status, 125); absent(f.pid()); return true;
  });
});

test('private descriptors cannot be forged by commands and malformed terminal protocols fail closed', async t => {
  const program = String.raw`const fs=require('node:fs');for(const fd of [3,4]){try{fs.writeSync(fd,'{"event":"completed"}\n')}catch{process.stdout.write('PRIVATE-CLOSED\n')}}process.exit(7)`;
  const result = await executeOwnedSdkCommand(process.execPath, ['-e', program], options);
  assert.equal(result.status, 7); assert.equal(result.retirement.descendants_absent, true);
  assert.equal(result.stdout.toString(), 'PRIVATE-CLOSED\nPRIVATE-CLOSED\n');
  for (const transform of [s => s.replace('"event": "completed"', '"event": "wrong-terminal"'),
    s => s.replace('raw = (json.dumps(row', 'emit_duplicate = row.get("event") == "completed"\n    raw = (json.dumps(row').replace('    while raw:', '    if emit_duplicate:\n        raw += raw\n    while raw:'),
    s => s.replace('if len(raw) > 1024:', 'if row.get("event") == "completed":\n        raw = raw[:-1]\n    if len(raw) > 1024:')]) {
    const run = await mutant(t, transform);
    await assert.rejects(run(process.execPath, ['-e', 'process.exit(0)'], options), error => {
      assert.equal(error.result.retirement.descendants_absent, false); return true;
    });
  }
});

test('one bounded reap pass cannot be mistaken for exhaustion; executed omission mutant is killed', async t => {
  const f = await fixture(t);
  const onePass = await mutant(t, s => s.replace('range(4096)', 'range(1)'));
  const correct = await onePass(process.execPath, ['-e', f.parent('setTimeout(()=>{},120);')], options);
  assert.equal(correct.retirement.descendants_absent, true); absent(f.pid());
  const bad = await fixture(t), early = await mutant(t, s => s.replace('            if pid == 0:\n                return', '            if pid == 0:\n                self.cleaned = self.leader is not None  # executed premature-exhaustion mutation\n                return'));
  await assert.rejects(async () => {
    const result = await early(process.execPath, ['-e', bad.parent('setTimeout(()=>{},15000);')], options);
    assert.equal(result.retirement.descendants_absent, true); absent(bad.pid());
  }, 'the real-process absence oracle must reject premature exhaustion');
});

test('foreground expiry after supervisor exit remains failure while a real outside writer delays stream close', async () => {
  let held, timer;
  try {
    await assert.rejects(executeOwnedSdkCommand(process.execPath, ['-e', "process.stdout.write('PID '+process.pid+'\\n');setTimeout(()=>process.exit(0),30)"],
      {...options, timeout: 200, onOutput: (_, bytes) => {
        const match = /^PID ([1-9][0-9]*)\n$/.exec(bytes.toString());
        if (match) {
          // Real outside writer, not a fake child/event adapter. It intentionally
          // holds the ORIGINAL pipe after the owner has exhausted its tree.
          held = openSync(`/proc/${match[1]}/fd/1`, constants.O_WRONLY | constants.O_NONBLOCK);
          timer = setTimeout(() => {closeSync(held); held = undefined;}, 350);
        }
      }}), error => {
        assert.match(error.message, /timed out/); assert.equal(error.result.status, 0);
        assert.equal(error.result.timedOut, true); assert.equal(error.result.retirement.supervisor_status, 0);
        assert.equal(error.result.retirement.close_observed, true); return true;
      });
  } finally {clearTimeout(timer); if (held !== undefined) closeSync(held);}
});

test('direct supervisor interruption is never repaired by subsequent descendant reaping', async t => {
  const f = await fixture(t); let timer;
  const program = "process.stdout.write('SUPERVISOR '+process.ppid+'\\n');" + f.parent('setTimeout(()=>{},15000);');
  try {
    await assert.rejects(executeOwnedSdkCommand(process.execPath, ['-e', program], {...options,
      onOutput: (_, bytes) => {
        const match = /^SUPERVISOR ([1-9][0-9]*)\n$/.exec(bytes.toString());
        if (match) timer = setTimeout(() => {
          const command = readFileSync(`/proc/${match[1]}/cmdline`, 'utf8').split('\0');
          assert(command.some(argument => argument.endsWith('/sdk-command-owner.py')));
          assert(command.includes(program), 'interrupt only the actual private test invocation');
          process.kill(Number(match[1]), 'SIGTERM');
        }, 250);
      }}), error => {
        assert.equal(error.result.status, 0); assert.equal(error.result.retirement.supervisor_status, 125);
        assert.equal(error.result.retirement.owner_error, 'supervisor-interrupted');
        assert.equal(error.result.retirement.descendants_absent, false); absent(f.pid()); return true;
      });
  } finally {clearTimeout(timer);}
});
