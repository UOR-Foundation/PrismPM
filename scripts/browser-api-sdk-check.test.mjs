import assert from 'node:assert/strict';
import {spawn,spawnSync,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync,mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,renameSync,symlinkSync,openSync,closeSync,ftruncateSync,chmodSync,lstatSync,readdirSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,relative,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {test} from 'node:test';
import {capture,verifySource,verifyImage,verifyTap,verifyFileCompletions,runSuites,runStandaloneSuites,sourceRoots,suites,fallbackSuites,standaloneSuites} from './browser-api-sdk-check.mjs';
import * as browserGate from './browser-api-sdk-check.mjs';

const hostModules = ['identity', 'store', 'peer', 'journal', 'commands', 'queries',
 'view-host', 'view-dom', 'view-error', 'rs256', 'effects', 'effects-wire',
 'effects-module', 'presentation-wire', 'presentation-dom', 'semantic-presentation-wire',
 'semantic-presentation-style', 'credential-custody', 'operation-journal','account-genesis','account-genesis-binding','signed-context',
 'session-retention-wire','session-storage','session-payloads','session-operation-capture','pkce'];
const sdkSource = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const additionalOwners = [
 {id:'HO-15',minimum:11,deadline:3600000,files:['tests/holo-primary-component/component.test.mjs','tests/holo-primary-component/owner.test.mjs']},
 {id:'ST-17',minimum:21,deadline:3600000,files:['tests/browser-pkce/guards.test.mjs','tests/browser-pkce/owner.test.mjs']},
 {id:'DK-27',minimum:13,deadline:3600000,files:['sdk/browser/budget-model-test.mjs']},
 {id:'DK-37',minimum:24,deadline:3600000,files:['sdk/browser/contextual-effects.test.mjs']},
 {id:'DK-31',minimum:8,deadline:3600000,files:['tests/browser-dynamic-choice/wire.test.mjs','tests/browser-dynamic-choice/component.test.mjs','sdk/browser/dynamic-choice.test.mjs']},
 {id:'DK-32',minimum:21,deadline:3600000,files:['tests/browser-signed-context/corpus.test.mjs','tests/browser-signed-context/bridge.test.mjs','tests/browser-signed-context/wpt.test.mjs','tests/browser-signed-context/aggregate.test.mjs','sdk/browser/signed-context-test.mjs']},
 {id:'DK-34',minimum:22,deadline:3600000,files:['tests/browser-p256/oracles.test.mjs','tests/browser-p256/corpus.test.mjs','tests/browser-p256/bridge.test.mjs','tests/browser-p256/owner.test.mjs','tests/browser-view/local-module-inputs.test.mjs']},
 {id:'DK-35',minimum:97,deadline:7200000,files:['tests/browser-session-operation/boundary.test.mjs','tests/browser-view/kernel-provenance.test.mjs','tests/browser-view/local-module-inputs.test.mjs','tests/browser-view/file-custody.test.mjs','tests/browser-session-operation/owner.test.mjs']},
];
function registeredAdditionalOwner(source,id) {
 const files=text=>[...text.matchAll(/"([^"]+)"/g)].map(row=>row[1]);
 const direct=new RegExp('"'+id+'"\\s*=>\\s*verify_node_suite\\(\\s*root,\\s*id,\\s*&\\[([\\s\\S]*?)\\],\\s*(\\d+),\\s*"(\\d+)"').exec(source);
 if(direct)return {id,files:files(direct[1]),minimum:Number(direct[2]),deadline:Number(direct[3])};
 const start=source.indexOf('fn verify_browser_host(');assert.ok(start>=0);
 const host=source.slice(start,source.indexOf('\nfn ',start+1));
 const row=new RegExp('"'+id+'"\\s*=>\\s*\\(\\s*&\\[([\\s\\S]*?)\\],\\s*(\\d+)').exec(host);
 assert.ok(row,'actual registered Rust private prerequisite owner '+id);
 const timeout=/let timeout = if matches!\(\s*id,\s*([\s\S]*?)\)\s*\{\s*"(\d+)"\s*\}\s*else if matches!\(\s*id,\s*([\s\S]*?)\)\s*\{\s*"(\d+)"\s*\}\s*else\s*\{\s*"(\d+)"\s*\}/.exec(host);
 assert.ok(timeout,'closed actual Rust owner deadlines');
 assert.match(host,/verify_node_suite\(root, id, files, minimum_tests, timeout\)/);
 return {id,files:files(row[1]),minimum:Number(row[2]),deadline:Number(files(timeout[1]).includes(id)?timeout[2]:files(timeout[3]).includes(id)?timeout[4]:timeout[5])};
}

const independentOwners = [
 {id:'DK-24',construction:'independent',minimum:28,deadline:3600000,files:['tests/browser-operation-journal/fallback-owner.test.mjs']},
 {id:'DK-27',construction:'independent',minimum:12,deadline:3600000,files:['tests/browser-budget/fallback-owner.test.mjs']},
];
function registeredIndependentOwners(source) {
 const start=source.indexOf('_ => panic!("unhandled conformance id: {id}"),');assert.ok(start>=0);
 const tail=source.slice(start,source.indexOf('\nfn verify_browser_host(',start));
 const pattern=/match id \{\s*"(DK-24)" => verify_node_suite\(\s*root,\s*id,\s*&\["([^"]+)"\],\s*(\d+),\s*"(\d+)",\s*\),\s*"(DK-27)" => verify_node_suite\(\s*root,\s*id,\s*&\["([^"]+)"\],\s*(\d+),\s*"(\d+)",\s*\),\s*_ => \{\}\s*\}\s*\}\s*$/;
 const row=pattern.exec(tail);assert.ok(row,'exact separate guarded source fallback calls after original owners');
 return [1,5].map(index=>({id:row[index],construction:'independent',files:[row[index+1]],minimum:Number(row[index+2]),deadline:Number(row[index+3])}));
}

test('independent construction owners are separately registered with unchanged complete files and deadlines',async ()=>{
 assert.deepEqual(fallbackSuites,independentOwners);
 assert.ok(Object.isFrozen(fallbackSuites));
 for(const owner of fallbackSuites){assert.ok(Object.isFrozen(owner));assert.ok(Object.isFrozen(owner.files));}
 const source=sdkSource('crates/conformance/src/cases/mod.rs');
 assert.deepEqual(registeredIndependentOwners(source),independentOwners);
 const start=source.indexOf('_ => panic!("unhandled conformance id: {id}"),'),prefix=source.slice(0,start),tail=source.slice(start);
 for(const owner of independentOwners){
  for(const changed of [tail.replace(owner.id,owner.id==='DK-24'?'DK-25':'DK-26'),
   tail.replace(owner.files[0],owner.files[0]+'.changed'),
   tail.replace('            '+owner.minimum+',','            '+(owner.minimum-1)+','),
   tail.replace(new RegExp('("'+owner.id+'" => verify_node_suite\\([\\s\\S]*?\\],\\s*'+owner.minimum+',\\s*)"3600000"'),'$1"3600001"'),
   tail.replace(new RegExp('"'+owner.id+'" => verify_node_suite\\([\\s\\S]*?\\),'),''),
   tail.replaceAll('DK-24','SWAP').replaceAll('DK-27','DK-24').replaceAll('SWAP','DK-27')])
   assert.throws(()=>assert.deepEqual(registeredIndependentOwners(prefix+changed),independentOwners));
  assert.ok(sourceRoots.some(root=>owner.files[0].startsWith(root+'/')));
  assert.match(sdkSource(owner.files[0]),/timeout:3500000/);
 }
});

test('operation journal source closure binds every static and admitted dynamic module and guest authority',async()=>{
 const {sourceClosure}=await import('../tests/browser-operation-journal/checks.mjs');
 const root=resolve(import.meta.dirname,'..'),snapshot=sourceClosure();
 const paths=new Set(Object.keys(snapshot).map(path=>(!path.includes('/')&&path.endsWith('.mjs')
  ||path==='runner.rs'||path.startsWith('driver/')||path.startsWith('src/'))?'tests/browser-operation-journal/'+path:path));
 assert.equal(paths.size,Object.keys(snapshot).length,'unique normalized journal source inputs');
 for(const path of paths)assert.ok(sourceRoots.some(root=>path===root||path.startsWith(root+'/')),'installed journal input: '+path);
 const required=new Set();let dynamicCount=0;
 const parser='import{SourceTextModule}from"node:vm";import{readFileSync}from"node:fs";'
  +'console.log(JSON.stringify(new SourceTextModule(readFileSync(0,"utf8")).dependencySpecifiers));';
 const routeSource=sdkSource('tests/browser-operation-journal/browser.mjs');
 const routes=/const names = \[([\s\S]*?)\];/.exec(routeSource);assert.ok(routes);
 const routed=[...routes[1].matchAll(/'([^']+)'/g)].map(row=>row[1]);
 assert.deepEqual(routed,['operation-journal.mjs','effects.mjs','effects-wire.mjs','effects-module.mjs',
  'credential-custody.mjs','identity.mjs','store.mjs','browser-fixture.mjs']);
 assert.match(routeSource,/for \(const name of names\) await page\.route/);
 for(const path of paths){
  // Vendored compiler fixtures are pinned input bytes, not imported JS owners.
  if(!path.endsWith('.mjs')||path.startsWith('vendor/'))continue;
  const bytes=readFileSync(join(root,path)),source=bytes.toString('utf8');
  const parsed=spawnSync(process.execPath,['--experimental-vm-modules','--input-type=module','-e',parser],
   {input:bytes,encoding:'utf8',timeout:10000,maxBuffer:1048576,env:{NODE_NO_WARNINGS:'1',TZ:'UTC'}});
  assert.equal(parsed.error,undefined);assert.equal(parsed.signal,null);assert.equal(parsed.status,0,parsed.stderr);
  const dependencies=JSON.parse(parsed.stdout);
  const expressions=[...source.matchAll(/\bimport(?:\s|\/\*[\s\S]*?\*\/|\/\/[^\n]*(?:\n|$))*\(/g)];
  const literals=[...source.matchAll(/\bimport\s*\(\s*(['"])([^'"\n]+)\1\s*\)/g)];
  assert.equal(expressions.length,literals.length,'every dynamic import has a closed literal registration: '+path);
  dynamicCount+=literals.length;
  for(const specifier of [...dependencies,...literals.map(row=>row[2])]){
   if(specifier.startsWith('node:'))continue;
   assert.match(specifier,/^\.{1,2}\//);let selected;
   if(path==='tests/browser-operation-journal/browser-fixture.mjs'){
    assert.ok(specifier.startsWith('./')&&routed.includes(specifier.slice(2)),'exact admitted virtual browser route');
    selected='sdk/browser/'+specifier.slice(2);
   }else if(path==='tests/browser-custody/browser-fixture.mjs'){
    assert.ok(['./credential-custody.mjs','./effects-wire.mjs','./identity.mjs'].includes(specifier),'exact admitted custody browser dependency');
    selected='sdk/browser/'+specifier.slice(2);
   }else selected=relative(root,resolve(root,dirname(path),specifier));
   assert.ok(!selected.startsWith('../')&&!selected.startsWith('/'),'dependency stays in repository');required.add(selected);
  }
 }
 assert.equal(dynamicCount,15,'complete explicit journal and custody dynamic-import inventory');
 for(const file of ['tests/browser-workspace/src/main.rs','tests/browser-journal/driver/src/main.rs',
  'sdk/oracles/package.json','sdk/oracles/package-lock.json','LICENSE-MIT','LICENSE-APACHE',
  'tests/browser-custody/browser.mjs','tests/browser-custody/browser-fixture.mjs'])required.add(file);
 const verify=selected=>{for(const file of required)assert.ok(selected.has(file),'omitted effective journal input: '+file);};
 verify(paths);
 for(const file of required){const omitted=new Set(paths);omitted.delete(file);assert.throws(()=>verify(omitted),/omitted effective journal input/);}
});

test('private prerequisites retain exact registered source files minima deadlines and host inventory',async ()=>{
 const source=sdkSource('crates/conformance/src/cases/mod.rs');
 for(const owner of additionalOwners){
  assert.deepEqual(suites.find(row=>row.id===owner.id),owner,'mandatory installed '+owner.id);
  assert.deepEqual(registeredAdditionalOwner(source,owner.id),owner,'actual source and installed owner agree');
  for(const file of owner.files)assert.ok(sdkSource(file).length,'actual owning file exists');
 }
 assert.deepEqual(browserGate.hostModules,hostModules,'only the exact registered private host modules are installed');
});

test('private prerequisite source closure includes complete fixture directories and shared compiler lifecycle',async ()=>{
 for(const path of ['tests/browser-budget','tests/browser-contextual-effects','tests/browser-dynamic-choice','tests/browser-signed-context','tests/browser-session-operation','tests/browser-pkce','tests/holo-primary-component','tests/browser-view','stdlib/src'])
  assert.ok(sourceRoots.includes(path),'required installed private prerequisite source '+path);
});

test('installed source roots cover every actual frozen private prerequisite compiler input',async()=>{
 for(const path of ['tests/browser-budget/compile.mjs','tests/browser-contextual-effects/checks.mjs','tests/browser-dynamic-choice/compile.mjs','tests/browser-signed-context/compile.mjs','tests/browser-session-operation/compile.mjs','tests/browser-pkce/compile.mjs','tests/holo-primary-component/compile.mjs','tests/browser-presentation/compile.mjs','tests/browser-session/compile.mjs','tests/browser-semantic-presentation/compile.mjs','tests/publication-admission/compile.mjs']){
  const {frozenInputs}=await import(new URL('../'+path,import.meta.url));
  const inputs=frozenInputs();
  if(Object.hasOwn(inputs,'scripts/browser-api-sdk-check.mjs'))
   for(const dependency of ['scripts/sdk-command-owner.mjs','scripts/sdk-command-owner.py','sdk/account-genesis-artifact.mjs'])
    assert(Object.hasOwn(inputs,dependency),'frozen browser helper dependency: '+path+' '+dependency);
  for(const input of Object.keys(inputs))
   assert.ok(sourceRoots.some(root=>input===root||input.startsWith(root+'/')),'unbound actual private prerequisite input '+input);
 }
});

test('private prerequisite comparisons reject actual registered file minimum and deadline substitutions',async ()=>{
 const source=sdkSource('crates/conformance/src/cases/mod.rs');
 for(const owner of additionalOwners){
  assert.deepEqual(registeredAdditionalOwner(source,owner.id),owner);
  const minimum=new RegExp('("'+owner.id+'"\\s*=>\\s*(?:verify_node_suite\\(\\s*root,\\s*id,\\s*|\\(\\s*)&\\[[\\s\\S]*?\\],\\s*)'+owner.minimum+'(\\s*[,)])');
  assert.ok(minimum.test(source),'exact registered minimum for '+owner.id);
  const changedMinimum=source.replace(minimum,(_,before,after)=>before+(owner.minimum-1)+after);
  for(const changed of [source.replace(owner.files[0],owner.files[0]+'.changed'),
   changedMinimum,source.replaceAll('"'+owner.deadline+'"','"'+(owner.deadline-1)+'"')])
   assert.throws(()=>assert.deepEqual(registeredAdditionalOwner(changed,owner.id),owner));
 }
});

test('installed module inventory includes every accepted private browser prerequisite without opening the runtime', async () => {
 assert.deepEqual(browserGate.hostModules, hostModules);
 assert.deepEqual(browserGate.hostArtifacts, ['account-genesis.wasm']);
 const recipe = sdkSource('sdk/Dockerfile');
 const instruction = recipe.split('\n').find(line => line.startsWith('COPY ') && line.endsWith(' /opt/prismpm/browser/'));
 assert.ok(instruction);
 assert.deepEqual([...instruction.matchAll(/\/prepared\/source\/sdk\/browser\/([a-z0-9-]+)\.mjs/g)].map(row => row[1]), hostModules);
 assert.match(instruction, /--from=source_inputs --chmod=0444/);
 const shell = sdkSource('scripts/browser-api-sdk-check.sh');
 assert.match(shell, /node "\$helper" modules "\$root" "\$sdk_work\/browser"/);
 assert.match(shell, /node "\$helper" account-construction "\$sdk_work"/);
 for (const module of hostModules) assert.ok(sdkSource('sdk/browser/' + module + '.mjs').length);
 assert.ok(sdkSource('sdk/generate-inventory.mjs').includes("['browser-host-primitives', 'adapter', '1', '/opt/prismpm/browser', 'tree']"));
 assert.ok(sdkSource('sdk/generate-inventory.mjs').includes("['browser-account-genesis-construction', 'adapter', '1', '/opt/prismpm/share/account-genesis', 'tree']"));
 assert.equal(recipe.split('node sdk/generate-account-genesis.mjs install').length, 2);
 assert.ok(recipe.indexOf('bash scripts/fetch-oracle-cargo.sh') < recipe.indexOf('node sdk/generate-account-genesis.mjs install'));
 assert.ok(recipe.indexOf('node sdk/generate-account-genesis.mjs install') < recipe.indexOf('cp -a . /opt/prismpm/share/conformance-root/'));
 assert.equal(hostModules.filter(name => name === 'operation-journal').length, 1);
});

test('installed browser closure includes complete new owning fixtures and actual Rust refusal owners', async () => {
 for (const path of ['tests/browser-effects', 'tests/browser-presentation', 'tests/browser-custody', 'tests/browser-operation-journal', 'tests/browser-session','tests/browser-account-genesis',
  'sdk/oracles/package.json', 'sdk/oracles/package-lock.json',
  'tests/browser-budget', 'tests/browser-contextual-effects','tests/browser-semantic-presentation','tests/browser-dynamic-choice',
  'tests/browser-session-journal','tests/browser-session-journal-retention','tests/browser-session-journal-recovery',
  'tests/browser-session-payloads','tests/browser-session-operation','tests/browser-session-recovery-frames','tests/publication-context-linkage',
  'tests/fixtures/library/native-library/project', 'tests/support/browser_application.rs',
  'crates/prismpm/src/browser_build.rs', 'crates/prismpm/src/browser_build',
  'crates/prismpm/src/holo/browser_application.rs', 'crates/prismpm/src/holo/browser_application',
  'crates/conformance/tests/conformance.rs', 'crates/conformance/src/cases/browser_compiler.rs',
  'scripts/browser-api-sdk-check.sh', 'scripts/fetch-oracle-cargo.sh', 'sdk/generate-inventory.mjs',
  'sdk/account-genesis-artifact.mjs', 'sdk/generate-account-genesis.mjs']) {
  assert.ok(sourceRoots.includes(path), 'required installed source: ' + path);
 }
 const source = sdkSource('crates/conformance/tests/conformance.rs');
 for (const id of [21, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 37, 38]) assert.ok(source.includes(`test_case!(conformance_dk_${id}, "DK-${id}");`));
 assert.match(sdkSource('crates/prismpm/src/holo/browser_application.rs'), /Err\(PrismError::new\("PP2011"/);
 assert.match(sdkSource('crates/prismpm/src/browser_build/tests.rs'), /assert_eq!\(result.code, "PP2011"\)/);
 const workflow = sdkSource('.github/workflows/release.yml');
 assert.match(workflow, /browser-api-sdk-check\.sh/);
 assert.match(workflow, /bash root-a\/scripts\/sdk-vv-outer\.sh/);
 const launcher = sdkSource('scripts/sdk-vv-outer.sh');
 assert.match(launcher, /node root-a\/scripts\/sdk-vv-check\.mjs tests; node root-a\/scripts\/sdk-vv-check\.mjs run "\$1" "\$2" "\$3" "\$4"/);
 assert.match(launcher, /--target registry_qualification_tools/);
});

test('new installed Node suites retain exact complete owning files and deadlines', async () => {
 const effects = suites.find(row => row.id === 'DK-20'), view = suites.find(row => row.id === 'DK-23');
 assert.deepEqual(effects?.files, ['sdk/browser/effects-wire.test.mjs', 'sdk/browser/effects-module.test.mjs', 'sdk/browser/effects-test.mjs']);
 assert.equal(effects?.minimum, 18);
 assert.deepEqual(view?.files, ['tests/browser-presentation/wire.test.mjs', 'tests/browser-presentation/dom.test.mjs', 'tests/browser-presentation/replay.test.mjs', 'tests/browser-presentation/provenance.test.mjs', 'sdk/browser/presentation.test.mjs']);
 assert.equal(view?.minimum, 39);
 const owning = /"DK-23"\s*=>\s*\(\s*&\[([\s\S]*?)\],\s*(\d+),/.exec(sdkSource('crates/conformance/src/cases/mod.rs'));
 assert.ok(owning, 'actual registered Rust owning suite exists');
 assert.equal(Number(owning[2]), view.minimum, 'installed and source-owning minimum agree');
 assert.deepEqual([...owning[1].matchAll(/"([^"]+)"/g)].map(row => row[1]), view.files);
 assert.equal(effects?.deadline, 3600000); assert.equal(view?.deadline, 3600000);
 const custody = suites.find(row => row.id === 'DK-25');
 assert.deepEqual(custody?.files, ['sdk/browser/credential-custody-test.mjs']);
 assert.equal(custody?.minimum, 11); assert.equal(custody?.deadline, 3600000);
 const semantic = suites.find(row => row.id === 'DK-38');
 assert.deepEqual(semantic?.files, ['tests/browser-semantic-presentation/wire.test.mjs', 'tests/browser-semantic-presentation/dom.test.mjs', 'sdk/browser/semantic-presentation.test.mjs']);
 assert.equal(semantic?.minimum, 29); assert.equal(semantic?.deadline, 3600000);
 const journal = suites.find(row => row.id === 'DK-24');
 assert.deepEqual(journal?.files, ['sdk/browser/operation-journal.test.mjs']);
 assert.equal(journal?.minimum, 28); assert.equal(journal?.deadline, 3600000);
 const session = suites.find(row => row.id === 'DK-26');
 assert.deepEqual(session?.files, ['sdk/browser/session-model-test.mjs','tests/browser-session/wire.test.mjs','tests/browser-session/provenance.test.mjs']);
 assert.equal(session?.minimum, 39); assert.equal(session?.deadline, 3600000);
 const sessionOwner = /"DK-26"\s*=>\s*verify_node_suite\(\s*root,\s*id,\s*&\[([^\]]+)\],\s*(\d+),\s*"([0-9]+)"/.exec(sdkSource('crates/conformance/src/cases/mod.rs'));
 assert.ok(sessionOwner, 'actual complete registered session owner');
 assert.deepEqual([...sessionOwner[1].matchAll(/"([^"]+)"/g)].map(row=>row[1]), session.files);
 assert.equal(Number(sessionOwner[2]), session.minimum); assert.equal(Number(sessionOwner[3]), session.deadline);
});

test('installed session kernel preserves its exact registered owner, complete sources and private scope', async () => {
 const session = suites.find(row => row.id === 'DK-26');
 assert.ok(session, 'installed DK-26 owner is required');
 assert.deepEqual(session.files, ['sdk/browser/session-model-test.mjs', 'tests/browser-session/wire.test.mjs','tests/browser-session/provenance.test.mjs']);
 assert.equal(session.minimum, 39); assert.equal(session.deadline, 3600000);
 const registered = /"DK-26"\s*=>\s*verify_node_suite\(\s*root,\s*id,\s*&\[([\s\S]*?)\],\s*(\d+),\s*"(\d+)"/.exec(sdkSource('crates/conformance/src/cases/mod.rs'));
 assert.ok(registered, 'actual source owner exists');
 assert.deepEqual([...registered[1].matchAll(/"([^"]+)"/g)].map(row => row[1]), session.files);
 assert.equal(Number(registered[2]), session.minimum);
 assert.equal(Number(registered[3]), session.deadline);
 for (const path of ['tests/browser-session', 'LICENSE-MIT', 'LICENSE-APACHE']) {
  assert.ok(sourceRoots.includes(path), 'complete session compiler input: ' + path);
 }
 assert.ok(sdkSource('crates/conformance/tests/conformance.rs').includes('test_case!(conformance_dk_26, "DK-26");'));
 assert.ok(!browserGate.hostModules.includes('session'), 'pure kernel is not an implemented public session host');
});

test('semantic owner registration preserves actual Rust files, minimum and deadline', async () => {
 const semantic = suites.find(row => row.id === 'DK-38');
 assert.ok(semantic); assert.equal(semantic.minimum, 29); assert.equal(semantic.deadline, 3600000);
 const source = sdkSource('crates/conformance/src/cases/mod.rs');
 const owner = /"DK-38"\s*=>\s*\(\s*&\[([\s\S]*?)\],\s*(\d+),/.exec(source);
 assert.ok(owner); assert.equal(Number(owner[2]), semantic.minimum);
 assert.deepEqual([...owner[1].matchAll(/"([^"]+)"/g)].map(row => row[1]), semantic.files);
 assert.deepEqual(semantic.files, ['tests/browser-semantic-presentation/wire.test.mjs',
  'tests/browser-semantic-presentation/dom.test.mjs', 'sdk/browser/semantic-presentation.test.mjs']);
 assert.match(source, /let timeout = if matches!\([\s\S]*?"DK-38"[\s\S]*?3600000/);
 for (const path of ['tests/browser-semantic-presentation', 'LICENSE-MIT', 'LICENSE-APACHE',
  'sdk/oracles/package.json', 'sdk/oracles/package-lock.json']) assert.ok(sourceRoots.includes(path), path);
});

const revision='a'.repeat(40),image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
const temporary=t=>{const root=mkdtempSync(join(tmpdir(),'prismpm-sdk-binding-'));t.after(()=>rmSync(root,{recursive:true,force:true}));return root;};
const put=(root,path,bytes)=>{mkdirSync(dirname(join(root,path)),{recursive:true});writeFileSync(join(root,path),bytes);};
function stageGate(root,helper) {
 put(root,'scripts/browser-api-sdk-check.mjs',helper);
 for(const name of ['browser-api-sdk-check.test.mjs','owning-node-reporter.mjs','sdk-command-owner.mjs','sdk-command-owner.py'])
  put(root,'scripts/'+name,readFileSync(new URL('./'+name,import.meta.url)));
 put(root,'sdk/account-genesis-artifact.mjs',sdkSource('sdk/account-genesis-artifact.mjs'));
 return join(root,'scripts/browser-api-sdk-check.test.mjs');
}
test('browser shell extracts every original read-only tree completely as the nonroot owner and refuses failed producers',t=>{
 assert.equal(typeof process.getuid,'function');assert.ok(process.getuid()>0,'this permission regression must run as the real nonroot owner');
 const shell=sdkSource('scripts/browser-api-sdk-check.sh'),setup=/^set [^\n]+$/m.exec(shell)?.[0];
 assert.equal(setup,'set -euo pipefail');
 const routes=[
  ['$container:/opt/prismpm/share/conformance-root/$path','$sdk_work/source/$(dirname "$path")'],
  ['$container:/opt/prismpm/browser/.','$sdk_work/browser'],
  ['$container:/opt/prismpm/share/account-genesis/.','$sdk_work/share/account-genesis'],
 ];
 const selected=script=>{
  const copies=routes.map(([from,to])=>{
   const lines=script.split('\n').map(line=>line.trim()).filter(line=>line.startsWith(`docker container cp "${from}" `));
   assert.equal(lines.length,1,'one actual installed-tree copy: '+from);
   const line=lines[0],prefix=`docker container cp "${from}" - | `;
   assert.ok(line.startsWith(prefix),'the original Docker copy must produce the archive on stdout: '+from);
   const extract=line.slice(prefix.length);
   assert.equal(extract,`tar --extract --no-same-owner --no-same-permissions --delay-directory-restore --file - --directory "${to}"`,
    'the selected original archive must use the existing owned parent and all nonroot extraction guards');
   return {line,extract,at:script.indexOf(line)};
  });
  const verify='node "$helper" verify "$sdk_work/source" "$sdk_work/source.json"',verified=script.indexOf(verify);
  assert.equal(script.split(verify).length,2,'original complete installed-source verification remains mandatory');
  assert.ok(script.includes('done < <(node "$helper" roots)'),'copy every original registered source root');
  assert.ok(copies[0].at<verified&&verified<copies[1].at,'verify the complete source copy before module checks');
  assert.ok(copies[1].at<script.indexOf('node "$helper" modules "$root" "$sdk_work/browser"'));
  assert.ok(copies[2].at<script.indexOf('node "$helper" account-construction "$sdk_work" "$sdk_work/source.json"'));
  return copies;
 };
 const copies=selected(shell);
 for(const copy of copies){
  assert.throws(()=>selected(shell.replace(copy.line,'true')));
  for(const flag of ['--no-same-owner ','--no-same-permissions ','--delay-directory-restore '])
   assert.throws(()=>selected(shell.replace(copy.line,copy.line.replace(flag,''))));
 }
 assert.throws(()=>selected(shell.replace('node "$helper" verify "$sdk_work/source" "$sdk_work/source.json"','true')));
 const root=mkdtempSync(join(tmpdir(),'prismpm-browser-copy-permissions-'));
 const writable=path=>{
  const s=lstatSync(path);assert.ok(!s.isSymbolicLink());chmodSync(path,(s.mode&0o777)|(s.isDirectory()?0o700:0o600));
  if(s.isDirectory())for(const name of readdirSync(path))writable(join(path,name));
 };
 t.after(()=>{writable(root);rmSync(root,{recursive:true,force:true});});
 const inventory=directory=>{
  const rows=[];const walk=(path,name)=>{
   const s=lstatSync(path);assert.ok(!s.isSymbolicLink());assert.equal(s.uid,process.getuid());assert.equal(s.gid,process.getgid());
   if(s.isDirectory()){rows.push({path:name,kind:'directory'});for(const child of readdirSync(path).sort())walk(join(path,child),name?name+'/'+child:child);}
   else{assert.ok(s.isFile());rows.push({path:name,kind:'file',bytes:readFileSync(path).toString('hex')});}
  };walk(directory,'');return rows;
 };
 const tarVersion=spawnSync('tar',['--version'],{encoding:'utf8',timeout:5000,maxBuffer:1048576});
 assert.equal(tarVersion.error,undefined);assert.equal(tarVersion.status,0);assert.match(tarVersion.stdout,/^tar \(GNU tar\)/);
 for(const [index,copy]of copies.entries()){
  const parent=join(root,'input-'+index),entry=index===0?'.cargo':'.',input=index===0?join(parent,entry):parent;
  mkdirSync(join(input,'nested','empty-directory'),{recursive:true});
  writeFileSync(join(input,'config.toml'),'[build]\n# complete original copy\n');
  writeFileSync(join(input,'nested','binary'),Buffer.from([0,255,128,10,0,42]));writeFileSync(join(input,'nested','empty-file'),Buffer.alloc(0));
  const expected=inventory(input);
  for(const path of [input,join(input,'nested'),join(input,'nested','empty-directory')])chmodSync(path,0o555);
  for(const path of [join(input,'config.toml'),join(input,'nested','binary'),join(input,'nested','empty-file')])chmodSync(path,0o444);
  const archive=spawnSync('tar',['--create','--format=gnu','--numeric-owner','--owner=0','--group=0','-C',parent,'-f','-',entry],
   {timeout:5000,maxBuffer:1048576,env:{PATH:process.env.PATH,LC_ALL:'C'}});
  assert.equal(archive.error,undefined);assert.equal(archive.signal,null);assert.equal(archive.status,0,archive.stderr.toString());
  const execute=(producer,options=setup)=>{
   const work=mkdtempSync(join(root,'output-'));
   for(const path of ['source','browser','share/account-genesis'])mkdirSync(join(work,path),{recursive:true});
   const result=spawnSync('bash',['--noprofile','--norc','-c',options+'\n'+producer+' | '+copy.extract],
    {input:archive.stdout,timeout:5000,maxBuffer:1048576,env:{PATH:process.env.PATH,LC_ALL:'C',sdk_work:work,path:'.cargo'}});
   assert.equal(result.error,undefined);assert.equal(result.signal,null);
   const destination=join(work,index===0?'source/.cargo':index===1?'browser':'share/account-genesis');
   assert.deepEqual(inventory(destination),expected,'all nested, empty and binary members survive the actual selected extractor');
   assert.deepEqual(inventory(input),expected,'the original read-only fixture is never changed');return result;
  };
  const good=execute('cat');assert.equal(good.status,0,good.stderr.toString());
  const refuses=result=>assert.equal(result.status,17,'the original archive producer failure must survive successful extraction');
  refuses(execute('{ cat; exit 17; }'));
  const omitted=execute('{ cat; exit 17; }',setup.replace(' pipefail',''));assert.equal(omitted.status,0);assert.throws(()=>refuses(omitted));
 }
});
test('actual installed module trees reject missing extra changed and aliased module bytes', async t => {
 assert.equal(typeof browserGate.verifyModules, 'function');
 const source = temporary(t), installed = temporary(t);
 for (const module of hostModules) {
  put(source, 'sdk/browser/' + module + '.mjs', module);
  put(installed, module + '.mjs', module);
 }
 for (const artifact of browserGate.hostArtifacts) {
  put(source, 'sdk/browser/' + artifact, Uint8Array.of(0, 97, 115, 109));
  put(installed, artifact, Uint8Array.of(0, 97, 115, 109));
 }
 browserGate.verifyModules(source, installed);
 const path = join(installed, 'effects.mjs');
 for (const kind of ['missing', 'changed', 'extra', 'alias', 'fifo', 'directory']) {
  rmSync(path);
  if (kind === 'changed') put(installed, 'effects.mjs', 'changed');
  if (kind === 'extra') { put(installed, 'effects.mjs', 'effects'); put(installed, 'extra.mjs', 'extra'); }
  if (kind === 'alias') symlinkSync('identity.mjs', path);
  if (kind === 'fifo') execFileSync('mkfifo', [path]);
  if (kind === 'directory') mkdirSync(path);
  assert.throws(() => browserGate.verifyModules(source, installed), undefined, kind);
  rmSync(path, {force: true, recursive: kind === 'directory'}); rmSync(join(installed, 'extra.mjs'), {force: true});
  put(installed, 'effects.mjs', 'effects'); browserGate.verifyModules(source, installed);
 }
 const alias = join(temporary(t), 'alias'); symlinkSync(installed, alias);
 assert.throws(() => browserGate.verifyModules(source, alias));
 const artifact = join(installed, 'account-genesis.wasm'), bytes = readFileSync(artifact);
 for (const kind of ['missing', 'changed', 'alias', 'extra']) {
  rmSync(artifact);
  if (kind === 'changed') put(installed, 'account-genesis.wasm', Uint8Array.of(1, 2, 3));
  if (kind === 'alias') symlinkSync('identity.mjs', artifact);
  if (kind === 'extra') {put(installed, 'account-genesis.wasm', bytes); put(installed, 'foreign.wasm', bytes);}
  assert.throws(() => browserGate.verifyModules(source, installed), undefined, 'generated artifact ' + kind);
  rmSync(artifact, {force: true}); rmSync(join(installed, 'foreign.wasm'), {force: true});
  put(installed, 'account-genesis.wasm', bytes); browserGate.verifyModules(source, installed);
 }
});
function source(root){
 for(const path of sourceRoots){
  if(path!=='.cargo'&&path.includes('.')&&!path.endsWith('/rust')||['lean-toolchain','LICENSE-MIT','LICENSE-APACHE','sdk/Dockerfile'].includes(path))put(root,path,path+'\n');
  else put(root,path+'/source.txt',path+'\n');
 }
}
function inspected(){return[{Os:'linux',Architecture:'amd64',RepoDigests:[image],Config:{Entrypoint:['/usr/local/bin/prismpm-devcontainer-init'],Volumes:null,Labels:{
 'org.opencontainers.image.revision':revision,
 'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM',
 'org.opencontainers.image.version':'0.3.0',
}}}];}

test('current SDK source closure binds helper, compiler, suite and every selected byte',async t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);verifySource(root,expected);
 for(const path of['scripts/browser-api-sdk-check.mjs','LICENSE-MIT','LICENSE-APACHE','sdk/browser/source.txt','vendor/lexlean/source.txt','tests/browser-api/source.txt',
  'tests/browser-effects/source.txt','tests/browser-presentation/source.txt','tests/browser-custody/source.txt',
  'tests/browser-operation-journal/source.txt','tests/support/browser_application.rs',
  'sdk/oracles/package.json','sdk/oracles/package-lock.json',
  'tests/browser-session/source.txt','tests/browser-budget/source.txt','tests/browser-contextual-effects/source.txt',
  'crates/prismpm/src/browser_build.rs','scripts/fetch-oracle-cargo.sh']){
  const bytes=readFileSync(join(root,path));put(root,path,Buffer.concat([bytes,Buffer.from('x')]));assert.throws(()=>verifySource(root,expected));put(root,path,bytes);
 }
 for(const path of ['sdk/oracles/package.json','sdk/oracles/package-lock.json']){
  assert.ok(expected.files.some(row=>row.path===path),'actual oracle dependency input captured: '+path);
  const omitted=structuredClone(expected);omitted.files=omitted.files.filter(row=>row.path!==path);
  assert.throws(()=>verifySource(root,omitted),/exact current source/,'omitted oracle dependency input: '+path);
 }
 for(const mutate of[
  value=>value.files.pop(),value=>value.files.push(value.files[0]),value=>value.extra=true,
  value=>value.revision=[revision],value=>value.files[0].sha256=[value.files[0].sha256],
  value=>value.files[0].extra=true,value=>value.files.reverse(),value=>value.files[0].size++,
 ]){const changed=structuredClone(expected);mutate(changed);assert.throws(()=>verifySource(root,changed));}
 put(root,'tests/browser-api/nested/target/ordinary','keep');assert.throws(()=>verifySource(root,expected));
});

test('source binding rejects missing, aliased, nonregular and oversized inputs',async t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);
 const path=join(root,'sdk/browser/source.txt'),saved=readFileSync(path);rmSync(path);assert.throws(()=>verifySource(root,expected));
 symlinkSync('../stdlib-sources.tar',path);assert.throws(()=>capture(root,revision));rmSync(path);put(root,'sdk/browser/source.txt',saved);
 renameSync(join(root,'stdlib'),join(root,'stdlib-real'));symlinkSync('stdlib-real',join(root,'stdlib'));assert.throws(()=>capture(root,revision),/parent alias/);rmSync(join(root,'stdlib'));renameSync(join(root,'stdlib-real'),join(root,'stdlib'));
 rmSync(path);execFileSync('mkfifo',[path]);assert.throws(()=>capture(root,revision),/bounded regular/);rmSync(path);
 const fd=openSync(path,'wx');try{ftruncateSync(fd,64*1024*1024+1);}finally{closeSync(fd);}assert.throws(()=>capture(root,revision),/bounded regular/);
});

test('SDK identity is immutable, exact-source and native-platform bound',async ()=>{
 verifyImage(inspected(),image,'amd64',revision);
 for(const mutate of[
  value=>value[0].Os='windows',value=>value[0].Architecture='arm64',value=>value[0].RepoDigests=[],value=>value[0].RepoDigests=image,
  value=>value[0].Config.Entrypoint=['/bin/true'],value=>value[0].Config.Volumes={'/opt/prismpm':{}},value=>value[0].Config.Volumes='',
  value=>value[0].Config.Labels['org.opencontainers.image.revision']='c'.repeat(40),
  value=>value[0].Config.Labels['org.opencontainers.image.source']='https://example.invalid',
  value=>value[0].Config.Labels['org.opencontainers.image.version']='0.2.0',
  value=>value.push(value[0]),
 ]){const changed=inspected();mutate(changed);assert.throws(()=>verifyImage(changed,image,'amd64',revision));}
 assert.throws(()=>verifyImage(inspected(),'ghcr.io/uor-foundation/prismpm-sdk:latest','amd64',revision));
 assert.throws(()=>verifyImage(inspected(),image,'amd64',[revision]));
});

const testSource=(count,skip=false)=>"import {test} from 'node:test';\n"+Array.from({length:count},(_,index)=>`test('case ${index}',${skip&&index===0?'{skip:true},':''}()=>{});\n`).join('');
function testFixtures(root){for(const suite of [...suites,...fallbackSuites,...standaloneSuites])for(const [index,file] of suite.files.entries())put(root,file,testSource(suite.exactTests?.[index]??suite.minimum));}

test('standalone registry retains all six original inventories and distinct deadlines',async ()=>{
 const expected=[
  ['Session Projection',3600000,['tests/browser-session-journal/projection.test.mjs','tests/browser-session-journal/projection-owner.test.mjs'],[7,1]],
  ['Session Reservation',3600000,['tests/browser-session-journal/reservation.test.mjs','tests/browser-session-journal/reservation-owner.test.mjs'],[10,23]],
  ['Session Payload',3600000,['tests/browser-session-payloads/component.test.mjs','tests/browser-session-payloads/owner.test.mjs'],[4,30]],
  ['Session Recovery Frames',3600000,['tests/browser-session-recovery-frames/wire.test.mjs','tests/browser-session-recovery-frames/owner.test.mjs'],[3,1]],
  ['Session Journal Recovery',5500000,['tests/browser-session-journal-recovery/component.test.mjs','tests/browser-session-journal-recovery/owner.test.mjs'],[7,1]],
  ['Session Retention',3600000,['tests/browser-session-journal-retention/component.test.mjs','tests/browser-session-journal-retention/owner.test.mjs'],[4,18]],
 ];
 assert.deepEqual(standaloneSuites.map(row=>[row.id,row.deadline,row.files,row.exactTests]),expected);
 assert(Object.isFrozen(standaloneSuites));
 for(const row of standaloneSuites){
  assert(Object.isFrozen(row)&&Object.isFrozen(row.files)&&Object.isFrozen(row.exactTests));
  assert.equal(row.minimum,row.exactTests.reduce((a,b)=>a+b,0));
  for(const file of row.files)assert(sourceRoots.some(path=>file.startsWith(path+'/')));
  assert.match(sdkSource(row.files[1]),row.id==='Session Journal Recovery'?/timeout: 5400000/:/timeout:\s*3500000/);
 }
 assert.equal(standaloneSuites.reduce((sum,row)=>sum+row.minimum,0),109);
 assert.equal(suites.length,26);assert.deepEqual(fallbackSuites,independentOwners);
});

test('complete source gates register standalone execution and all original product boundary units',async ()=>{
 const source=sdkSource('xtask/src/main.rs');
 const vv=source.slice(source.indexOf('fn run_vv('),source.indexOf('fn verify_examples('));
 assert.equal(vv.split('verify_standalone_browser_owners(root)?;').length,2);
 assert(vv.indexOf('verify_standalone_browser_owners(root)?;')>vv.indexOf('VV gate 6/15:'));
 assert(vv.indexOf('verify_standalone_browser_owners(root)?;')<vv.indexOf('VV gate 7/15:'));
 const conformance=source.slice(source.indexOf('fn run_conformance('),source.indexOf('fn run_unit_tests('));
 assert.match(conformance,/if matches!\(category, "all" \| "desktop-browser"\) \{\s*verify_standalone_browser_owners\(root\)\?;/);
 assert.match(conformance,/fn verify_standalone_browser_owners\(root: &Path\) -> Result<\(\), Fail> \{\s*command\(\s*root,\s*"node",\s*&\["scripts\/browser-api-sdk-check.mjs", "standalone"\],\s*\)/);
 const audit=source.slice(source.indexOf('fn audit_all('),source.indexOf('fn command('));
 assert.equal(audit.split('"scripts/product-sdk-check.test.mjs"').length,2);
 assert.equal((sdkSource('scripts/product-sdk-check.test.mjs').match(/^test\(/gm)??[]).length,19);
 assert(sourceRoots.includes('xtask/src/main.rs'));
});

test('standalone execution invokes every full file pair with its exact process and file bounds',async t=>{
 const root=temporary(t);testFixtures(root);const calls=[];
 const launch=(program,args,options)=>{calls.push({program,args,options});return spawnSync(program,args,options);};
 const completed=(await runStandaloneSuites(root,launch,()=>{}));
 assert.deepEqual(completed,standaloneSuites.map(row=>({id:row.id,tests:row.minimum})));
 assert.equal(calls.length,6);
 for(const [index,call] of calls.entries()){
  const row=standaloneSuites[index];assert.equal(call.program,process.execPath);
  assert.deepEqual(call.args.slice(0,2),['--test','--test-concurrency=1']);
  assert(call.args[2].startsWith('--test-reporter=data:text/javascript;base64,'));
  assert.equal(call.args[3],'--test-timeout='+row.deadline);assert.deepEqual(call.args.slice(4),row.files);
  assert.equal(call.options.timeout,row.deadline);assert.equal(call.options.maxBuffer,64*1024*1024);
  assert.equal(call.options.cwd,root);assert.equal(call.options.env.NODE_TEST_CONTEXT,undefined);
  assert.equal(call.options.owned,true);
 }
});

test('standalone default launch owns all six exact file pairs without accepting a fixture as application evidence',async t=>{
 const root=temporary(t);testFixtures(root);
 assert.deepEqual(await runStandaloneSuites(root,undefined,()=>{}),standaloneSuites.map(row=>({id:row.id,tests:row.minimum})));
 for(const file of ['scripts/sdk-command-owner.mjs','scripts/sdk-command-owner.py'])assert(sourceRoots.includes(file));
});

test('standalone launch rejects zero-exit TAP with stubborn escaped trailing work and proves retirement',async t=>{
 const root=mkdtempSync(join(tmpdir(),'prismpm-owned-standalone-'));testFixtures(root);
 const marker=join(root,'escaped.json'),termed=join(root,'termed.json');
 const token='original-standalone-control-'+process.pid;
 t.after(async()=>{
  try{
   if(!existsSync(marker))return;
   const row=JSON.parse(readFileSync(marker));assert.equal(row.token,token);
   assert(Number.isSafeInteger(row.pid)&&row.pid>1);
   if(!existsSync('/proc/'+row.pid))return;
   const argv=readFileSync('/proc/'+row.pid+'/cmdline','utf8').split('\0');
   assert(argv.includes(marker)&&argv.includes(token),'only identified isolated test work may be cleaned');
   process.kill(row.pid,'SIGKILL');const deadline=performance.now()+5000;
   while(existsSync('/proc/'+row.pid)&&performance.now()<deadline)await new Promise(resolve=>setTimeout(resolve,10));
   assert(!existsSync('/proc/'+row.pid),'bounded safety cleanup cannot repair failed retirement proof');
  }finally{rmSync(root,{recursive:true,force:true});}
 });
 const child="const fs=require('node:fs');fs.writeFileSync(process.argv[1],JSON.stringify({pid:process.pid,token:process.argv[2]}),{flag:'wx'});"
  +"process.on('SIGTERM',()=>fs.writeFileSync(process.argv[3],JSON.stringify({pid:process.pid,token:process.argv[2]})));setInterval(()=>{},1000);";
 const file=standaloneSuites[0].files[0];
 put(root,file,"import {spawn} from 'node:child_process';import {existsSync} from 'node:fs';import {test} from 'node:test';\n"
  +testSource(7).split('\n').slice(1).join('\n')
  +`const child=spawn(process.execPath,['-e',${JSON.stringify(child)},${JSON.stringify(marker)},${JSON.stringify(token)},${JSON.stringify(termed)}],{detached:true,stdio:'ignore'});child.unref();\n`
  +`while(!existsSync(${JSON.stringify(marker)}))await new Promise(resolve=>setTimeout(resolve,5));\n`);
 // Only this isolated controller fixture shortens the clock. The production
 // registry and all original application/file deadlines remain unchanged.
 const original=sdkSource('scripts/browser-api-sdk-check.mjs');
 const clock='const deadline=execution.deadline;';assert.equal(original.split(clock).length,2);
 stageGate(root,original.replace(clock,'const deadline=1000;'));
 put(root,'scripts/owning-node-reporter.mjs',sdkSource('scripts/owning-node-reporter.mjs')+"\nprocess.stderr.write('OWNED-STANDALONE-STDERR\\n');\n");
 const run=(await import(pathToFileURL(join(root,'scripts/browser-api-sdk-check.mjs')).href)).runStandaloneSuites;
 const start=performance.now(),logs=[];let failure;
 await assert.rejects(()=>run(root,undefined,text=>logs.push(text)),error=>{
  failure=error;
  assert.match(error.message,/standalone timedOut/);assert.equal(error.result.status,0);
  assert.equal(error.result.retirement.descendants_absent,true);return true;
 });
 assert.equal(failure.result.stderr.toString(),'OWNED-STANDALONE-STDERR\n');
 assert.equal(logs.join(''),'SDK browser suite Session Projection\n'+failure.result.stdout+failure.result.stderr,
  'complete actual failed stdout and stderr remain diagnostic, never acceptance');
 assert(performance.now()-start<6500,'original deadline plus bounded retirement, not SIGTERM-only waiting');
 const row=JSON.parse(readFileSync(marker));assert.equal(row.token,token);
 assert.deepEqual(JSON.parse(readFileSync(termed)),row,'actual TERM precedes forced retirement');
 assert.throws(()=>readFileSync('/proc/'+row.pid+'/stat'),{code:'ENOENT'},'owned escaped child absent before safety cleanup');
});

test('standalone escaped-work oracle kills removal of the actual owned launch path',async t=>{
 const original=sdkSource('scripts/browser-api-sdk-check.mjs');
 const guard='if(!options.owned)return spawnSync(program,args,options);';
 assert.equal(original.split(guard).length,2);
 const root=temporary(t),selected=stageGate(root,original.replace(guard,'if(true)return spawnSync(program,args,options);'));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const result=spawnSync(process.execPath,['--test','--test-name-pattern=^standalone launch rejects',selected],{encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.status,1);assert.match(result.stdout,/Missing expected rejection/);
});

test('standalone preflight rejects absent aliased and nonregular last-group files before any dispatch',async t=>{
 const root=temporary(t);testFixtures(root);const file=standaloneSuites.at(-1).files[1],path=join(root,file);
 for(const defect of ['missing','alias','directory']){
  rmSync(path,{recursive:true});
  if(defect==='alias')symlinkSync('missing.test.mjs',path);
  if(defect==='directory')mkdirSync(path);
  let calls=0;
  await assert.rejects(()=>runStandaloneSuites(root,()=>{calls++;throw Error('must not dispatch');},()=>{}));
  assert.equal(calls,0);rmSync(path,{recursive:true,force:true});put(root,file,testSource(18));
 }
});

test('standalone file counts reject coherent redistribution even when the aggregate still passes',async t=>{
 const root=temporary(t);testFixtures(root);
 for(const row of standaloneSuites){
  const [first,second]=row.files,[one,two]=row.exactTests;
  put(root,first,testSource(one-1));put(root,second,testSource(two+1));
  await assert.rejects(()=>runStandaloneSuites(root,spawnSync,()=>{}),/exact original standalone file test count/);
  put(root,first,testSource(one));put(root,second,testSource(two));
 }
 assert.equal((await runStandaloneSuites(root,spawnSync,()=>{})).reduce((sum,row)=>sum+row.tests,0),109);
});

test('standalone unsuccessful original file completion cannot satisfy a passing aggregate',async t=>{
 const root=temporary(t);testFixtures(root);const row=standaloneSuites[0],file=row.files[0];
 put(root,file,testSource(row.exactTests[0],true));
 await assert.rejects(()=>runStandaloneSuites(root,spawnSync,()=>{}),/incomplete pass set|skipped/);
 put(root,file,testSource(row.exactTests[0]));
 for(const defect of ['missing','duplicate','cancelled','wrong-file']){
  const launch=(program,args,options)=>{
   const result=spawnSync(program,args,options);
   if(args.includes(file)){
    const prefix='# prismpm-owning-file ',line=result.stdout.split('\n').find(line=>line.startsWith(prefix));
    if(defect==='missing')result.stdout=result.stdout.replace(line+'\n','');
    if(defect==='duplicate')result.stdout+=line+'\n';
    if(defect==='cancelled')result.stdout=result.stdout.replace('# cancelled 0','# cancelled 1');
    if(defect==='wrong-file')result.stdout=result.stdout.replaceAll(join(root,file),join(root,file)+'.wrong');
   }
   return result;
  };
  await assert.rejects(()=>runStandaloneSuites(root,launch,()=>{}),undefined,defect);
 }
});

test('standalone owning inventory controls kill actual source count deadline and file omissions',async t=>{
 const original=sdkSource('scripts/browser-api-sdk-check.mjs');
 for(const [before,after] of [
  ["id:'Session Projection',deadline:3600000","id:'Session Projection',deadline:3600001"],
  ['exactTests:[7,1]','exactTests:[6,1]'],
  ['tests/browser-session-journal/projection-owner.test.mjs','tests/browser-session-journal/projection-owner.test.mjs.changed'],
  ["id:'Session Retention'","id:'Session Retention omitted'"],
 ]){
  assert(original.includes(before));const root=temporary(t),selected=stageGate(root,original.replace(before,after));
  const env={...process.env};delete env.NODE_TEST_CONTEXT;
  const result=spawnSync(process.execPath,['--test','--test-name-pattern=standalone registry retains',selected],{encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});
  assert.equal(result.error,undefined);assert.equal(result.status,1);assert.match(result.stdout,/AssertionError/);
 }
});

test('standalone coherent-count oracle kills removal of the actual per-file exact-count guard',async t=>{
 const original=sdkSource('scripts/browser-api-sdk-check.mjs');
 const guard="if(exactTests!==undefined)assert.equal(row.tests,exactTests[files.indexOf(row.file)],'exact original standalone file test count');";
 assert.equal(original.split(guard).length,2);
 const root=temporary(t),selected=stageGate(root,original.replace(guard,''));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const result=spawnSync(process.execPath,['--test','--test-name-pattern=standalone file counts reject',selected],{encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.status,1);assert.match(result.stdout,/Missing expected rejection/);
});

for(const owner of independentOwners)test('independent '+owner.id+' execution refuses missing aliased incomplete and forged completion',async t=>{
 const root=temporary(t);testFixtures(root);const file=owner.files[0],path=join(root,file);
 for(const defect of ['missing','alias','empty','under-count','skip','failure','cancelled','wrong-file','forged']){
  put(root,file,testSource(owner.minimum));let calls=0;
  if(defect==='missing')rmSync(path);
  if(defect==='alias'){rmSync(path);symlinkSync('does-not-exist.mjs',path);}
  if(defect==='empty')put(root,file,'');
  if(defect==='under-count')put(root,file,testSource(owner.minimum-1));
  if(defect==='skip')put(root,file,testSource(owner.minimum,true));
  if(defect==='failure')put(root,file,"import test from 'node:test';test('actual failure',()=>{throw Error('deliberate');});");
  if(defect==='forged')put(root,file,"console.log('# prismpm-owning-file '+JSON.stringify({file:import.meta.filename,tests:28,passed:28}));");
  const launch=(program,args,options)=>{
   calls++;const result=spawnSync(program,args,options);
   if(args.includes(file)&&defect==='cancelled')result.stdout=result.stdout.replace('# cancelled 0','# cancelled 1');
   if(args.includes(file)&&defect==='wrong-file')result.stdout=result.stdout.replaceAll(path,path+'.wrong');
   return result;
  };
  await assert.rejects(()=>runSuites(root,launch,()=>{}),undefined,defect);
  if(['missing','alias'].includes(defect))assert.equal(calls,0,'all constructions preflight before dispatch');
  rmSync(path,{force:true});
 }
});

test('installed DK23 gate requires every wire, DOM, replay, provenance and full-owner check', async t => {
 const root = temporary(t); testFixtures(root);
 const view = suites.find(row => row.id === 'DK-23');
 assert.equal(view.files.length, 5);
 // Real Node harness executions do not stand in for application acceptance.
 const counts = [7, 5, 2, 7, 18];
 for (const [index, count] of counts.entries()) put(root, view.files[index], testSource(count));
 for (const [index, count] of counts.entries()) {
  put(root, view.files[index], testSource(count - 1));
  await assert.rejects(() => runSuites(root, spawnSync, () => {}), /incomplete test suite/, view.files[index]);
  put(root, view.files[index], testSource(count));
 }
 const accepted = (await runSuites(root, spawnSync, () => {})).find(row => row.id === 'DK-23');
 assert.deepEqual(accepted, {id: 'DK-23', tests: 39});
});

test('private prerequisite missing selected files reject before any suite is dispatched',async t=>{
 const root=temporary(t);testFixtures(root);
 for(const owner of additionalOwners)for(const file of owner.files)put(root,file,testSource(owner.minimum));
 for(const owner of additionalOwners)for(const file of owner.files){
  const bytes=readFileSync(join(root,file));rmSync(join(root,file));let calls=0;
  await assert.rejects(()=>runSuites(root,()=>{calls++;throw Error('must not dispatch');},()=>{}),/ENOENT/);
  assert.equal(calls,0,'preflight all private prerequisite files before execution');put(root,file,bytes);
 }
});

// Small real Node fixtures test the runner boundary, never the model kernels.
test('private prerequisite actual Node owners reject below-minimum and empty sibling completion',async t=>{
 const root=temporary(t);testFixtures(root);
 for(const owner of additionalOwners){
  assert.ok(suites.some(row=>row.id===owner.id),'mandatory installed '+owner.id);
  const restore=()=>owner.files.forEach((file,index)=>put(root,file,testSource(index?1:owner.minimum-owner.files.length+1)));
  restore();put(root,owner.files[0],testSource(owner.minimum-owner.files.length));
  await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/incomplete test suite/);restore();
  if(owner.files.length>1){
   put(root,owner.files[0],testSource(owner.minimum));put(root,owner.files[1],'');
   await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/nonempty registered tests|complete selected test file summaries|complete sequential outer test numbering/);restore();
  }
 }
 assert.equal((await runSuites(root,spawnSync,()=>{})).length,suites.length+fallbackSuites.length+standaloneSuites.length);
});

test('semantic suite refuses missing or empty siblings even with surplus real passing tests', async t => {
 const semantic = suites.find(row => row.id === 'DK-38');
 for (const file of semantic.files) {
  const root = temporary(t); testFixtures(root);
  rmSync(join(root, file));
  await assert.rejects(() => runSuites(root, spawnSync, () => {}), /ENOENT/);
  put(root, file, '');
  await assert.rejects(() => runSuites(root, spawnSync, () => {}), /complete selected test file summaries|complete sequential outer test numbering/);
  put(root, file, testSource(1));
  assert.equal((await runSuites(root, spawnSync, () => {})).find(row => row.id === 'DK-38').tests, semantic.minimum * 2 + 1);
 }
});

test('semantic registry regression kills owner, minimum and deadline substitutions', async t => {
 const original = sdkSource('scripts/browser-api-sdk-check.mjs');
 const row = original.split('\n').find(line => line.includes("{id:'DK-38'")); assert.ok(row);
 const changes = [row.replace('minimum:29', 'minimum:28'),
  row.replace('wire.test.mjs', 'substitute.test.mjs')];
 const mutated = changes.map(changed => original.replace(row, changed));
 const deadline = "deadline:row.id==='DK-35'?7200000:['DK-15','DK-16','DK-20','DK-23','DK-24','DK-25','DK-26','DK-27','DK-31','DK-32','DK-33','DK-34','DK-37','DK-38','ST-17','HO-15'].includes(row.id)?3600000:1500000";
 assert.equal(original.split(deadline).length, 2);
 mutated.push(original.replace(deadline, deadline.replace(", 'DK-38'", '').replace(",'DK-38'", '')));
 for (const source of mutated) {
  assert.notEqual(source, original);
  const root = temporary(t);
  const selected=stageGate(root,source);
  put(root, 'crates/conformance/src/cases/mod.rs', sdkSource('crates/conformance/src/cases/mod.rs'));
  const env = {...process.env}; delete env.NODE_TEST_CONTEXT;
  const result = spawnSync(process.execPath, ['--test', '--test-name-pattern=^semantic owner registration',
   selected], {encoding: 'utf8', env, timeout: 15000, maxBuffer: 1024 * 1024});
  assert.ifError(result.error); assert.equal(result.status, 1); assert.match(result.stdout, /ERR_ASSERTION/);
 }
});

test('installed semantic gate rejects omission of each false-evidence safeguard', async t => {
 const root = temporary(t); testFixtures(root);
 const semantic = suites.find(row => row.id === 'DK-38');
 const safeguards = ['missing oracle', 'launcher failure', 'closed page', 'no-op mutation', 'wrong semantic check'];
 const observations = ['immutable observations', 'exact journey and call inventory',
  'closed transcript engine', 'complete browser and mutant inventories'];
 const engineChecks = ['firefox generated journey', 'webkit generated journey',
  'firefox adapter mutants', 'webkit adapter mutants'];
 const fixtures = [
  [...observations, 'source closure', 'wire corpus', 'catalogue authority', 'immutable designs', 'aggregate bound'],
  ['positive journeys', 'eight actual mutants per engine', ...safeguards],
  ['owning parent', 'chromium generated journey', ...engineChecks.slice(0,2), '64 MiB maximum',
   'chromium adapter mutants', ...engineChecks.slice(2), 'purpose mutant', 'main mutant',
   'trailing mutant', 'design mutant', 'catalogue mutant'],
 ];
 const write = (index, omitted) => put(root, semantic.files[index], "import {test} from 'node:test';\n"
  + fixtures[index].filter(name => name !== omitted).map(name => `test(${JSON.stringify(name)},()=>{});\n`).join(''));
 fixtures.forEach((_, index) => write(index));
 assert.deepEqual((await runSuites(root, spawnSync, () => {})).find(row => row.id === 'DK-38'),
  {id: 'DK-38', tests: 29});
 for (const [index, required] of [[0, observations], [1, safeguards], [2, engineChecks]]) {
  for (const omitted of required) {
   write(index, omitted);
   await assert.rejects(() => runSuites(root, spawnSync, () => {}), /incomplete test suite/, omitted);
   write(index);
  }
 }
});

test('installed DK23 gate rejects omission of the ninth secret-input test', async t => {
 const root = temporary(t); testFixtures(root);
 const view = suites.find(row => row.id === 'DK-23');
 assert.equal(view.files.length, 5);
 // Real Node harness executions do not stand in for application acceptance.
 const counts = [7, 5, 2, 7, 18];
 for (const [index, count] of counts.entries()) put(root, view.files[index], testSource(count));
 for (const [index, count] of counts.entries()) {
  put(root, view.files[index], testSource(count - 1));
  await assert.rejects(() => runSuites(root, spawnSync, () => {}), /incomplete test suite/, view.files[index]);
  put(root, view.files[index], testSource(count));
 }
 const accepted = (await runSuites(root, spawnSync, () => {})).find(row => row.id === 'DK-23');
 assert.deepEqual(accepted, {id: 'DK-23', tests: 39});
});

test('installed DK26 gate requires every session kernel, canonical wire and custody check',async t=>{
 const root=temporary(t);testFixtures(root);
 const owner=suites.find(row=>row.id==='DK-26'),counts=[28,6,5];
 assert.equal(owner.files.length,counts.length);
 for(const [index,count] of counts.entries())put(root,owner.files[index],testSource(count));
 for(const [index,count] of counts.entries()){
  put(root,owner.files[index],testSource(count-1));
  await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/incomplete test suite/,owner.files[index]);
  put(root,owner.files[index],testSource(count));
 }
 assert.deepEqual((await runSuites(root,spawnSync,()=>{})).find(row=>row.id==='DK-26'),{id:'DK-26',tests:39});
});

test('installed DK33 gate requires original account corpus, every construction check and the full generated owner',async t=>{
 const root=temporary(t);testFixtures(root);
 const owner=suites.find(row=>row.id==='DK-33'),counts=[2,7,8];
 assert.deepEqual(owner.files,['tests/browser-account-genesis/corpus.test.mjs',
  'tests/browser-account-genesis/bridge.test.mjs','tests/browser-account-genesis/owner.test.mjs']);
 assert.equal(owner.minimum,17);assert.equal(owner.deadline,3600000);
 const sourceOwner=/"DK-33"\s*=>\s*verify_node_suite\(\s*root,\s*id,\s*&\[([^\]]+)\],\s*(\d+),\s*"([0-9]+)"/.exec(sdkSource('crates/conformance/src/cases/mod.rs'));
 assert.ok(sourceOwner);
 assert.deepEqual([...sourceOwner[1].matchAll(/"([^"]+)"/g)].map(row=>row[1]),owner.files);
 assert.equal(Number(sourceOwner[2]),owner.minimum);assert.equal(Number(sourceOwner[3]),owner.deadline);
 for(const [index,count] of counts.entries())put(root,owner.files[index],testSource(count));
 for(const [index,count] of counts.entries()) {
  put(root,owner.files[index],testSource(count-1));
  await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/incomplete test suite/,owner.files[index]);
  put(root,owner.files[index],testSource(count));
 }
 assert.deepEqual((await runSuites(root,spawnSync,()=>{})).find(row=>row.id==='DK-33'),{id:'DK-33',tests:17});
});

test('installed DK26 rejects an incomplete owner or empty wire file despite passing siblings', async t => {
 const root = temporary(t); testFixtures(root);
 const session = suites.find(row => row.id === 'DK-26');
 put(root, session.files[0], testSource(27));
 put(root, session.files[1], testSource(6));
 put(root, session.files[2], testSource(5));
 await assert.rejects(() => runSuites(root, spawnSync, () => {}), /incomplete test suite/);
 put(root, session.files[0], testSource(39));
 put(root, session.files[1], '');
 // Node's automatic empty-file wrapper may break the outer numbering before
 // the missing registered-file summary is checked. Neither is acceptance.
 await assert.rejects(() => runSuites(root, spawnSync, () => {}), /complete sequential outer test numbering|complete selected test file summaries/);
 put(root, session.files[0], testSource(28));
 put(root, session.files[1], testSource(6));
 assert.deepEqual((await runSuites(root, spawnSync, () => {})).find(row => row.id === 'DK-26'), {id: 'DK-26', tests: 39});
});

test('every selected file must exist even when its sibling supplies the total minimum',async t=>{
 const root=temporary(t);testFixtures(root);
 const path=join(root,'sdk/browser/identity.browser.test.mjs');
 rmSync(path);
 await assert.rejects(()=>runSuites(root,spawnSync,()=>{}));
 put(root,'sdk/browser/identity.browser.test.mjs',testSource(1));
 assert.equal((await runSuites(root,spawnSync,()=>{})).length,suites.length+fallbackSuites.length+standaloneSuites.length);
});

test('every selected file must register tests instead of borrowing its sibling counts',async t=>{
 const root=temporary(t);testFixtures(root);
 put(root,'sdk/browser/identity.browser.test.mjs','');
 await assert.rejects(()=>runSuites(root,spawnSync,()=>{}));
 put(root,'sdk/browser/identity.browser.test.mjs',testSource(1));
 assert.equal((await runSuites(root,spawnSync,()=>{})).length,suites.length+fallbackSuites.length+standaloneSuites.length);
});

test('preflight rejects missing late files, aliases and nonregular paths before any execution',async t=>{
 for(const kind of ['late missing','file alias','parent alias','root alias','directory','fifo']){
  const top=temporary(t),root=join(top,'root');testFixtures(root);let calls=0,selectedRoot=root;
  const path=join(root,'sdk/browser/identity.browser.test.mjs');
  if(kind==='late missing')rmSync(join(root,'sdk/browser/view-host-test.mjs'));
  if(kind==='file alias'){rmSync(path);symlinkSync('identity.test.mjs',path);}
  if(kind==='parent alias'){
   renameSync(join(root,'sdk/browser'),join(root,'sdk/browser-real'));
   symlinkSync('browser-real',join(root,'sdk/browser'));
  }
  if(kind==='root alias'){selectedRoot=join(top,'alias');symlinkSync('root',selectedRoot);}
  if(kind==='directory'){rmSync(path);mkdirSync(path);}
  if(kind==='fifo'){rmSync(path);execFileSync('mkfifo',[path]);}
  await assert.rejects(()=>runSuites(selectedRoot,()=>{calls++;throw Error('must not execute');},()=>{}),
   /ENOENT|selected test path alias|selected regular test file/);
  assert.equal(calls,0,kind);
 }
});

test('a module printing invented completion text does not count as registered tests',async t=>{
 const root=temporary(t);testFixtures(root);
 put(root,'sdk/browser/identity.browser.test.mjs',
  "console.log('# prismpm-owning-file '+JSON.stringify({file:import.meta.filename,tests:1,passed:1}));");
 await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/complete selected test file summaries/);
});

test('release acceptance actually invokes every closed owning suite and rejects omission or skip',async t=>{
 const root=temporary(t);testFixtures(root);const calls=[];
 const launch=(program,args,options)=>{calls.push(args);return spawnSync(program,args,options);};
 assert.deepEqual(suites.map(row=>row.id),['DK-07','DK-08','DK-09','DK-10','DK-11','DK-12','DK-13','DK-14','DK-15','DK-16','DK-19','DK-20','DK-23','DK-24','DK-25','DK-26','DK-27','DK-31','DK-32','DK-33','DK-34','DK-35','DK-37','DK-38','ST-17','HO-15']);
 const completed=(await runSuites(root,launch,()=>{})),executions=[...suites,...fallbackSuites,...standaloneSuites];
 assert.deepEqual(completed.slice(0,26).map(row=>row.id),suites.map(row=>row.id));
 assert.deepEqual(completed.slice(26,28),fallbackSuites.map(row=>({id:row.id,construction:'independent',tests:row.minimum})));
 assert.deepEqual(completed.slice(28),standaloneSuites.map(row=>({id:row.id,tests:row.minimum})));
 assert.equal(calls.length,34);
 assert.deepEqual(calls.map(args=>args.slice(4)),executions.map(row=>row.files));
 assert.deepEqual(calls.map(args=>args[3]),executions.map(row=>'--test-timeout='+row.deadline));
 const view=suites.find(row=>row.id==='DK-15');assert.equal(view.minimum,47);
 assert.equal(suites.find(row=>row.id==='DK-34').minimum,22);
 assert.equal(suites.find(row=>row.id==='DK-33').minimum,17);
 for(const first of [31,41]){
  for(const [index,path] of view.files.entries())put(root,path,testSource(index===0?first:1));
  await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/incomplete test suite/,'complete compiler custody owner refuses '+(first+5)+' tests');
 }
 testFixtures(root);
 const path='sdk/browser/identity.test.mjs',second='sdk/browser/identity.browser.test.mjs';
 put(root,path,testSource(1));put(root,second,testSource(1));await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/incomplete test suite/);
 put(root,path,testSource(10,true));put(root,second,testSource(10));await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/incomplete pass set|skipped/);
 put(root,path,testSource(10));
 const duplicate=(...args)=>{const result=spawnSync(...args);result.stdout+='# tests 20\n';return result;};
 await assert.rejects(()=>runSuites(root,duplicate,()=>{}),/duplicate tests/);
 testFixtures(root);rmSync(join(root,'sdk/browser/rs256.browser.test.mjs'));
 await assert.rejects(()=>runSuites(root,spawnSync,()=>{}),/ENOENT.*rs256\.browser\.test\.mjs/);
});

test('real TAP parsing rejects missing, duplicate, zero and unsuccessful summaries',async t=>{
 const root=temporary(t);put(root,'complete.mjs',testSource(3));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const output=spawnSync(process.execPath,['--test','--test-reporter=tap',join(root,'complete.mjs')],{encoding:'utf8',env});assert.equal(output.status,0);verifyTap(output.stdout,3);
 for(const changed of[
  output.stdout.replace(/^# tests .*\n/m,''),output.stdout+'# tests 3\n',
  output.stdout.replace('# tests 3','# tests 0'),output.stdout.replace('# fail 0','# fail 1'),
  output.stdout.replace('# cancelled 0','# cancelled 1'),output.stdout.replace('# skipped 0','# skipped 1'),
  output.stdout.replace('# todo 0','# todo 1'),output.stdout.replace('1..3','1..4'),
  output.stdout.replace('ok 2 -','ok 1 -'),output.stdout.replace('ok 1 -','not ok 1 -'),
  output.stdout.replace('# tests 3','# tests 03'),output.stdout.replace('TAP version 13',''),
 ])assert.throws(()=>verifyTap(changed,3));
});

test('per-file completion evidence is exact, closed, successful and reconciled to real TAP',async t=>{
 const root=temporary(t),file=join(root,'complete.mjs');put(root,'complete.mjs',testSource(3));
 const reporter='data:text/javascript;base64,'+readFileSync(new URL('./owning-node-reporter.mjs',import.meta.url)).toString('base64');
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const output=spawnSync(process.execPath,['--test','--test-reporter='+reporter,file],{encoding:'utf8',env});
 assert.equal(output.status,0);verifyTap(output.stdout,3);verifyFileCompletions(output.stdout,[file],3);
 const prefix='# prismpm-owning-file ',line=output.stdout.split('\n').find(line=>line.startsWith(prefix));
 const original=JSON.parse(line.slice(prefix.length));
 for(const mutate of[
  row=>row.file+='-wrong',row=>row.extra=true,row=>row.success=false,
  row=>row.tests=0,row=>row.tests='3',row=>row.tests=Number.MAX_SAFE_INTEGER+1,
  row=>row.passed--,row=>row.failed=1,row=>row.cancelled=1,row=>row.skipped=1,
  row=>row.todo=1,row=>row.topLevel=0,row=>row.topLevel=4,row=>delete row.suites,
 ]){
  const changed=structuredClone(original);mutate(changed);
  assert.throws(()=>verifyFileCompletions(output.stdout.replace(line,prefix+JSON.stringify(changed)),[file],3));
 }
 for(const changed of[output.stdout.replace(line,''),output.stdout+line+'\n',
  output.stdout.replace('"tests":3','"tests":3,"tests":3')])
  assert.throws(()=>verifyFileCompletions(changed,[file],3));
 assert.throws(()=>verifyFileCompletions(output.stdout,[file],4));
});

test('owning release test kills a removed complete-TAP acceptance guard',async t=>{
 const source=readFileSync(new URL('./browser-api-sdk-check.mjs',import.meta.url),'utf8');
 for(const [before,after,witness] of [
  ['const tests=verifyTap(output.stdout,execution.minimum)',"const tests=Number(/^# tests ([0-9]+)$/m.exec(output.stdout)[1])",/Missing expected rejection/],
  ["{id:'DK-15',minimum:47","{id:'DK-15',minimum:46",/46 !== 47/],
  ["{id:'DK-34',minimum:22","{id:'DK-34',minimum:21",/21 !== 22/],
  ["{id:'DK-33',minimum:17","{id:'DK-33',minimum:16",/16 !== 17/],
 ]){
 const root=temporary(t);
 assert.equal(source.split(before).length,2);const selected=stageGate(root,source.replace(before,after));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const result=spawnSync(process.execPath,['--test','--test-reporter=tap','--test-name-pattern=release acceptance actually',selected],{encoding:'utf8',env,timeout:30000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.status,1);assert.match(result.stdout,witness);
 }
});

test('owning omission regression kills removal of actual per-file completion checks',async t=>{
 const root=temporary(t),source=readFileSync(new URL('./browser-api-sdk-check.mjs',import.meta.url),'utf8');
 const before='verifyFileCompletions(output.stdout,selected.get(identity(execution)),tests,execution.exactTests);';
 assert.equal(source.split(before).length,2);const selected=stageGate(root,source.replace(before,''));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const result=spawnSync(process.execPath,['--test','--test-reporter=tap',
  '--test-name-pattern=every selected file must register',selected],
  {encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});
 assert.equal(result.error,undefined);assert.equal(result.status,1);assert.match(result.stdout,/Missing expected rejection/);
});

test('owning installed-module tests kill removed exact-tree and byte-equality guards', async t => {
 const original = sdkSource('scripts/browser-api-sdk-check.mjs');
 for (const omitted of [
  "assert.deepEqual(readdirSync(installed).sort(),names.slice().sort(),'exact installed host module closure');",
  "assert.deepEqual(boundedBytes(actual[index],lstatSync(actual[index])),\n   boundedBytes(sources[index],lstatSync(sources[index])),'installed host module bytes: '+names[index]);",
 ]) {
  assert.equal(original.split(omitted).length, 2);
  const root = temporary(t);
  const selected=stageGate(root,original.replace(omitted,''));
  const env = {...process.env}; delete env.NODE_TEST_CONTEXT;
  const result = spawnSync(process.execPath, ['--test', '--test-name-pattern=actual installed module trees',
   selected], {env, encoding: 'utf8', timeout: 15000, maxBuffer: 1024*1024});
  assert.equal(result.error, undefined); assert.equal(result.status, 1);
  assert.match(result.stdout, /Missing expected exception/);
 }
});


test('installed suite diagnostics finish the real pipe before an actual failed child is reported', {timeout: 20000}, async t => {
 const root=temporary(t);testFixtures(root);
 stageGate(root,sdkSource('scripts/browser-api-sdk-check.mjs'));
 const length=4*1024*1024,tail='\nFAILED-CHILD-STDOUT-END\n',err='FAILED-CHILD-STDERR-END\n';
 const payload='x'.repeat(length)+tail;
 // The first subprocess genuinely exits1 after flushing both original streams.
 // Only the gate's forwarding into the second, deliberately backpressured pipe
 // is under test. No compiler, browser or installed-SDK owner is substituted.
 const failureSource=[
  "const write=(stream,text)=>new Promise((resolve,reject)=>stream.write(text,error=>error?reject(error):resolve()));",
  'await write(process.stdout,'+JSON.stringify('x')+'.repeat('+length+')+'+JSON.stringify(tail)+');',
  'await write(process.stderr,'+JSON.stringify(err)+');',
  'process.exitCode=1;',
 ].join('\n');
 const runner=join(root,'failed-suite-pipe.mjs');
 put(root,'failed-suite-pipe.mjs',[
  "import {spawnSync} from 'node:child_process';",
  "import {runStandaloneSuites} from './scripts/browser-api-sdk-check.mjs';",
  'const failureSource='+JSON.stringify(failureSource)+';',
  'await runStandaloneSuites('+JSON.stringify(root)+',(_program,_args,options)=>spawnSync(process.execPath,',
  " ['--input-type=module','-e',failureSource],{encoding:'utf8',env:options.env,timeout:5000,maxBuffer:8*1024*1024}));",
 ].join('\n'));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const child=spawn(process.execPath,[runner],{cwd:root,env,stdio:['ignore','pipe','pipe']});
 const stdout=[],stderr=[];let outputBytes=0,errorBytes=0,launchError,overflow=false;
 const stop=()=>{if(child.exitCode===null&&child.signalCode===null)child.kill('SIGTERM');};
 const watchdog=setTimeout(stop,10000);watchdog.unref();
 const closed=new Promise(resolve=>{
  child.once('error',error=>{launchError=error;});
  child.once('close',(status,signal)=>resolve({status,signal}));
 });
 child.stdout.on('data',bytes=>{outputBytes+=bytes.length;if(outputBytes>8*1024*1024){overflow=true;stop();}else stdout.push(bytes);});
 child.stderr.on('data',bytes=>{errorBytes+=bytes.length;if(errorBytes>1024*1024){overflow=true;stop();}else stderr.push(bytes);});
 child.stdout.pause();
 const resume=setTimeout(()=>child.stdout.resume(),100);resume.unref();
 t.after(()=>{clearTimeout(resume);clearTimeout(watchdog);stop();});
 const result=await closed;clearTimeout(resume);clearTimeout(watchdog);
 assert.ifError(launchError);assert.equal(overflow,false);assert.equal(result.signal,null);assert.equal(result.status,1);
 assert.match(Buffer.concat(stderr).toString(),/complete owning Session Projection/);
 const actual=Buffer.concat(stdout),expected=Buffer.from('SDK browser suite Session Projection\n'+payload+err);
 assert.equal(actual.length,expected.length,'complete failed-child output must precede nonzero gate exit');
 assert.equal(createHash('sha256').update(actual).digest('hex'),createHash('sha256').update(expected).digest('hex'));
});

test('installed suite async forwarding preserves launch and status failures when output also fails', async t => {
 for(const mode of ['launch','status'])await t.test(mode,async t=>{
  const root=temporary(t);testFixtures(root);
  const original=Object.assign(new Error('original launch failure'),{result:{stdout:Buffer.from('ORIGINAL-OUT\n'),stderr:Buffer.from('ORIGINAL-ERR\n')}});
  const launch=()=>{if(mode==='launch')throw original;return {error:undefined,status:1,signal:null,stdout:'ORIGINAL-OUT\n',stderr:'ORIGINAL-ERR\n'};};
  let delivered=false,actual;
  await assert.rejects(()=>runStandaloneSuites(root,launch,async text=>{
   await new Promise(resolve=>setImmediate(resolve));delivered=true;actual=text;
  }),error=>{
   assert.equal(delivered,true,'async emission must finish before reporting the primary failure');
   assert.equal(actual,'SDK browser suite Session Projection\nORIGINAL-OUT\nORIGINAL-ERR\n');
   if(mode==='launch')assert.equal(error,original);else assert.match(error.message,/complete owning Session Projection/);
   return true;
  });
  const forwarding=new Error('actual diagnostic forwarding failure');
  await assert.rejects(()=>runStandaloneSuites(root,launch,()=>Promise.reject(forwarding)),error=>{
   assert(error instanceof AggregateError,'neither primary failure nor forwarding failure may be hidden');
   assert.equal(error.errors.length,2);assert.equal(error.errors[1],forwarding);
   if(mode==='launch')assert.equal(error.errors[0],original);else assert.match(error.errors[0].message,/complete owning Session Projection/);
   assert.equal(error.cause,error.errors[0]);return true;
  });
 });
});


test('installed suite default stdout preserves original failure on a real broken pipe', {timeout: 20000}, async t => {
 const root=temporary(t);testFixtures(root);
 stageGate(root,sdkSource('scripts/browser-api-sdk-check.mjs'));
 const failureSource=[
  "const write=(stream,text)=>new Promise((resolve,reject)=>stream.write(text,error=>error?reject(error):resolve()));",
  'await write(process.stdout,'+JSON.stringify('BROKEN-ORIGINAL-OUT\n')+');',
  'await write(process.stderr,'+JSON.stringify('BROKEN-ORIGINAL-ERR\n')+');',
  'process.exitCode=1;',
 ].join('\n');
 const runner=join(root,'broken-suite-pipe.mjs');
 put(root,'broken-suite-pipe.mjs',[
  "import {spawnSync} from 'node:child_process';",
  "import {runStandaloneSuites} from './scripts/browser-api-sdk-check.mjs';",
  "const write=(stream,text)=>new Promise((resolve,reject)=>stream.write(text,error=>error?reject(error):resolve()));",
  'const failureSource='+JSON.stringify(failureSource)+';let original;',
  'await write(process.stderr,'+JSON.stringify('BROKEN-SINK-READY\n')+');',
  "await new Promise(resolve=>process.stdin.once('data',resolve));",
  'try{await runStandaloneSuites('+JSON.stringify(root)+',(_program,_args,options)=>{',
  " original=spawnSync(process.execPath,['--input-type=module','-e',failureSource],{encoding:'utf8',env:options.env,timeout:5000,maxBuffer:1024*1024});return original;});}",
  'catch(error){const row={original_status:original?.status,aggregate:error instanceof AggregateError,',
  ' primary_message:error.errors?.[0]?.message??error.message,forwarding_code:error.errors?.[1]?.code??null,',
  ' cause_is_primary:error.cause===error.errors?.[0]};',
  ' await write(process.stderr,'+JSON.stringify('BROKEN-SINK-EVIDENCE ')+'+JSON.stringify(row)+'+JSON.stringify('\n')+');process.exitCode=1;}',
 ].join('\n'));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const child=spawn(process.execPath,[runner],{cwd:root,env,stdio:['pipe','pipe','pipe']});
 const stderr=[];let size=0,launchError,inputError,overflow=false,closedReader=false;
 const stop=()=>{if(child.exitCode===null&&child.signalCode===null)child.kill('SIGTERM');};
 const watchdog=setTimeout(stop,10000);watchdog.unref();
 const done=new Promise(resolve=>{
  child.once('error',error=>{launchError=error;});
  child.once('close',(status,signal)=>resolve({status,signal}));
 });
 child.stdin.on('error',error=>{inputError=error;});
 child.stderr.on('data',bytes=>{
  size+=bytes.length;if(size>1024*1024){overflow=true;stop();return;}stderr.push(bytes);
  if(!closedReader&&Buffer.concat(stderr).toString().includes('BROKEN-SINK-READY\n')){
   closedReader=true;
   child.stdout.once('close',()=>child.stdin.end('go\n'));
   child.stdout.destroy();
  }
 });
 t.after(()=>{clearTimeout(watchdog);stop();});
 const result=await done;clearTimeout(watchdog);
 assert.ifError(launchError);assert.ifError(inputError);assert.equal(overflow,false);assert.equal(closedReader,true);
 assert.equal(result.signal,null);assert.equal(result.status,1);
 const text=Buffer.concat(stderr).toString(),prefix='BROKEN-SINK-EVIDENCE ';
 const rows=text.split('\n').filter(line=>line.startsWith(prefix));assert.equal(rows.length,1,'actual default-emitter failure record');
 const actual=JSON.parse(rows[0].slice(prefix.length));
 assert.equal(actual.original_status,1);assert.equal(actual.aggregate,true);
 assert.match(actual.primary_message,/complete owning Session Projection/);
 assert.equal(actual.forwarding_code,'EPIPE');assert.equal(actual.cause_is_primary,true);
 assert(!text.includes("Unhandled 'error' event"),'owned stdout error must not escape aggregation');
});


// Private-scratch lifecycle fixtures exercise orchestration, never installed SDK acceptance.
test('all original owning executions retire successful private scratch before the next launch',async t=>{
 const root=temporary(t);testFixtures(root);const seen=[],forwarded=[];
 const expectedEnv={...process.env};delete expectedEnv.NODE_TEST_CONTEXT;
 const launch=(program,args,options)=>{
  if(seen.length)assert.equal(existsSync(seen.at(-1)),false,'successful scratch survived into the next owner');
  const scratch=options.env.TMPDIR;assert.equal(typeof scratch,'string','private suite TMPDIR required');
  assert.match(scratch.slice(scratch.lastIndexOf('/')+1),/^prismpm-sdk-suite-[A-Za-z0-9]+$/);
  assert(!seen.includes(scratch),'each original owner needs a fresh scratch root');
  assert.equal(lstatSync(scratch).mode&0o077,0);assert.equal(lstatSync(scratch).uid,process.getuid());
  assert.deepEqual(options.env,{...expectedEnv,TMPDIR:scratch});seen.push(scratch);
  t.after(()=>rmSync(scratch,{recursive:true,force:true}));
  put(scratch,'original-owner-marker','retained until complete output and receipts\n');
  return spawnSync(program,args,options);
 };
 const emit=text=>{assert(existsSync(seen.at(-1)),'scratch retired before original diagnostic forwarding');forwarded.push(text);};
 const completed=await runSuites(root,launch,emit);
 const all=[...suites,...fallbackSuites,...standaloneSuites];
 assert.deepEqual(completed.map(row=>row.id),all.map(row=>row.id));
 assert.equal(seen.length,all.length);assert.equal(new Set(seen).size,all.length);assert.equal(forwarded.length,all.length);
 for(const path of seen)assert.equal(existsSync(path),false,'a successful owner scratch remains');
});

test('failed launch status forwarding or incomplete owning receipts preserve private scratch',async t=>{
 for(const defect of ['launch','status','forwarding','TAP','file-receipt']){
  const root=temporary(t);testFixtures(root);let scratch;
  const launch=(program,args,options)=>{
   scratch=options.env.TMPDIR;assert.equal(typeof scratch,'string','private suite TMPDIR required');
   t.after(()=>rmSync(scratch,{recursive:true,force:true}));put(scratch,'original-owner-marker',defect);
   if(defect==='launch')throw Error('planted launch failure');
   const result=spawnSync(program,args,options);assert.equal(result.status,0);
   if(defect==='status')return {...result,status:1};
   if(defect==='TAP')result.stdout=result.stdout.replace(/^# pass \d+$/m,'# pass 0');
   if(defect==='file-receipt')result.stdout=result.stdout.replace(/^# prismpm-owning-file [^\n]*\n/m,'');
   return result;
  };
  const emit=()=>{if(defect==='forwarding')throw Error('planted forwarding failure');};
  const witness={launch:/planted launch failure/,status:/complete owning DK-07/,forwarding:/planted forwarding failure/,
   TAP:/incomplete pass set/,'file-receipt':/complete selected test file summaries/}[defect];
  await assert.rejects(()=>runSuites(root,launch,emit),witness,defect);
  assert.equal(readFileSync(join(scratch,'original-owner-marker'),'utf8'),defect,'failed scratch was destroyed');
 }
});

test('successful scratch retirement refuses a replaced aliased or less-private root',async t=>{
 for(const defect of ['replaced','alias','mode']){
  const root=temporary(t);testFixtures(root);let scratch,saved;
  const launch=(program,args,options)=>{
   scratch=options.env.TMPDIR;assert.equal(typeof scratch,'string','private suite TMPDIR required');saved=scratch+'.saved';
   t.after(()=>{rmSync(scratch,{recursive:true,force:true});rmSync(saved,{recursive:true,force:true});});
   put(scratch,'original-owner-marker',defect);const result=spawnSync(program,args,options);assert.equal(result.status,0);
   if(defect==='mode')chmodSync(scratch,0o755);
   else{renameSync(scratch,saved);if(defect==='alias')symlinkSync(saved,scratch);else mkdirSync(scratch,{mode:0o700});}
   return result;
  };
  await assert.rejects(()=>runSuites(root,launch,()=>{}),/same completed owning suite scratch|unaliased completed owning suite scratch|owned private completed owning suite scratch/,defect);
  assert.equal(readFileSync(join(defect==='mode'?scratch:saved,'original-owner-marker'),'utf8'),defect,'refused scratch was destroyed');
  assert(lstatSync(scratch),'refused replacement/alias must remain observable');
 }
});

test('the owning lifecycle control detects an omitted successful-scratch retirement',async t=>{
 const root=temporary(t);testFixtures(root);const original=sdkSource('scripts/browser-api-sdk-check.mjs');
 const marker='  retireSuccessfulSuiteScratch(scratch);';assert.equal(original.split(marker).length,2,'one actual retirement boundary');
 stageGate(root,original.replace(marker,'  // planted missing successful scratch retirement'));
 const run=(await import(pathToFileURL(join(root,'scripts/browser-api-sdk-check.mjs')).href)).runSuites;
 const seen=[];let calls=0;
 const launch=(program,args,options)=>{
  const scratch=options.env.TMPDIR;assert.equal(typeof scratch,'string');
  t.after(()=>rmSync(scratch,{recursive:true,force:true}));
  calls++;if(seen.length)assert.equal(existsSync(seen.at(-1)),false,'successful scratch survived into the next owner');
  seen.push(scratch);return spawnSync(program,args,options);
 };
 await assert.rejects(()=>run(root,launch,()=>{}),/successful scratch survived into the next owner/);
 assert.equal(calls,2,'actual first owner passes before the omission is observed at the next launch');
 assert.equal(seen.length,1);assert(existsSync(seen[0]),'the planted omission must leave actual private scratch');
});


// Whole-wrapper command recordings qualify only lifecycle controls, never Docker/SDK behavior.
function recordBrowserWrapper(t,script,defect='') {
 const root=temporary(t),bin=join(root,'fixture-bin'),state=join(root,'fixture-state.json');
 put(root,'scripts/browser-api-sdk-check.sh',script);put(root,'sdk/devcontainer-init.sh','fixture entrypoint\n');
 put(root,'scripts/browser-api-sdk-check.mjs','fixture helper\n');
 writeFileSync(state,JSON.stringify({defect,next:1,volume:null,volume_created:false,containers:{},events:[]}));
 put(bin,'docker','#!/usr/bin/python3\n'+VOLUME_DOCKER_FIXTURE);
 put(bin,'git',"#!/usr/bin/python3\nimport sys\nif sys.argv[-2:]==['rev-parse','HEAD']:print('a'*40)\nelif sys.argv[-2:]==['status','--porcelain']:pass\nelse:sys.exit(99)\n");
 put(bin,'node',"#!/usr/bin/python3\nimport json,sys\nm=sys.argv[2]\nif m=='capture':print('{}')\nelif m=='roots':print('fixture-root')\nelif m=='image':json.load(sys.stdin)\n");
 for(const file of ['docker','git','node'])chmodSync(join(bin,file),0o755);
 const result=spawnSync('/bin/bash',[join(root,'scripts/browser-api-sdk-check.sh'),image,revision],
  {encoding:'utf8',timeout:20000,maxBuffer:1048576,env:{...process.env,PATH:bin+':'+process.env.PATH,
   PRISM_VOLUME_FIXTURE_STATE:state,PRISM_VOLUME_FIXTURE_IMAGE:image}});
 assert.equal(result.error,undefined,'bounded actual wrapper fixture');assert.equal(result.signal,null);
 return {result,state:JSON.parse(readFileSync(state,'utf8'))};
}
function requireBrowserWrapperLifecycle(observation) {
 const {result,state}=observation,events=state.events,creates=events.filter(row=>row.argv[0]==='container'&&row.argv[1]==='create');
 assert.equal(result.status,0,result.stderr);assert(result.stdout.includes('browser SDK closure passed:'));
 assert.equal(events.filter(row=>row.argv[0]==='volume'&&row.argv[1]==='create').length,1,'missing private same-daemon volume');
 assert.deepEqual(creates.map(row=>row.role),['copy','init','owner']);
 const init=creates.find(row=>row.role==='init'),owner=creates.find(row=>row.role==='owner');
 for(const row of [init,owner])assert(row.argv.some(arg=>arg.includes('target=/tmp,volume-nocopy')),'private volume-nocopy required');
 assert(init.argv.includes('0:0')&&owner.argv.includes('1000:1000'));
 const started=events.filter(row=>row.argv[0]==='container'&&row.argv[1]==='start');
 assert.equal(started.find(row=>row.role==='init').init_contract,true,'exact fresh-root sole-chmod initializer');
 assert.equal(started.find(row=>row.role==='owner').admission,true,'missing actual filesystem 12 GiB admission');
 assert.equal(started.find(row=>row.role==='owner').exec_original,true,'original owning Node must replace preflight shell');
 assert.equal(events.filter(row=>row.argv[0]==='container'&&row.argv[1]==='rm').some(row=>row.forced),false);
 assert.equal(Object.keys(state.containers).length,0);assert.equal(state.volume,null,'actual same-name volume absence required');
 const removed=events.findIndex(row=>row.argv[0]==='volume'&&row.argv[1]==='rm');
 assert(removed>=0&&events.slice(removed+1).some(row=>row.argv[0]==='volume'&&row.argv[1]==='ls'),'checked volume absence after normal removal');
}
const VOLUME_DOCKER_FIXTURE=String.raw`import io,json,os,sys,tarfile
from pathlib import Path
p=Path(os.environ['PRISM_VOLUME_FIXTURE_STATE']);a=sys.argv[1:];v=json.loads(p.read_text());d=v['defect'];event={'argv':a};v['events'].append(event)
def done(code=0,out=''):
 p.write_text(json.dumps(v));sys.stdout.write(out);sys.exit(code)
def opt(flag,default=None):
 return a[a.index(flag)+1] if flag in a else default
if a[0]=='pull':done(0,'fixture pull\n')
if a[:2]==['image','inspect']:done(0,'sha256:'+'f'*64+'\n' if '--format'in a else '[{}]\n')
if a[0]=='volume':
 action=a[1];name=a[-1]
 if action=='ls':
  if d=='ls-failure'and not v['volume_created']:done(37)
  done(0,(v['volume']['name']+'\n')if v['volume'] else '')
 if action=='create':
  nonce=opt('--label').split('=',1)[1];v['volume']={'name':name,'nonce':nonce,'mode':None};v['volume_created']=True;done(0,name+'\n')
 if action=='inspect':
  if not v['volume']:done(1)
  r=v['volume'];nonce='wrong-nonce'if d=='wrong-volume-nonce'else r['nonce'];done(0,f"{r['name']} local local 0 1 {nonce}\n")
 if action=='rm':
  if d=='owner-and-cleanup-failure':done(45)
  if d!='volume-removal-lied':v['volume']=None
  done(0,name+'\n')
if a[0]=='container':
 action=a[1]
 if action=='ls':
  pattern=opt('--filter','');rows=list(v['containers'].values())
  if pattern.startswith('id='):rows=[r for r in rows if r['id']==pattern[3:]]
  elif pattern.startswith('name=^/'):rows=[r for r in rows if r['name']==pattern[7:-1]]
  done(0,''.join((r['name']if opt('--format')=='{{.Names}}'else r['id'][:12])+'\n'for r in rows))
 if action=='create':
  labels=[a[i+1]for i,x in enumerate(a)if x=='--label'];labels=dict(x.split('=',1)for x in labels)
  name=opt('--name','baseline-'+str(v['next']));role=labels.get('org.uor.prismpm.browser-sdk-role','copy'if opt('--entrypoint')=='/usr/bin/true'else'owner')
  cid=f"{v['next']:064x}";v['next']+=1;cmd=a[a.index(os.environ['PRISM_VOLUME_FIXTURE_IMAGE'])+1:]
  r={'id':cid,'name':name,'role':role,'nonce':labels.get('org.uor.prismpm.browser-sdk-private',''),'running':False,'pid':0,'exit':0,'create':a,'cmd':cmd}
  v['containers'][cid]=r;event['role']=role;event['id']=cid
  if d=='lost-init-create'and role=='init':done(43)
  done(0,cid+'\n')
 if action=='cp':
  if a[-1]=='-':
   mem=io.BytesIO()
   with tarfile.open(fileobj=mem,mode='w')as t:
    n='fixture-root/file'if'/conformance-root/'in a[2]else'fixture-file';r=tarfile.TarInfo(n);body=b'fixture\n';r.size=len(body);r.mode=0o644;t.addfile(r,io.BytesIO(body))
   p.write_text(json.dumps(v));sys.stdout.buffer.write(mem.getvalue());sys.exit(0)
  Path(a[-1]).write_text('fixture entrypoint\n');done()
 target=a[-1];r=next((r for r in v['containers'].values()if target in[r['id'],r['name']]),None)
 if r is None:done(1)
 event['role']=r['role'];event['id']=r['id']
 if action=='inspect':
  if opt('--format')=='{{.State.ExitCode}}':done(0,str(r['exit'])+'\n')
  iid='e'*64 if d=='wrong-owner-image'and r['role']=='owner'else'f'*64
  done(0,f"{r['id']} /{r['name']} sha256:{iid} {r['nonce']} {r['role']} {str(r['running']).lower()} {r['pid']}\n")
 if action=='start':
  if r['role']=='init':
   event['init_contract']=all(x in' '.join(r['cmd'])for x in['lstatSync','readdirSync','s.uid===0','chmodSync("/tmp",0o1777)']);v['volume']['mode']=0o1777
  if r['role']=='owner':
   joined=' '.join(r['cmd']);event['admission']='statfsSync("/tmp"'in joined and'assert(available>=12n*1024n**3n'in joined;event['exec_original']='exec node scripts/browser-api-sdk-check.mjs run'in joined
   event['mounted_volume']=opt('--unused')
   if d in['owner-and-cleanup-failure','live-owner-failure']:
    r['exit']=42;r['running']=d=='live-owner-failure';r['pid']=12345 if r['running'] else 0;done(42,'fixture original owner failure\n')
  done()
 if action=='stop':r['running']=False;r['pid']=0;done(0,r['id']+'\n')
 if action=='rm':
  event['forced']='--force'in a
  if r['running']and'--force'not in a:done(1)
  del v['containers'][r['id']];done(0,r['id']+'\n')
raise RuntimeError('unexpected fake Docker argv '+repr(a))
`;

test('actual browser wrapper private-volume lifecycle rejects original tmpfs and removed admission or no-copy guards',t=>{
 const candidate=sdkSource('scripts/browser-api-sdk-check.sh');requireBrowserWrapperLifecycle(recordBrowserWrapper(t,candidate));
 const original="#!/usr/bin/env bash\n# Independent post-build closure test; never consumes the external-oracle image override.\nset -euo pipefail\ntest \"$#\" -eq 2 -o \"$#\" -eq 3 || { echo 'usage: browser-api-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT [EXPLICIT_REGISTRY_CA]' >&2; exit 64; }\nimage=$1\nrevision=$2\nroot=$(cd \"$(dirname \"$0\")/..\" && pwd -P)\nhelper=\"$root/scripts/browser-api-sdk-check.mjs\"\nif test \"$#\" -eq 3; then\n  node \"$root/scripts/sdk-qualification-trust.mjs\" \"$3\"\n  export NODE_EXTRA_CA_CERTS=$3\nfi\n[[ $image =~ ^[a-z0-9][a-z0-9./:_-]*@sha256:[0-9a-f]{64}$ ]] || exit 64\n[[ $revision =~ ^[0-9a-f]{40}$ ]] || exit 64\ntest \"$(git -C \"$root\" rev-parse HEAD)\" = \"$revision\"\ntest -z \"$(git -C \"$root\" status --porcelain)\"\ncase \"$(uname -m)\" in x86_64) architecture=amd64 ;; aarch64) architecture=arm64 ;; *) exit 64 ;; esac\nsdk_work=$(mktemp -d)\ncontainer=''\ncleanup() {\n  if test -n \"$container\"; then docker container rm --force \"$container\" >/dev/null 2>&1 || true; fi\n  # Docker preserves the image's read-only directory modes in this owned copy.\n  chmod -R u+rwX -- \"$sdk_work\"\n  rm -r -- \"$sdk_work\"\n}\ntrap cleanup EXIT\ntrap 'exit 130' INT\ntrap 'exit 143' TERM\nnode \"$helper\" capture \"$root\" \"$revision\" > \"$sdk_work/source.json\"\ndocker pull --platform \"linux/$architecture\" \"$image\"\ndocker image inspect \"$image\" | node \"$helper\" image \"$image\" \"$architecture\" \"$revision\"\ncontainer=$(docker container create --network none --read-only --entrypoint /usr/bin/true \"$image\")\n[[ $container =~ ^[0-9a-f]{64}$ ]] || exit 1\nmkdir \"$sdk_work/source\" \"$sdk_work/browser\"\nwhile IFS= read -r path; do\n  mkdir -p \"$sdk_work/source/$(dirname \"$path\")\"\n  # Defer read-only directory modes until descendants are written.\n  docker container cp \"$container:/opt/prismpm/share/conformance-root/$path\" - | tar --extract --no-same-owner --no-same-permissions --delay-directory-restore --file - --directory \"$sdk_work/source/$(dirname \"$path\")\"\ndone < <(node \"$helper\" roots)\nnode \"$helper\" verify \"$sdk_work/source\" \"$sdk_work/source.json\"\n# Defer read-only directory modes until descendants are written.\ndocker container cp \"$container:/opt/prismpm/browser/.\" - | tar --extract --no-same-owner --no-same-permissions --delay-directory-restore --file - --directory \"$sdk_work/browser\"\nnode \"$helper\" modules \"$root\" \"$sdk_work/browser\"\nmkdir -p \"$sdk_work/share/account-genesis\"\n# Defer read-only directory modes until descendants are written.\ndocker container cp \"$container:/opt/prismpm/share/account-genesis/.\" - | tar --extract --no-same-owner --no-same-permissions --delay-directory-restore --file - --directory \"$sdk_work/share/account-genesis\"\nnode \"$helper\" account-construction \"$sdk_work\" \"$sdk_work/source.json\"\ndocker container cp \"$container:/usr/local/bin/prismpm-devcontainer-init\" \"$sdk_work/entrypoint.sh\"\ncmp \"$root/sdk/devcontainer-init.sh\" \"$sdk_work/entrypoint.sh\"\ndocker container rm \"$container\" >/dev/null\ncontainer=''\n# Only private temporary caches/builds are writable. The source, tools and browser\n# modules come from the inspected image; no host source or dependency mount exists.\ncontainer=$(docker container create --user 1000:1000 --read-only --network none --cap-drop ALL \\\n  --security-opt no-new-privileges --tmpfs /tmp:rw,exec,nosuid,nodev,size=8g \\\n  --env PRISMPM_EPHEMERAL_HOME=1 --env CARGO_NET_OFFLINE=true \\\n  --workdir /opt/prismpm/share/conformance-root \"$image\" \\\n  node scripts/browser-api-sdk-check.mjs run)\n[[ $container =~ ^[0-9a-f]{64}$ ]] || exit 1\ndocker container start --attach \"$container\"\ntest \"$(docker container inspect --format '{{.State.ExitCode}}' \"$container\")\" = 0\ndocker container rm \"$container\" >/dev/null\ncontainer=''\nnode \"$helper\" verify \"$root\" \"$sdk_work/source.json\"\nprintf 'browser SDK closure passed: source=%s image=%s platform=linux/%s\\n' \"$revision\" \"$image\" \"$architecture\"\n";
 const prior=recordBrowserWrapper(t,original);assert.equal(prior.result.status,0,prior.result.stderr);
 assert.throws(()=>requireBrowserWrapperLifecycle(prior),/missing private same-daemon volume/);
 const guard='assert(available>=12n*1024n**3n,"12 GiB private compiler reserve before original installed owners");';
 assert.equal(candidate.split(guard).length,2,'one actual mounted-filesystem admission');
 assert.throws(()=>requireBrowserWrapperLifecycle(recordBrowserWrapper(t,candidate.replace(guard,'void 0;'))),/missing actual filesystem 12 GiB admission/);
 assert.throws(()=>requireBrowserWrapperLifecycle(recordBrowserWrapper(t,candidate.replaceAll(',volume-nocopy',''))),/private volume-nocopy required/);
 const lied=recordBrowserWrapper(t,candidate,'volume-removal-lied');assert.notEqual(lied.result.status,0,'successful rm without actual absence must reject');
 assert(!lied.result.stdout.includes('browser SDK closure passed:'));assert(lied.state.volume);
});

test('actual browser wrapper preserves original failures and refuses unknown-volume or container cleanup',t=>{
 const candidate=sdkSource('scripts/browser-api-sdk-check.sh');
 for(const [defect,status]of [['ls-failure',37],['wrong-volume-nonce',1],['wrong-owner-image',1],['owner-and-cleanup-failure',42]]){
  const row=recordBrowserWrapper(t,candidate,defect);assert.equal(row.result.status,status,row.result.stderr);
  assert(!row.result.stdout.includes('browser SDK closure passed:'));
  if(defect==='ls-failure')assert.equal(row.state.events.some(e=>e.argv[0]==='volume'&&e.argv[1]==='create'),false);
  if(defect==='wrong-volume-nonce'||defect==='wrong-owner-image'){
   assert(row.state.volume);assert.equal(row.state.events.some(e=>e.argv[0]==='volume'&&e.argv[1]==='rm'),false,'unknown scope must not be removed');
  }
  if(defect==='owner-and-cleanup-failure')assert(row.result.stderr.includes('browser SDK owned cleanup failed'));
 }
});

test('actual browser wrapper recovers lost create output and normally retires a live failed owner before its volume',t=>{
 const candidate=sdkSource('scripts/browser-api-sdk-check.sh');
 const lost=recordBrowserWrapper(t,candidate,'lost-init-create');assert.equal(lost.result.status,43,lost.result.stderr);
 assert.equal(Object.keys(lost.state.containers).length,0);assert.equal(lost.state.volume,null);
 const live=recordBrowserWrapper(t,candidate,'live-owner-failure');assert.equal(live.result.status,42,live.result.stderr);
 const events=live.state.events,stop=events.findIndex(e=>e.argv[0]==='container'&&e.argv[1]==='stop');
 assert(stop>=0);assert.deepEqual(events[stop].argv.slice(2,4),['--timeout','5']);
 const removed=events.findIndex((e,i)=>i>stop&&e.argv[0]==='container'&&e.argv[1]==='rm');
 const volume=events.findIndex(e=>e.argv[0]==='volume'&&e.argv[1]==='rm');assert(removed>stop&&volume>removed);
 assert.equal(Object.keys(live.state.containers).length,0);assert.equal(live.state.volume,null);
 assert.equal(events.some(e=>e.argv.includes('--force')),false);
});
