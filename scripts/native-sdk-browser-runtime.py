"""Hosted-only custody and real native browser execution; never release acceptance."""
import hashlib
import json
import os
import pathlib
import platform
import re
import selectors
import signal
import shutil
import stat
import subprocess
import sys
import time
import urllib.parse
import urllib.request
import uuid
import zipfile

SOURCE = 'f7ec7d7cb868d086e9bfa1aa80a8bf845277bf3e'
RUN = 38197776651
REPO = 'UOR-Foundation/PrismPM'
PINS = {
    'amd64': (11725111972, 'ef4104523910e271266e5d95ed1f996f1a70aaac4e5706ee75743b7e1ec355fb'),
    'arm64': (11725301639, 'efa957991869fc9e11eaa39c4b04ad54be353bb55a9f163dacabed764a240ccf'),
}
ZOT = 'ghcr.io/project-zot/zot@sha256:cd2aea942f428630bcb4190542be6abd35e14177aab84fc7ccad0dca8ecb363d'
SMALL = ['authority-result.json', 'candidate.json', 'cli.json', 'config.json', 'digest.txt',
         'inventory.json', 'manifest.json', 'model-check.json', 'standards.lock', 'tamper.json']
OWNERS = ['scripts/browser-environment-preflight.sh', 'scripts/browser-environment-preflight.mjs',
          'sdk/browser/browser-test-server.mjs', 'sdk/browser-engine/authority.json', 'sdk/browser-engine/install.py']
VERIFIERS = ['scripts/native-sdk-browser-runtime.py', 'scripts/native-sdk-browser-runtime.mjs',
             'scripts/native-sdk-browser-runtime.test.py', '.github/workflows/browser-input-qualification.yml']


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def execute(argv, timeout=600, log=None, limit=16 * 1024 * 1024, on_log_error=None):
    """Bound output before allocation; always reap the actual owned client."""
    process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    captured = bytearray()
    output_bytes = 0
    started = time.monotonic()
    try:
        with selectors.DefaultSelector() as poll:
            poll.register(process.stdout, selectors.EVENT_READ, 'stdout')
            poll.register(process.stderr, selectors.EVENT_READ, 'stderr')
            while poll.get_map():
                require(time.monotonic() - started < timeout, 'bounded command deadline')
                for key, _ in poll.select(0.25):
                    block = os.read(key.fd, 65536)
                    if not block:
                        poll.unregister(key.fileobj)
                        continue
                    output_bytes += len(block)
                    require(output_bytes <= limit, 'bounded command output')
                    if key.data == 'stdout':
                        captured.extend(block)
                    if log:
                        try:
                            log.write(block)
                            log.flush()
                        except OSError as error:
                            if on_log_error is None:
                                raise
                            on_log_error(error)
                            log = None
        status = process.wait(timeout=5)
        require(status == 0, f'command exited {status}: {argv[0:2]!r}; retained {len(captured)} output bytes')
        return bytes(captured).decode('utf-8')
    finally:
        if process.poll() is None:
            process.kill()
        process.wait(timeout=5)
        process.stdout.close()
        process.stderr.close()


def small_json(path):
    require(path.stat().st_size <= 8 * 1024 * 1024, 'bounded JSON record')
    return json.loads(path.read_bytes())


def api(path):
    request = urllib.request.Request('https://api.github.com/' + path,
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'], 'Accept': 'application/vnd.github+json'})
    with urllib.request.urlopen(request, timeout=60) as response:
        raw = response.read(2 * 1024 * 1024 + 1)
    require(len(raw) <= 2 * 1024 * 1024, 'bounded provider metadata')
    return json.loads(raw)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, target):
        return None


def acquire(artifact, archive):
    request = urllib.request.Request(f'https://api.github.com/repos/{REPO}/actions/artifacts/{artifact["id"]}/zip',
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN']})
    try:
        urllib.request.build_opener(NoRedirect).open(request, timeout=60)
        raise ValueError('expected authenticated artifact redirect')
    except urllib.error.HTTPError as error:
        require(error.code == 302, 'artifact redirect status')
        location = error.headers['Location']
    url = urllib.parse.urlsplit(location)
    require(url.scheme == 'https' and not url.username and not url.password and
        url.hostname.endswith('.blob.core.windows.net'), 'artifact storage authority')
    started = time.monotonic()
    count = 0
    # Never forward the API token to signed storage or persist the signed URL.
    with urllib.request.build_opener(NoRedirect).open(location, timeout=120) as response, archive.open('xb') as output:
        require(response.status == 200, 'complete artifact response')
        require(int(response.headers['Content-Length']) == artifact['size_in_bytes'], 'provider length')
        for block in iter(lambda: response.read(1024 * 1024), b''):
            count += len(block)
            require(count <= artifact['size_in_bytes'] and time.monotonic() - started < 900, 'bounded acquisition')
            output.write(block)
    require(count == artifact['size_in_bytes'], 'complete artifact bytes')
    require('sha256:' + sha(archive) == artifact['digest'], 'full provider artifact digest')


def main(architecture, directory):
    require(os.environ.get('GITHUB_REPOSITORY') == REPO and os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted',
        'large SDK acquisition is restricted to GitHub hosted runners')
    require(architecture in PINS and platform.machine() == {'amd64': 'x86_64', 'arm64': 'aarch64'}[architecture],
        'native architecture required')
    work = pathlib.Path(directory)
    work.mkdir(mode=0o700, parents=False, exist_ok=False)
    evidence = work / 'evidence'
    evidence.mkdir()
    require(shutil.disk_usage(work).free >= 35 * 1024 ** 3, '35 GiB hosted storage headroom required')
    source = pathlib.Path.cwd()
    require(subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == os.environ['GITHUB_SHA'],
        'verification source identity')
    require(not subprocess.check_output(['git', 'status', '--porcelain']), 'clean verification source')
    require(not subprocess.check_output(['git', 'diff', SOURCE, '--', *OWNERS]), 'exact original owner source preserved')

    def record(name, value):
        with (evidence / name).open('x') as output:
            json.dump(value, output, sort_keys=True)
            output.write('\n')

    command_count = 0
    cleanup_errors = []
    def command(argv, timeout=600, cleanup=False):
        nonlocal command_count
        command_count += 1
        path = evidence / f'command-{command_count:03d}.log'
        def log_error(error):
            cleanup_errors.append({'resource': str(path), 'error_type': type(error).__name__, 'error': str(error)[:4096]})
        try:
            output = path.open('xb')
        except OSError as error:
            if not cleanup:
                raise
            log_error(error)
            return execute(argv, timeout)
        try:
            return execute(argv, timeout, output, on_log_error=log_error if cleanup else None)
        finally:
            try:
                output.close()
            except OSError as error:
                if not cleanup:
                    raise
                log_error(error)

    run = api(f'repos/{REPO}/actions/runs/{RUN}')
    require(run['head_sha'] == SOURCE and run['status'] == 'completed' and run['conclusion'] == 'success',
        'successful exact SDK construction required')
    record('construction-run.json', run)
    artifact_id, digest = PINS[architecture]
    artifact = api(f'repos/{REPO}/actions/artifacts/{artifact_id}')
    require(artifact['id'] == artifact_id and not artifact['expired'] and
        artifact['name'] == f'unpublished-sdk-construction-{architecture}-{SOURCE}' and
        artifact['digest'] == 'sha256:' + digest and 0 < artifact['size_in_bytes'] <= 6 * 1024 ** 3 and
        artifact['workflow_run']['id'] == RUN and artifact['workflow_run']['head_sha'] == SOURCE,
        'closed artifact identity')
    record('artifact-before.json', artifact)
    archive = work / 'artifact.zip'
    acquire(artifact, archive)
    extracted = work / 'construction'
    extracted.mkdir()
    names = ['construction.json', 'sdk.oci.tar'] + ['evidence/' + name for name in SMALL]
    with zipfile.ZipFile(archive) as payload:
        infos = payload.infolist()
        require(len(infos) == len(names) and sorted(row.filename for row in infos) == sorted(names), 'closed ZIP membership')
        for row in infos:
            mode = row.external_attr >> 16
            require(not row.is_dir() and not row.flag_bits & 1 and
                stat.S_IFMT(mode) in (0, stat.S_IFREG), 'regular unencrypted member')
            require(row.file_size <= (6 * 1024 ** 3 if row.filename == 'sdk.oci.tar' else 8 * 1024 ** 2), 'bounded member')
            path = extracted / row.filename
            path.parent.mkdir(exist_ok=True)
            with payload.open(row) as reader, path.open('xb') as writer:
                shutil.copyfileobj(reader, writer, 1024 * 1024)
            require(path.stat().st_size == row.file_size, 'complete member including ZIP CRC')
    after = api(f'repos/{REPO}/actions/artifacts/{artifact_id}')
    require(after == artifact, 'stable provider artifact metadata')
    record('artifact-after.json', after)
    archive.unlink()  # Exact task-owned, reconstructible verified ZIP; retain all digest evidence.
    construction = small_json(extracted / 'construction.json')
    require(construction['source_revision'] == SOURCE and construction['workflow_revision'] == SOURCE and
        construction['run_id'] == str(RUN) and construction['platform'] == 'linux/' + architecture and
        construction['scope'] == 'construction-only-not-acceptance' and construction['smoke']['status'] == 0,
        'original construction identity and scope')
    oci = extracted / 'sdk.oci.tar'
    require(oci.stat().st_size == construction['archive']['byte_length'] and
        'sha256:' + sha(oci) == construction['archive']['digest'], 'complete original OCI digest')
    for row in construction['smoke_evidence']:
        require(row['path'] in ['evidence/' + name for name in SMALL], 'original evidence path')
        path = extracted / row['path']
        require(path.stat().st_size == row['byte_length'] and 'sha256:' + sha(path) == row['digest'], 'original evidence digest')
    require(len(construction['smoke_evidence']) == 10 and len({row['path'] for row in construction['smoke_evidence']}) == 10,
        'all original construction records')
    record('construction.json', construction)
    tools = work / 'oras'
    command(['bash', 'scripts/sdk-candidate.sh', 'install-oras', str(tools)], 180)
    os.environ['PATH'] = str(tools) + ':' + os.environ['PATH']
    inspect = work / 'inspect'
    command(['bash', 'scripts/sdk-candidate.sh', 'inspect', str(oci), architecture, SOURCE, str(inspect)], 180)
    manifest = (inspect / 'digest.txt').read_text().strip()
    require(manifest == construction['manifest']['digest'], 'original manifest identity')
    require((inspect / 'config.json').read_bytes() == (extracted / 'evidence/config.json').read_bytes(), 'original native config')
    nonce = uuid.uuid4().hex
    registry_name = 'prismpm-readiness-registry-' + nonce
    volume = 'prismpm-readiness-config-' + nonce
    volume_pending = False
    actors = []
    retired = []
    label = 'prismpm.readiness-owner=' + nonce
    primary = None
    readiness = None
    def interrupted(signum, frame):
        raise RuntimeError('qualification interrupted by signal ' + str(signum))
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    try:
        # Register explicit names BEFORE sending potentially ambiguous create requests.
        volume_pending = True
        require(command(['docker', 'volume', 'create', '--label', label, volume]).strip() == volume, 'owned volume response')
        config = work / 'zot.json'
        config.write_text(json.dumps({'distSpecVersion': '1.1.1', 'http': {'address': '0.0.0.0', 'port': 5000},
            'log': {'level': 'warn'}, 'storage': {'rootDirectory': '/tmp/zot'}}))
        actors.append(registry_name)
        registry = command(['docker', 'create', '--name', registry_name, '--label', label, '--publish', '127.0.0.1::5000', '--read-only',
            '--tmpfs', '/tmp:rw,nosuid,nodev', '--volume', volume + ':/config', ZOT, 'serve', '/config/config.json']).strip()
        require(re.fullmatch('[0-9a-f]{64}', registry) is not None, 'actual registry CID')
        command(['docker', 'cp', str(config), registry + ':/config/config.json'])
        command(['docker', 'start', registry])
        endpoint = command(['docker', 'port', registry, '5000/tcp']).strip()
        require(endpoint.startswith('127.0.0.1:') and endpoint.split(':')[1].isdigit(), 'loopback-only registry')
        ready = False
        for _ in range(40):
            try:
                with urllib.request.urlopen('http://' + endpoint + '/v2/', timeout=2) as response:
                    ready = response.status == 200
                if ready:
                    break
            except OSError:
                time.sleep(0.25)
        require(ready, 'owned transport ready')
        command(['oras', 'cp', '--from-oci-layout', '--to-plain-http', str(oci) + '@' + manifest, endpoint + '/sdk:readiness'], 900)
        require(command(['oras', 'resolve', '--plain-http', endpoint + '/sdk:readiness']).strip() == manifest, 'exact transported manifest')
        image = endpoint + '/sdk@' + manifest
        command(['docker', 'pull', '--platform', 'linux/' + architecture, image], 900)
        image_info = json.loads(command(['docker', 'image', 'inspect', image]))[0]
        original_manifest = small_json(inspect / 'manifest.json')
        require(image_info['Id'] == original_manifest['config']['digest'] and image_info['Architecture'] == architecture and
            image_info['Config']['Labels']['org.opencontainers.image.revision'] == SOURCE, 'actual installed native image identity')
        record('image.json', image_info)
        for uid in (0, 1000):
            actor_name = f'prismpm-readiness-browser-{nonce}-{uid}'
            actors.append(actor_name)
            argv = ['docker', 'create', '--name', actor_name, '--label', label, '--platform', 'linux/' + architecture, '--user', f'{uid}:{uid}',
                '--init', '--read-only', '--network', 'none', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges',
                '--memory', '3g', '--memory-swap', '3g', '--cpus', '2', '--shm-size', '256m',
                '--tmpfs', '/tmp:rw,nosuid,nodev,size=1073741824,mode=1777',
                '--tmpfs', '/home/vscode:rw,nosuid,nodev,size=134217728,uid=1000,gid=1000,mode=0700',
                '--tmpfs', '/root:rw,nosuid,nodev,size=134217728,mode=0700',
                '--env', 'XDG_CACHE_HOME=/tmp/browser-cache', '--env', 'XDG_CONFIG_HOME=/tmp/browser-config',
                '--volume', str(source) + ':/review:ro', '--entrypoint', '/usr/bin/timeout', image,
                '--signal=TERM', '--kill-after=5s', '240s', 'bash', '-ec',
                'python3 -B /review/scripts/native-sdk-browser-runtime.test.py; exec node /review/scripts/native-sdk-browser-runtime.mjs']
            actor = command(argv).strip()
            require(re.fullmatch('[0-9a-f]{64}', actor) is not None, 'actual browser CID')
            record(f'uid-{uid}-before.json', json.loads(command(['docker', 'inspect', actor]))[0])
            with (evidence / f'uid-{uid}.log').open('xb') as output:
                attached = execute(['docker', 'start', '--attach', actor], 270, output)
            terminal = json.loads(command(['docker', 'inspect', actor]))[0]
            record(f'uid-{uid}-terminal.json', terminal)
            require(terminal['State']['Status'] == 'exited' and
                terminal['State']['ExitCode'] == 0 and not terminal['State']['OOMKilled'] and not terminal.get('ExecIDs'), 'real browser owner completion')
            report = json.loads(attached.splitlines()[-1])
            expected_inventory = small_json(extracted / 'evidence/inventory.json')
            require(set(report) == {'schema', 'scope', 'uid', 'architecture', 'inventory_sha256', 'commands', 'artifacts',
                'command_result_sha256', 'owners', 'result', 'trees', 'unclaimed'} and
                report['schema'] == 'prismpm/independent-sdk-browser-readiness/1' and report['scope'] == 'browser-readiness-only',
                'closed actual readiness report')
            require(report['inventory_sha256'] == sha(extracted / 'evidence/inventory.json') and
                report['commands'] == len(expected_inventory['commands']) and report['artifacts'] == len(expected_inventory['artifacts']),
                'actual unchanged original installed inventory')
            require(report['owners'] == [{'path': path, 'sha256': sha(source / path)} for path in OWNERS],
                'original immutable installed owner source bindings')
            require(report['uid'] == uid and report['architecture'] == {'amd64': 'x64', 'arm64': 'arm64'}[architecture], 'actual owner identity')
            require(report['result']['engines'] == [{'engine': 'chromium', 'version': '157.0.8099.0', 'evaluated': True},
                {'engine': 'firefox', 'version': '153.0', 'evaluated': True}, {'engine': 'webkit', 'version': '26.5', 'evaluated': True}], 'all original actual engines')
            record(f'uid-{uid}-report.json', report)
        readiness = {'schema': 'prismpm/independent-sdk-native-browser-qualification/1', 'source_revision': SOURCE,
            'verification_revision': os.environ['GITHUB_SHA'], 'construction_run': RUN, 'artifact_id': artifact_id,
            'artifact_digest': artifact['digest'], 'manifest_digest': manifest, 'platform': 'linux/' + architecture,
            'image_config_digest': image_info['Id'], 'original_owners': [{'path': path, 'sha256': sha(source / path)} for path in OWNERS],
            'verification_sources': [{'path': path, 'sha256': sha(source / path)} for path in VERIFIERS],
            'uids': [0, 1000], 'scope': 'installed-native-browser-readiness-only',
            'unclaimed': ['full-sdk-qualification', 'full-vv', 'release', 'product-readiness']}
    except BaseException:
        primary = sys.exc_info()
    finally:
        # Ignore additional termination signals only during this finite reconciliation.
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        signal.signal(signal.SIGINT, signal.SIG_IGN)
        for name in reversed(actors):
            try:
                before = json.loads(command(['docker', 'inspect', name], 30, cleanup=True))[0]
                actor = before['Id']
                require(before['Name'] == '/' + name and re.fullmatch('[0-9a-f]{64}', actor) is not None and
                    before['Config']['Labels'].get('prismpm.readiness-owner') == nonce and
                    not before.get('ExecIDs'), 'exact owned retirement identity')
                if before['State']['Running']:
                    command(['docker', 'stop', '--time', '5', actor], 30, cleanup=True)
                terminal = json.loads(command(['docker', 'inspect', actor], 30, cleanup=True))[0]
                require(not terminal['State']['Running'] and not terminal.get('ExecIDs'), 'terminal actor required')
                command(['docker', 'rm', actor], 30, cleanup=True)
                listed = command(['docker', 'container', 'ls', '--all', '--no-trunc', '--format', '{{.ID}}'], 30, cleanup=True).splitlines()
                require(actor not in listed, 'actual actor retirement')
                retired.append({'name': name, 'before': before, 'terminal': terminal, 'actual_absence': True})
            except BaseException as error:
                cleanup_errors.append({'resource': name, 'error_type': type(error).__name__, 'error': str(error)[:4096]})
        if volume_pending:
            try:
                owned = json.loads(command(['docker', 'volume', 'inspect', volume], 30, cleanup=True))[0]
                require(owned['Name'] == volume and owned['Labels'].get('prismpm.readiness-owner') == nonce, 'exact owned volume')
                command(['docker', 'volume', 'rm', volume], 30, cleanup=True)
                require(volume not in command(['docker', 'volume', 'ls', '--format', '{{.Name}}'], 30, cleanup=True).splitlines(), 'actual volume absence')
            except BaseException as error:
                cleanup_errors.append({'resource': volume, 'error_type': type(error).__name__, 'error': str(error)[:4096]})
        try:
            record('retirement.json', {'retired': retired, 'errors': cleanup_errors,
                'pending_create_absence_is_unknown': True,
                'primary_error': None if primary is None else {'type': primary[0].__name__, 'message': str(primary[1])[:4096]}})
        except OSError as error:
            cleanup_errors.append({'resource': 'retirement.json', 'error_type': type(error).__name__, 'error': str(error)[:4096]})
    if primary:
        raise primary[1].with_traceback(primary[2])
    require(not cleanup_errors and len(retired) == 3 and readiness is not None, 'complete real resource retirement required')
    require(not command(['git', 'status', '--porcelain']), 'final verification source remains clean')
    record('readiness.json', readiness)


if __name__ == '__main__':
    require(len(sys.argv) == 3, 'architecture and fresh evidence directory required')
    main(*sys.argv[1:])
