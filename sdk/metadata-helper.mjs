// SDK-selected helpers execute as sealed snapshots under a Linux subreaper.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {constants, closeSync, fstatSync, lstatSync, openSync, readFileSync, readSync, realpathSync} from 'node:fs';
import {dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {parseJson} from './metadata-layer.mjs';

function immutableRuntime(row) {
  assert(typeof row.executable === 'string' && row.executable.startsWith('/') && realpathSync(row.executable) === row.executable,
    'unaliased SDK interpreter required');
  // The interpreter is part of the root-owned SDK, not the helper/config input.
  for (let path = row.executable; ; path = dirname(path)) {
    const entry = lstatSync(path);
    assert(entry.uid === 0 && (entry.mode & 0o022) === 0, 'immutable SDK supervisor ownership required');
    if (path === '/') break;
  }
  const entry = lstatSync(row.executable);
  assert(entry.isFile() && entry.size > 0 && entry.size <= 128 * 1024 * 1024, 'bounded SDK interpreter required');
  const bytes = readFileSync(row.executable);
  assert(bytes.length <= 128 * 1024 * 1024 && createHash('sha256').update(bytes).digest('hex') === row.sha256,
    'SDK supervisor digest differs');
  return row.executable;
}

function helperHeader(row, commands) {
  const fd = openSync(row.executable,constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    assert(fstatSync(fd).isFile(), 'regular credential helper required');
    const bytes = Buffer.alloc(256), length = readSync(fd,bytes,0,bytes.length,0);
    if (bytes.subarray(0,2).toString() !== '#!') return {header:null, interpreter:null};
    const newline = bytes.subarray(0,length).indexOf(10);
    assert(newline > 2, 'bounded credential helper interpreter required');
    const header = bytes.subarray(0,newline).toString('utf8');
    const parsed = /^#!\s*(\/[^\s]+)(?:\s+(.*))?$/.exec(header);
    assert(parsed, 'absolute credential helper interpreter required');
    const interpreter = realpathSync(parsed[1]);
    const matches = commands.filter(command => command.executable === interpreter);
    assert(matches.length > 0, 'credential helper interpreter absent from SDK');
    let selectedRow = matches[0], executable = immutableRuntime(selectedRow), args = parsed[2] ? [parsed[2]] : [];
    if (interpreter.endsWith('/env')) {
      assert(parsed[2] && /^[a-zA-Z0-9._-]+$/.test(parsed[2]), 'closed SDK env interpreter selection required');
      const selected = commands.filter(command => command.command === parsed[2]);
      assert(selected.length === 1, 'env interpreter absent from SDK'); selectedRow = selected[0]; executable = immutableRuntime(selectedRow); args = [];
    }
    // Dispatch directly, never re-resolve a shebang symlink or /usr/bin/env
    // PATH lookup after validation. Node requires its script fd be preserved.
    const runtimeIs = name => commands.some(row => row.command === name && row.executable === executable);
    if (runtimeIs('node') && !args.includes('--preserve-symlinks-main')) args.unshift('--preserve-symlinks-main');
    if (runtimeIs('python3') && !args.includes('-I')) args.unshift('-I');
    return {header, interpreter:{executable,args}};
  } finally {closeSync(fd);}
}

export function credentialHelperRunner(commands) {
  assert(Array.isArray(commands), 'verified SDK command inventory required');
  return async (command, server, signal) => {
    signal.throwIfAborted();
    assert(/^docker-credential-[a-z0-9][a-z0-9._-]{0,63}$/.test(command), 'invalid SDK credential helper name');
    assert(typeof server === 'string' && server.length > 0 && server.length <= 4096 && !/[\r\n\0]/.test(server), 'bounded registry server required');
    const rows = commands.filter(row => row.command === command);
    assert(rows.length === 1, 'credential helper is not uniquely present in the verified SDK');
    const runtimes = commands.filter(row => row.command === 'python3');
    assert(runtimes.length === 1, 'verified SDK Python supervisor required');
    const python = immutableRuntime(runtimes[0]), {header,interpreter} = helperHeader(rows[0],commands);
    const directories = [...new Set(commands.map(row => dirname(row.executable)))].filter(directory => {
      for (let path = directory; ; path = dirname(path)) {
        const entry = lstatSync(path);
        if (!entry.isDirectory() || entry.uid !== 0 || (entry.mode & 0o022) !== 0) return false;
        if (path === '/') return true;
      }
    });
    const environment = {LANG:'C.UTF-8', PATH:directories.join(':')};
    for (const key of ['HOME','DOCKER_CONFIG','DBUS_SESSION_BUS_ADDRESS','DISPLAY','XDG_RUNTIME_DIR','GNUPGHOME','PASSWORD_STORE_DIR','GPG_TTY'])
      if (process.env[key] !== undefined) environment[key] = process.env[key];
    const request = JSON.stringify({command:rows[0],header,interpreter,server,environment});
    assert(Buffer.byteLength(request) <= 65536, 'bounded credential helper request required');
    return new Promise((resolve, reject) => {
      const child = spawn(python, ['-I',fileURLToPath(new URL('./metadata-helper-supervisor.py',import.meta.url))],
        {detached:true, env:environment, stdio:['pipe','pipe','pipe']});
      let size = 0, failure, hardTimer, settled = false; const stdout = [];
      const fail = message => {
        if (failure) return; failure = new Error(message);
        child.kill('SIGTERM');
        hardTimer = setTimeout(() => {
          // A stuck supervisor cannot establish cleanup. Return failure, never
          // retry or pretend successful acquisition; close inherited readers.
          child.kill('SIGKILL'); child.stdout.destroy(); child.stderr.destroy();
          failure = new Error('credential helper cleanup not established');
          if (!settled) {
            settled = true; clearTimeout(timer); signal.removeEventListener('abort',abort);
            child.stdin.destroy(); child.unref(); reject(failure);
          }
        },2500);
      };
      const abort = () => fail('credential helper deadline exceeded');
      const timer = setTimeout(abort,15000);
      signal.addEventListener('abort',abort,{once:true}); if (signal.aborted) abort();
      const collect = (bytes, output) => {
        size += bytes.length;
        if (size > 512 * 1024) fail('credential supervisor output exceeds bound');
        else if (output) stdout.push(bytes);
      };
      child.stdout.on('data',bytes => collect(bytes,true)); child.stderr.on('data',bytes => collect(bytes,false));
      child.stdin.on('error',() => fail('credential supervisor input failed'));
      child.on('error',() => fail('credential supervisor execution failed'));
      child.on('close',code => {
        clearTimeout(timer); clearTimeout(hardTimer); signal.removeEventListener('abort',abort);
        if (settled) return; settled = true;
        if (failure) {reject(failure); return;}
        if (code !== 0) {reject(new Error('credential helper execution or cleanup failed')); return;}
        try {
          const envelope = parseJson(Buffer.concat(stdout));
          assert(envelope && Object.keys(envelope).join(',') === 'output');
          assert(envelope.output === null || (typeof envelope.output === 'string' && Buffer.byteLength(envelope.output) <= 65536));
          resolve(envelope.output === null ? null : parseJson(Buffer.from(envelope.output)));
        } catch {reject(new Error('invalid credential helper response'));}
      });
      child.stdin.end(request);
    });
  };
}
