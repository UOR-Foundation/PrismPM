/**
 * The host side of running the SDK image: which program runs Docker here, how
 * a host path is spelled inside a bind mount, and the cache volume a lockfile's
 * image gets.
 *
 * Three ways of reaching a Docker daemon are told apart:
 *
 *   native   `docker` (or `podman`) is an executable on the PATH, and a host
 *            path mounts as itself (Docker Desktop and Linux both accept it).
 *   wsl      Windows without Docker Desktop, where the daemon runs inside a
 *            WSL distribution. The command is `wsl.exe -d <distro> -- docker`,
 *            and a host path `C:\a\b` mounts as `/mnt/c/a/b`, because the
 *            daemon only sees the distribution's filesystem.
 *   missing  nothing answered.
 *
 * `resolveDocker` probes once, and `setup` writes the answer to
 * `$PRISM_HOME/config.json` so every later command spawns the right program
 * without probing. `PRISM_DOCKER` overrides both: a command line such as
 * `wsl.exe -d Ubuntu -- docker`, with `PRISM_DOCKER_MOUNT=wsl` when host
 * paths need the `/mnt/<drive>` spelling.
 *
 * Everything here is a pure function of its context, so the tests give it a
 * `find` and a `run` and never touch this machine.
 */
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { which } from './agents.mjs';
import { fail } from './errors.mjs';

export const HOST_CONFIG_SCHEMA = 'prism/host-config/1';
export const MOUNT_STYLES = Object.freeze(['native', 'wsl']);

/** Split a command line on spaces, honouring double quotes, for `PRISM_DOCKER`. */
export function splitCommandLine(text) {
  const out = [];
  const re = /"([^"]*)"|(\S+)/g;
  for (const m of text.matchAll(re)) out.push(m[1] ?? m[2]);
  return out;
}

/** `C:\a\b` or `C:/a/b` as the WSL daemon sees it: `/mnt/c/a/b`. A path without a drive letter is left alone. */
export function wslMountPath(hostPath) {
  const m = /^([A-Za-z]):[\\/](.*)$/.exec(hostPath);
  if (!m) return hostPath.replace(/\\/g, '/');
  return `/mnt/${m[1].toLowerCase()}/${m[2].replace(/\\/g, '/')}`;
}

/** The host path as the daemon's bind mount source, by mount style. */
export function mountPath(hostPath, style) {
  if (style === 'wsl') return wslMountPath(hostPath);
  if (style === 'native') return hostPath;
  fail('docker.mount', `unknown mount style ${JSON.stringify(style)}; one of ${MOUNT_STYLES.join(', ')}`);
}

/** The cache volume for an SDK image: one per image digest, so two lockfiles never share a cache. */
export function cacheVolumeName(image) {
  const m = /@sha256:([0-9a-f]{64})$/.exec(image);
  if (!m) fail('lock.image', `not an immutable SDK image: ${image}`);
  return `prism-sdk-cache-${m[1].slice(0, 12)}`;
}

const DAEMON_ARGS = ['version', '--format', '{{.Server.Version}}'];

function answers(run, command, args) {
  const result = run(command, args, { encoding: 'utf8', timeout: 20000, windowsHide: true });
  if (result.error || result.status !== 0) return null;
  return String(result.stdout ?? '').trim() || null;
}

/** WSL distributions, from `wsl.exe -l -q`, which prints UTF-16 with a NUL after every character. */
export function parseDistributions(stdout) {
  return String(stdout ?? '')
    .replace(/\0/g, '')
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !/^docker-desktop/i.test(line));
}

/**
 * Find a Docker daemon. Returns `{ kind, command, args, mount, version, distro }`:
 * `command` and `args` are what to spawn before `run ...`, `mount` the mount style.
 * `kind` is `missing` with a `reason` when nothing answered.
 */
export function resolveDocker({ platform = process.platform, env = process.env, find, run = spawnSync } = {}) {
  const lookup = find ?? ((name) => which(name, { env, platform }));
  const override = env.PRISM_DOCKER;
  if (override) {
    const [command, ...args] = splitCommandLine(override);
    const mount = env.PRISM_DOCKER_MOUNT ?? (/\bwsl(\.exe)?$/i.test(command) ? 'wsl' : 'native');
    if (!MOUNT_STYLES.includes(mount)) fail('docker.mount', `PRISM_DOCKER_MOUNT must be one of ${MOUNT_STYLES.join(', ')}; got ${mount}`);
    const version = answers(run, command, [...args, ...DAEMON_ARGS]);
    if (!version) return { kind: 'missing', reason: `PRISM_DOCKER=${override} did not answer docker version` };
    return { kind: 'native', command, args, mount, version, distro: null, source: 'PRISM_DOCKER' };
  }
  for (const name of ['docker', 'podman']) {
    const file = lookup(name);
    if (!file) continue;
    const version = answers(run, file, DAEMON_ARGS);
    if (version) return { kind: 'native', command: file, args: [], mount: 'native', version, distro: null, source: 'PATH' };
  }
  if (platform === 'win32') {
    const wsl = lookup('wsl') ?? lookup('wsl.exe');
    if (wsl) {
      const listed = run(wsl, ['-l', '-q'], { encoding: 'utf8', timeout: 20000, windowsHide: true });
      for (const distro of listed.error ? [] : parseDistributions(listed.stdout)) {
        const args = ['-d', distro, '--', 'docker'];
        const version = answers(run, wsl, [...args, ...DAEMON_ARGS]);
        if (version) return { kind: 'wsl', command: wsl, args, mount: 'wsl', version, distro, source: 'WSL' };
      }
      return { kind: 'missing', reason: 'docker is not on the PATH, and no WSL distribution answered docker version' };
    }
  }
  return { kind: 'missing', reason: 'docker is not on the PATH; prism-code check, build and verify run prismpm inside the SDK image' };
}

/** The host config `setup` wrote, or null. A missing or unreadable file is no config. */
export function readHostConfig(configDir) {
  try {
    const config = JSON.parse(readFileSync(join(configDir, 'config.json'), 'utf8'));
    const docker = config?.docker;
    if (!docker || typeof docker.command !== 'string' || !Array.isArray(docker.args) || !MOUNT_STYLES.includes(docker.mount)) return null;
    return { command: docker.command, args: docker.args.map(String), mount: docker.mount, cache: config.cache === true };
  } catch {
    return null;
  }
}

/** What `config.json` records about Docker, from a resolution. */
export function hostConfigEntry(resolved) {
  return { command: resolved.command, args: resolved.args, mount: resolved.mount };
}

/**
 * The Docker invocation every delegated command uses on this machine: the
 * config `setup` wrote, else a fresh resolution. Throws `docker.missing` when
 * there is none, with the reason.
 */
export function hostDocker({ configDir, env = process.env, platform = process.platform, run = spawnSync, find, probe = true } = {}) {
  const saved = env.PRISM_DOCKER ? null : readHostConfig(configDir);
  if (saved) return { ...saved, kind: saved.mount === 'wsl' ? 'wsl' : 'native', source: 'config' };
  // A dry run prints a command line; it assumes `docker` rather than probing the machine.
  if (!probe) return { kind: 'native', command: 'docker', args: [], mount: 'native', cache: false, source: 'default' };
  const resolved = resolveDocker({ platform, env, find, run });
  if (resolved.kind === 'missing') fail('docker.missing', resolved.reason);
  return { ...resolved, cache: false };
}
