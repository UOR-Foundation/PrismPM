import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {run, sha, draft, repository} from './compile.mjs';
import {literalLabels} from './label-model.mjs';
const production=name=>['view-host.mjs','view-dom.mjs','view-error.mjs'].includes(name)?join(repository,'sdk/browser',name):join(draft,name);
const sourceClosure = () => Object.fromEntries([
  'view-host.mjs','view-dom.mjs','view-error.mjs','browser-fixture.mjs','browser.mjs',
  '../../sdk/browser/identity.mjs','../../sdk/browser/store.mjs','../../sdk/browser/commands.mjs','../../sdk/browser/queries.mjs','../../sdk/browser/journal.mjs',
  'exception-fixture.mjs',
].map(name => [name,sha(readFileSync(production(name)))]));
export function inputs(view,dependencies) {
  const bytes={View:view.wasmBytes,Command:dependencies.Command.wasmBytes,Query:dependencies.Query.wasmBytes,Journal:dependencies.Query.journalWasm};
  for(const [name,value]of view.sources){const path=join(name.endsWith('.Labels')?draft:join(repository,'stdlib'),'src',...name.split('.'))+'.lex.tex';assert.deepEqual(readFileSync(path),value,'fresh View source '+name);}
  for(const kind of ['Command','Query'])for(const [name,value]of dependencies[kind].sources)assert.deepEqual(readFileSync(join(repository,'stdlib/src/Foundation/Browser/V1',name+'.lex.tex')),value,'fresh dependency source '+name);
  const labels=Buffer.from(literalLabels);assert.equal(run(join(view.work,'native-target/release/browser-workspace-view-runner'),['--labels'],view.runner),literalLabels);
  return {view,dependencies,bytes,labels};
}
async function journey(build, replacements = {}, keyboard = false) {
  return withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage(), errors = [];
    page.on('pageerror', error => errors.push(error.message));
    for (const name of ['view-host.mjs', 'view-dom.mjs', 'view-error.mjs', 'browser-fixture.mjs']) {
      const body = replacements[name] ?? readFileSync(production(name), 'utf8');
      await page.route('**/' + name, route => route.fulfill({status: 200, contentType: 'text/javascript', body}));
    }
    await page.route(baseURL, route => route.fulfill({status:200,headers:{'content-type':'text/html',
      'cross-origin-opener-policy':'same-origin','cross-origin-embedder-policy':'require-corp'},body:'<!doctype html><title>Isolated private View acceptance</title>'}));
    await page.goto(baseURL);
    const result = await page.evaluate(async input => (await import('./browser-fixture.mjs')).runFixture(input),
      {modules: Object.fromEntries(Object.entries(build.bytes).map(([key, value]) => [key, [...value]])), labels: [...build.labels]});
    assert.equal(result.cases.length, 8);
    if (keyboard) {
      const root = page.locator('main').last();
      await root.getByLabel(result.names.workspace, {exact: true}).fill(result.keyboardWorkspace);
      await root.getByLabel(result.names.workspace, {exact: true}).press('Enter');
      await root.locator('[data-slot=status]').filter({hasText: /^Ready$/}).waitFor();
      assert.equal(await root.locator('tbody tr').count(), 1);
      assert.equal(await root.locator('[data-slot=result]').evaluate(node => node === document.activeElement), true);
      await root.getByLabel(result.names.action, {exact: true}).selectOption('4');
      await root.getByLabel(result.names.body, {exact: true}).fill('keyboard message');
      const submit = root.getByRole('button', {name: result.names.submit, exact: true}); await submit.focus(); await submit.press('Enter');
      await root.locator('[data-slot=status]').filter({hasText: /^Refresh required$/}).waitFor();
      assert.equal(await root.getByRole('button', {name: result.names.messages, exact: true}).isDisabled(), true);
      await root.getByRole('button', {name: result.names.refresh, exact: true}).press('Enter');
      await root.locator('[data-slot=status]').filter({hasText: /^Ready$/}).waitFor();
      await root.getByRole('button', {name: result.names.messages, exact: true}).press('Enter');
      await root.locator('tbody tr').waitFor();
      assert.equal(await root.locator('tbody tr td').last().textContent(), 'keyboard message');
      await root.getByRole('button', {name: result.names.close, exact: true}).press('Enter');
      assert.equal(await root.locator('tbody tr').count(), 0);
    }
    Object.assign(result,await page.evaluate(() => globalThis.__viewKeyboardFixture.finish()));
    assert.deepEqual(errors, []); return result;
  });
}
export async function verifyJourneys(t,build) {
  const closure = sourceClosure(), result = await journey(build, {}, true);
  const groups = {View: [], Command: [], Query: []};
  for (const call of result.calls) {
    assert.ok(['View', 'Command', 'Query', 'Journal'].includes(call.kind));
    assert.match(call.request, /^(?:[0-9a-f]{2})*$/); assert.match(call.response, /^(?:[0-9a-f]{2})+$/);
    groups[call.kind === 'Journal' ? 'Query' : call.kind].push(call);
  }
  for (const [kind, calls] of Object.entries(groups)) {
    assert.ok(calls.length > 0); const work = kind === 'View' ? build.view.work : build.dependencies[kind].work;
    const path = join(build.view.work, kind.toLowerCase() + '-transcript.tsv');
    writeFileSync(path, calls.map((call, index) => call.kind + index + '\t' + call.request + '\t' + call.response + '\n').join(''));
    const executable = join(work, 'native-target/release/browser-workspace-' + kind.toLowerCase() + '-runner');
    const output = run(executable, [path], join(work, 'runner'));
    assert.match(output, new RegExp('PASS ' + calls.length + ' complete generated (?:View|command) vectors twice'));
    const changed = join(build.view.work, kind.toLowerCase() + '-transcript-mutant.tsv');
    writeFileSync(changed, calls.map((call, index) => call.kind + index + '\t' + call.request + '\t' + (index === 0 ? 'ff' : call.response) + '\n').join(''));
    assert.throws(() => run(executable, [changed], join(work, 'runner')), /native output|assertion|mismatch/, kind + ' changed observed output must fail actual generated native replay');
  }
  const summary = {cases: result.cases, calls: result.calls.length, maximum: result.maximum,
    source: closure,
    modules: Object.fromEntries(Object.entries(build.bytes).map(([kind, bytes]) => [kind, sha(bytes)]))};
  assert.deepEqual(sourceClosure(),closure,'actual host and primitive source closure remained frozen');
  writeFileSync(join(build.view.work, 'browser-evidence.json'), JSON.stringify(summary, null, 2) + '\n');
  t.diagnostic(JSON.stringify(summary));
}

export async function verifyMutants(t,build) {
  const closure = sourceClosure(), host = readFileSync(production('view-host.mjs'), 'utf8'), dom = readFileSync(production('view-dom.mjs'), 'utf8');
  const mutants = [
    ['BOM silently discarded', 'view-dom.mjs', dom.replace('fatal: true, ignoreBOM: true', 'fatal: true'), /expected private host invalid-labels|admitted message content remains exact text/],
    ['unsafe DOM sink', 'view-dom.mjs', dom.replace('node.textContent = text', 'node.innerHTML = text'), /modeled labels are text, not HTML/],
    ['lost actual refresh barrier', 'view-host.mjs', host.replace('barrier = commands.status().requiresRefresh !== false', 'barrier = false'), /expected private host model-rejected/],
    ['late completion', 'view-host.mjs', host.replace('if (this.#closed || this.#active !== token) return;', '/* planted late completion */'), /host-unavailable/],
    ['delayed public capture', 'view-host.mjs', host.replace('dispatch(value) {', 'async dispatch(value) { await Promise.resolve();'), /synchronous model pending|invalid-input/],
    ['failed terminal rendering stays live', 'view-host.mjs', host.replace('#terminate(failed) {\n    this.#closed = true; this.#active = null;', '#terminate(failed) {\n    /* planted missing terminal state */'), /model-rejected|failed terminal rendering|host-unavailable|view-closed/],
    ['late renderer diagnostic', 'view-dom.mjs', dom.replace('if (!closed) diagnostic.textContent', 'diagnostic.textContent'), /late completion cannot reopen or diagnose closed UI/],
  ];
  for (const [name, path, changed, expected] of mutants) {
    assert.notEqual(changed, path === 'view-dom.mjs' ? dom : host, name);
    await assert.rejects(journey(build, {[path]: changed}), expected, name); t.diagnostic('killed ' + name);
  }
  const duplicate = host.replace('return this.#execute(declared, token);', 'return Promise.all([this.#execute(declared, token), this.#execute(declared, token)]).then(() => undefined);');
  assert.notEqual(duplicate, host); await journey(build, {'view-host.mjs': duplicate});
  const unguarded = duplicate.replace('if (token.started) return;', '/* planted duplicated declared effect */');
  await assert.rejects(journey(build, {'view-host.mjs': unguarded}), /actual stale head rejects revoke without committing|synchronous model pending|actual contributor grant|known modeled role rejection|close never claims rollback|model-rejected|invalid-generated-output/);
  assert.deepEqual(sourceClosure(),closure,'mutants never alter authoritative source closure');
  t.diagnostic('duplicate private dispatch contained; omitted one-shot guard killed');
}

export async function verifyExceptions(t,build) {
  const closure=sourceClosure();
  const journey=replacements=>withBrowser(async({browser,baseURL})=>{
    const page=await browser.newPage();
    for(const name of ['view-host.mjs','view-dom.mjs','view-error.mjs','exception-fixture.mjs'])await page.route('**/'+name,route=>route.fulfill({status:200,contentType:'text/javascript',body:replacements[name]??readFileSync(production(name),'utf8')}));
    await page.goto(baseURL);
    return page.evaluate(async input=>(await import('./exception-fixture.mjs')).exceptions(input),{modules:Object.fromEntries(Object.entries(build.bytes).map(([key,bytes])=>[key,[...bytes]])),labels:[...build.labels]});
  });
  const cases=await journey({});
  assert.deepEqual(cases,['bootstrap-trap','bootstrap-model-rejection','bootstrap-render',
    'dispatch0','dispatch1','dispatch2','dispatch3','completion0','completion1','completion2','completion3']);
  const host=readFileSync(production('view-host.mjs'),'utf8');
  const missingCleanup=host.replace('} catch { throw this.#abort(); }\n  }\n  #promote(raw)',
    "} catch { throw fail('host-unavailable'); }\n  }\n  #promote(raw)");
  assert.notEqual(missingCleanup,host,'actual constructor cleanup mutant');
  await assert.rejects(journey({'view-host.mjs':missingCleanup}),/failed bootstrap clears mounted DOM and detaches listeners/);
  assert.deepEqual(sourceClosure(),closure);t.diagnostic(cases.join(',')+'; killed missing mounted bootstrap cleanup');
}
