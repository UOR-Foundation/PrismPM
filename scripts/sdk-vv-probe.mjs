// SDK verification probes, not Foundation application services.
import assert from 'node:assert/strict';
import { closeSync, constants, fstatSync, openSync, readFileSync, readdirSync, readSync, readlinkSync } from 'node:fs';
import { createConnection, createServer, isIPv4 } from 'node:net';
import { Resolver } from 'node:dns/promises';
import { pathToFileURL } from 'node:url';

export const isolatedResolver = 'nameserver 127.0.0.1\noptions timeout:1 attempts:1\n';
export function validateResolver(text) {
  assert(typeof text === 'string' && Buffer.byteLength(text) <= 16384);
  const rows = text.split('\n').map(line => line.replace(/#.*/, '').trim()).filter(Boolean);
  assert.equal(rows.filter(row => row === 'nameserver 127.0.0.1').length, 1);
  assert(rows.every(row => row === 'nameserver 127.0.0.1' || row === 'options timeout:1 attempts:1'), 'external or unexpected resolver configuration');
  assert(rows.length <= 2, 'duplicate resolver directives');
}

export function inspectNamespace() {
  return { interfaces: readdirSync('/sys/class/net').sort(), ipv4: readFileSync('/proc/net/route', 'utf8'),
    ipv6: readFileSync('/proc/net/ipv6_route', 'utf8'), identity: readlinkSync('/proc/self/ns/net') };
}

export function inspectNativeExecutable(path) {
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  const bytes = Buffer.alloc(64);
  try {
    const before = fstatSync(fd, {bigint: true});
    assert(before.isFile() && before.size >= 64n && before.size <= 268435456n);
    assert.equal(readSync(fd, bytes, 0, bytes.length, 0), bytes.length);
    const after = fstatSync(fd, {bigint: true});
    for (const key of ['dev', 'ino', 'size', 'mode', 'mtimeNs', 'ctimeNs']) assert.equal(after[key], before[key]);
  } finally { closeSync(fd); }
  assert(bytes.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])));
  assert.equal(bytes[4], 2); assert.equal(bytes[5], 1);
  const machine = bytes.readUInt16LE(18);
  assert([62, 183].includes(machine));
  return { architecture: machine === 62 ? 'amd64' : 'arm64', process_architecture: process.arch };
}

export function connect(host, port) {
  return new Promise(resolve => {
    let settled = false;
    const socket = createConnection({host, port});
    const finish = result => { if (!settled) { settled = true; socket.destroy(); resolve(result); } };
    socket.setTimeout(1500, () => finish(false));
    socket.once('connect', () => finish(true)); socket.once('error', () => finish(false));
  });
}

export async function connectivity(control, port) {
  assert(isIPv4(control)); assert(Number.isSafeInteger(port) && port > 0 && port <= 65535);
  const server = createServer(socket => socket.end());
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  let local;
  try { local = await connect('127.0.0.1', server.address().port); }
  finally { await new Promise(resolve => server.close(resolve)); }
  assert(local, 'local network positive control failed');
  const external = await connect(control, port);
  assert(!external, 'disconnected bootstrap control is reachable');
  const resolver = new Resolver({timeout: 1000, tries: 1});
  let resolved = false;
  try { resolved = (await resolver.resolve4('registry-1.docker.io')).length > 0; }
  catch (error) { assert(['ENOTFOUND', 'ETIMEOUT', 'ECONNREFUSED', 'ESERVFAIL', 'EREFUSED', 'ECANCELLED'].includes(error.code)); }
  finally { resolver.cancel(); }
  assert(!resolved, 'external DNS resolution succeeded');
  return { local_tcp: 'passed', bootstrap_tcp: 'blocked', external_dns: 'blocked' };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [operation, ...args] = process.argv.slice(2);
  if (operation === 'namespace' && args.length === 0) console.log(JSON.stringify(inspectNamespace()));
  else if (operation === 'native' && args.length === 0) console.log(JSON.stringify(inspectNativeExecutable('/usr/local/bin/prismpm')));
  else if (operation === 'connectivity' && args.length === 2) console.log(JSON.stringify(await connectivity(args[0], Number(args[1]))));
  else throw Error('usage: sdk-vv-probe.mjs namespace | native | connectivity CONTROL_IPV4 PORT');
}
