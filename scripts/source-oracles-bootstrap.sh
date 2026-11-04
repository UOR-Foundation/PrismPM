#!/usr/bin/env bash
# Same-source Node dependencies only; no Rust helper or SDK acceptance.
set -euo pipefail
umask 077
unset NODE_PATH NODE_OPTIONS
[[ $# == 3 ]] || { echo "usage: $0 SOURCE_REVISION NODE_ROOT EVIDENCE_DIRECTORY" >&2; exit 64; }
revision=$1
node_root=$2
evidence=$3
[[ $revision =~ ^[0-9a-f]{40}$ && $node_root == /* && $evidence == /* ]] || exit 64
mkdir -p "$evidence"
rm -f "$evidence/node-oracles-path.txt" "$evidence/node-oracles-SHA256SUMS"
exec > "$evidence/node-oracles-bootstrap.log" 2>&1
finish() {
  local status=$?
  if test "$status" != 0; then rm -f "$evidence/node-oracles-path.txt" "$evidence/node-oracles-SHA256SUMS"; fi
  printf '%s\n' "$status" > "$evidence/node-oracles-bootstrap-exit-status.txt"
}
trap finish EXIT
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$root"
printf '%s\n' "$revision" > "$evidence/node-oracles-selected-source.txt"
git rev-parse HEAD > "$evidence/node-oracles-source-revision.txt"
git rev-parse 'HEAD^{tree}' > "$evidence/node-oracles-source-tree.txt"
test "$(cat "$evidence/node-oracles-source-revision.txt")" = "$revision"
source_status=$(git status --porcelain)
test -z "$source_status"
cp tools.lock "$evidence/node-oracles-tools.lock"
python3 - <<'PYTHON'
import json, pathlib, tomllib
assert tomllib.loads(pathlib.Path("tools.lock").read_text())["sdk-tools"]["node"] == "22.23.2", "source Node pin must match the bootstrap"
for name in ("package.json", "package-lock.json"):
    path = pathlib.Path("sdk/oracles") / name
    assert path.is_file() and not path.is_symlink() and path.resolve() == pathlib.Path.cwd() / path, "regular source oracle manifests required"
package = json.loads(pathlib.Path("sdk/oracles/package.json").read_text())
assert package["engines"]["node"] == "22.23.2", "source oracle engine must match the pinned Node"
PYTHON
python3 - "$root" "$node_root" <<'PYTHON'
import os, pathlib, sys
root, target = map(pathlib.Path, sys.argv[1:])
assert target.is_absolute() and os.path.normpath(target) == str(target)
assert target.parent.is_dir() and target.parent.resolve() == target.parent, "canonical oracle target parent required"
assert not target.is_relative_to(root) and not root.is_relative_to(target), "oracle target must be outside the source checkout"
assert not os.path.lexists(target), "fresh private oracle target required"
PYTHON
mkdir "$node_root" "$node_root/.npm-cache"
printf '' > "$node_root/.npm-userconfig"
printf '' > "$node_root/.npm-globalconfig"
cp sdk/oracles/package.json "$evidence/node-oracles-package.json"
cp sdk/oracles/package-lock.json "$evidence/node-oracles-package-lock.json"
cp sdk/oracles/package.json "$node_root/package.json"
cp sdk/oracles/package-lock.json "$node_root/package-lock.json"
node --version > "$evidence/node-version.txt"
test "$(cat "$evidence/node-version.txt")" = v22.23.2
command -v node > "$evidence/node-path.txt"
command -v npm > "$evidence/npm-path.txt"
# Config and cache are private; command-line settings also defeat inherited
# npm_config selectors. Lifecycle scripts cannot run in this acquisition.
npm_private() (
  # Preserve configured network proxy/CA selectors, not ambient npm behavior.
  local variable
  while IFS= read -r variable; do
    case "${variable,,}" in
      npm_config_ca|npm_config_cafile|npm_config_proxy|npm_config_https_proxy|npm_config_noproxy) ;;
      npm_config_*) unset "$variable" ;;
    esac
  done < <(compgen -e)
  command npm --prefix "$node_root" --global=false --userconfig "$node_root/.npm-userconfig" \
    --globalconfig "$node_root/.npm-globalconfig" --cache "$node_root/.npm-cache" "$@"
)
npm_private --version > "$evidence/npm-version.txt"
(cd "$node_root" && npm_private ci --ignore-scripts --no-audit --no-fund)
cmp sdk/oracles/package.json "$evidence/node-oracles-package.json"
cmp sdk/oracles/package-lock.json "$evidence/node-oracles-package-lock.json"
cmp "$node_root/package.json" "$evidence/node-oracles-package.json"
cmp "$node_root/package-lock.json" "$evidence/node-oracles-package-lock.json"
node --input-type=module - "$node_root" > "$evidence/node-oracles-probe.json" <<'JAVASCRIPT'
import assert from 'node:assert/strict';
import {lstatSync, readFileSync, realpathSync} from 'node:fs';
import {createRequire} from 'node:module';
import {join, resolve, sep} from 'node:path';
assert.equal(process.version, 'v22.23.2');
assert.equal(process.env.NODE_PATH, undefined);
assert.equal(process.env.NODE_OPTIONS, undefined);
const root = resolve(process.argv[2]), modules = join(root, 'node_modules');
assert.equal(realpathSync(root), root);
assert(lstatSync(modules).isDirectory() && !lstatSync(modules).isSymbolicLink());
assert.equal(realpathSync(modules), modules);
const require = createRequire(join(root, 'package.json'));
const manifest = JSON.parse(readFileSync(join(root, 'package.json')));
const entries = {};
for (const name of ['js-yaml', 'ajv']) {
  const entry = require.resolve(name), actual = realpathSync(entry);
  assert(actual.startsWith(modules + sep), 'oracle entry must resolve inside private node_modules');
  const packagePath = join(modules, name, 'package.json');
  assert(realpathSync(packagePath).startsWith(modules + sep), 'oracle package must remain private');
  const metadata = JSON.parse(readFileSync(packagePath));
  assert.equal(metadata.name, name); assert.equal(metadata.version, manifest.dependencies[name]);
  entries[name] = {entry: actual, version: metadata.version};
}
const yaml = require('js-yaml'), Ajv = require('ajv');
const ajv2020Entry = realpathSync(require.resolve('ajv/dist/2020.js'));
assert(ajv2020Entry.startsWith(modules + sep), 'Ajv2020 must resolve inside private node_modules');
const Ajv2020 = require('ajv/dist/2020.js');
entries['ajv/dist/2020.js'] = {entry: ajv2020Entry, version: manifest.dependencies.ajv};
const data = yaml.load('ready: true\ncount: 2\n');
assert.deepEqual(data, {ready: true, count: 2});
assert.throws(() => yaml.load('ready: [\n'));
const validate = new Ajv({allErrors: true, strict: true}).compile({type: 'object',
  properties: {ready: {type: 'boolean'}, count: {type: 'integer', minimum: 1}},
  required: ['ready', 'count'], additionalProperties: false});
assert.equal(validate(data), true);
assert.equal(validate({ready: 'yes', count: 0}), false);
assert.equal(validate({...data, extra: true}), false);
const validate2020 = new Ajv2020({allErrors: true, strict: true}).compile({
  $schema: 'https://json-schema.org/draft/2020-12/schema', type: 'array',
  prefixItems: [{type: 'boolean'}, {type: 'integer', minimum: 1}], minItems: 2, items: false});
assert.equal(validate2020([true, 2]), true);
assert.equal(validate2020([true, 0]), false);
assert.equal(validate2020([true, 2, 'extra']), false);
process.stdout.write(JSON.stringify({node: process.version, entries, yaml_probe: true, ajv_probe: true, ajv2020_probe: true}) + '\n');
JAVASCRIPT
# The probes cannot replace either source authority or either exact copy.
source_status=$(git status --porcelain)
test -z "$source_status"
test "$(git rev-parse HEAD)" = "$revision"
test "$(git rev-parse 'HEAD^{tree}')" = "$(cat "$evidence/node-oracles-source-tree.txt")"
cmp tools.lock "$evidence/node-oracles-tools.lock"
cmp sdk/oracles/package.json "$evidence/node-oracles-package.json"
cmp sdk/oracles/package-lock.json "$evidence/node-oracles-package-lock.json"
cmp "$node_root/package.json" "$evidence/node-oracles-package.json"
cmp "$node_root/package-lock.json" "$evidence/node-oracles-package-lock.json"
sha256sum tools.lock sdk/oracles/package.json sdk/oracles/package-lock.json \
  "$node_root/package.json" "$node_root/package-lock.json" \
  "$(cat "$evidence/node-path.txt")" "$(cat "$evidence/npm-path.txt")" "$evidence/node-oracles-probe.json" > "$evidence/node-oracles-SHA256SUMS"
# This receipt authorizes only the declared private Node dependency location.
printf '%s\n' "$node_root/node_modules" > "$evidence/node-oracles-path.txt"
