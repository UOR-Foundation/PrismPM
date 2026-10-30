// Scratch: records which repository files one browser owner really reads.
// Every tracked atime is reset to an old date; relatime then updates it on the
// first read by any process, including Docker daemons and child compilers.
import {execFileSync, spawnSync} from 'node:child_process';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {suites, standaloneSuites} from './browser-api-sdk-check.mjs';

const id = process.argv[2];
const suite = [...suites, ...standaloneSuites].find(row => row.id === id);
if (!suite) throw Error(`unknown owner ${id}`);
const bash = script => execFileSync('bash', ['-c', script], {encoding: 'utf8', maxBuffer: 1 << 30});
const reset = String.raw`find . -path ./target -prune -o \( -type f -o -type d \) -print0 | xargs -0 touch -a -h -d 2000-01-01T00:00:00Z 2>/dev/null || true`;
const list = String.raw`find . -path ./target -prune -o \( -type f -o -type d \) -newerat 2001-01-01T00:00:00Z -print | LC_ALL=C sort`;

console.log('mount', bash('findmnt -T . -o TARGET,OPTIONS -n || true').trim());
bash(reset);
const before = bash(list).split('\n').filter(Boolean);
readFileSync('Justfile');
const canary = bash(list).split('\n').filter(Boolean).includes('./Justfile');
console.log(JSON.stringify({canary, preRead: before.length}));
if (!canary) throw Error('atime tracing does not work on this mount');
bash(reset);

const started = Date.now();
const run = spawnSync(process.execPath, ['--test', '--test-concurrency=1', `--test-timeout=${suite.deadline}`, ...suite.files],
  {stdio: ['ignore', 'pipe', 'pipe'], maxBuffer: 1 << 30, timeout: 19800000});
const elapsedSeconds = Math.round((Date.now() - started) / 1000);
const reads = bash(list).split('\n').filter(Boolean).map(path => path.replace(/^\.\//, ''));
const tail = run.stdout.toString().split('\n').slice(-14).join('\n');
console.log(tail); console.log(run.stderr.toString().split('\n').slice(-10).join('\n'));
mkdirSync('target/trace', {recursive: true});
writeFileSync(`target/trace/${id.replaceAll(' ', '_')}.json`, JSON.stringify({id, files: suite.files, status: run.status, signal: run.signal,
  elapsedSeconds, readCount: reads.length, reads}, null, 1));
console.log(JSON.stringify({id, status: run.status, elapsedSeconds, readCount: reads.length}));
