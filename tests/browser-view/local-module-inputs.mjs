// Test-only static ESM closure. Parse declarations without executing modules.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {dirname, relative, resolve} from 'node:path';

export function localModuleInputs(repository, entries, readSource) {
  repository = resolve(repository);
  const captured = new Map(), pending = [...entries], environment = {...process.env};
  delete environment.NODE_OPTIONS; delete environment.NODE_PATH;
  const parser = 'import {SourceTextModule} from "node:vm";import {readFileSync} from "node:fs";'
    + 'const source=readFileSync(0,"utf8");const module=new SourceTextModule(source);'
    + 'process.stdout.write(JSON.stringify(module.dependencySpecifiers));';
  while (pending.length) {
    const path = pending.pop();
    assert.equal(typeof path, 'string');
    assert.ok(path.endsWith('.mjs') && path.split('/').every(part => /^[A-Za-z0-9_.-]+$/.test(part)
      && part !== '.' && part !== '..'), 'closed local module path');
    if (captured.has(path)) continue;
    const bytes = readSource(path);
    assert.ok(Buffer.isBuffer(bytes) && bytes.length <= 4194304, 'bounded captured module buffer');
    const source = new TextDecoder('utf-8', {fatal: true}).decode(bytes);
    // These fixed owning modules have no dynamic imports. Fail closed even on
    // lookalike strings/comments; this is not a general JavaScript linker.
    const dynamicImport = new RegExp('\\bim' + 'port(?:\\s|/\\*[\\s\\S]*?\\*/|//[^\\n]*(?:\\n|$))*\\(');
    assert.ok(!dynamicImport.test(source),
      'dynamic imports require a separately registered owning closure');
    const parsed = spawnSync(process.execPath, ['--experimental-vm-modules', '--input-type=module', '-e', parser],
      {input: bytes, encoding: 'utf8', env: environment, timeout: 10000, maxBuffer: 1048576});
    assert.equal(parsed.error, undefined); assert.equal(parsed.signal, null);
    assert.equal(parsed.status, 0, 'actual static ESM parsing: ' + path + '\n' + parsed.stderr);
    const imports = JSON.parse(parsed.stdout);
    assert.ok(Array.isArray(imports) && imports.every(value => typeof value === 'string'));
    captured.set(path, bytes);
    for (const specifier of imports) {
      if (specifier.startsWith('node:')) continue;
      assert.ok(specifier.startsWith('./') || specifier.startsWith('../'), 'no undeclared package or remote import');
      const selected = relative(repository, resolve(repository, dirname(path), specifier));
      assert.ok(!selected.startsWith('../') && selected !== '..' && !selected.startsWith('/'), 'module import cannot escape repository');
      pending.push(selected);
    }
  }
  return new Map([...captured].sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0));
}
