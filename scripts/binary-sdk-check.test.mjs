// Synthetic parser/boundary tests are not installed CLI acceptance evidence.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,readdirSync,rmSync,renameSync,symlinkSync,linkSync,cpSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {test} from 'node:test';
import {capture,validateGenerationWorkflow,verifySource,sourceRoots,sourceAliases,cli,checkAccepted as checkWithAuthority,mutateModule,tree,verifyImage,verifyResult,testOutput,hash,canonical,fixtureProgram,modes,processTools,binaryPaths,completedChecks,unclaimed,inventoryEvidence,regularBytes,rejectChangedEvidence as rejectWithAuthority} from './binary-sdk-check.mjs';
import {inventory,parserFixture,sourceAuthorityFixture} from './binary-sdk-check-fixtures.mjs';
const checkAccepted=(root,receipt)=>checkWithAuthority(root,receipt,sourceAuthorityFixture);
const rejectChangedEvidence=(root,receipt,path)=>rejectWithAuthority(root,receipt,path,sourceAuthorityFixture);
const revision='a'.repeat(40),image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
const temporary=t=>{const root=mkdtempSync(join(tmpdir(),'prismpm-binary-gate-test-'));t.after(()=>rmSync(root,{recursive:true,force:true}));return root;};
const put=(root,path,bytes)=>{mkdirSync(dirname(join(root,path)),{recursive:true});writeFileSync(join(root,path),bytes);};
function source(root){
 for(const path of sourceRoots)if(!Object.hasOwn(sourceAliases,path))put(root,path+'/input','source');
 for(const [path,target] of Object.entries(sourceAliases)){
  const full=join(root,path);mkdirSync(dirname(full),{recursive:true});
  if(path.includes('/embedded/')){const actual=resolve(dirname(full),target);mkdirSync(dirname(actual),{recursive:true});writeFileSync(actual,'bound alias target');}
  symlinkSync(target,full);
 }
}
const result=value=>({status:0,signal:null,stdout:JSON.stringify(value),stderr:''});
const error=code=>({status:code==='PP6101'?5:1,signal:null,stdout:JSON.stringify({schema:'prismpm/error-result/1',diagnostic:{code}}),stderr:''});

test('closed binary source binding includes implementation, tests, schemas and owning pipeline',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);verifySource(root,expected);
 for(const path of ['crates/prismpm/src','schemas','tests/fixtures/binary/binary-program/project','scripts/binary-sdk-check.mjs','scripts/binary-sdk-check.test.mjs','.github/workflows/release.yml']){
  assert.ok(sourceRoots.includes(path));put(root,path+'/input','mutated');assert.throws(()=>verifySource(root,expected));put(root,path+'/input','source');
 }
 put(root,'crates/prismpm/src/nested/target/extra','ordinary source');assert.throws(()=>verifySource(root,expected));
});
test('source capture rejects missing members, unknown aliases and malformed closure records',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);
 for(const change of [v=>v.files.pop(),v=>v.files.push(v.files[0]),v=>v.files.reverse(),v=>v.files[0].sha256='c'.repeat(64),v=>v.extra=true]){const bad=structuredClone(expected);change(bad);assert.throws(()=>verifySource(root,bad));}
 const path=join(root,sourceRoots[0]+'/input');rmSync(path);assert.throws(()=>verifySource(root,expected));symlinkSync('/etc/hosts',path);assert.throws(()=>capture(root,revision));assert.throws(()=>capture(root,'main'));
});
test('only reviewed compiler aliases with bound target bytes are accepted',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision),path=join(root,'crates/prismpm/model');
 rmSync(path);symlinkSync('../../schemas',path);assert.throws(()=>capture(root,revision));rmSync(path);symlinkSync('../../model',path);
 put(root,'tests/hologram-oracle/Cargo.lock','changed target');assert.throws(()=>verifySource(root,expected));
});
test('native image identity rejects mutable tags, different revision and implicit mounts',()=>{
 const base=[{Os:'linux',Architecture:'amd64',RepoDigests:[image],Config:{Entrypoint:['/usr/local/bin/prismpm-devcontainer-init'],Volumes:null,Labels:{'org.opencontainers.image.revision':revision,'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM','org.opencontainers.image.version':'0.3.0'}}}];
 verifyImage(base,image,'amd64',revision);
 for(const change of [v=>v[0].Architecture='arm64',v=>v[0].Config.Labels['org.opencontainers.image.revision']='c'.repeat(40),v=>v[0].RepoDigests=[],v=>v[0].Config.Volumes={'/opt/prismpm':{}}]){const bad=structuredClone(base);change(bad);assert.throws(()=>verifyImage(bad,image,'amd64',revision));}
 assert.throws(()=>verifyImage(base,'ghcr.io/uor-foundation/prismpm-sdk:latest','amd64',revision));
});
test('immutable inventory binds actual CLI and the current source revision',t=>{
 const value=inventory(),bytes=Buffer.from(canonical(value));assert.deepEqual(inventoryEvidence(bytes,revision),{sha256:hash(bytes),cli_sha256:'c'.repeat(64)});
 for(const change of [v=>v.commands.find(row=>row.command==='prismpm').sha256='f'.repeat(64),v=>v.artifacts.find(row=>row.id==='sdk-vv-source').version='f'.repeat(40),v=>v.artifacts=v.artifacts.filter(row=>row.id!=='prismpm'),v=>v.commands.find(row=>row.command==='prismpm').executable='/tmp/prismpm',v=>v.extra=true]){const bad=structuredClone(value);change(bad);assert.throws(()=>inventoryEvidence(Buffer.from(canonical(bad)),revision));}
 const root=temporary(t);put(root,'inventory.json',bytes);put(root,'formatter.log','test-only formatter output');put(root,'format.patch','');put(root,'changed-paths.txt','');
 const invoke=()=>spawnSync(process.execPath,[new URL('./binary-sdk-check.mjs',import.meta.url).pathname,'format-evidence',root,revision,image],{encoding:'utf8'});
 assert.equal(invoke().status,0);assert.equal(JSON.parse(readFileSync(join(root,'format-result.json'))).status,'passed');
 put(root,'format.patch','test-only patch');put(root,'changed-paths.txt','crates/prismpm/src/lib.rs\n');assert.equal(invoke().status,0);assert.equal(JSON.parse(readFileSync(join(root,'format-result.json'))).status,'review-required');
 put(root,'changed-paths.txt','Cargo.lock\n');assert.notEqual(invoke().status,0);

});
test('CLI transport demands installed executable, exact result and real diagnostic exit class',()=>{
 const valid={schema:'prismpm/check-result/1',semantic_id:'a'.repeat(64),snapshot_id:'b'.repeat(64),model_id:'c'.repeat(64),entity_count:0};let called;
 cli('/tmp/fixture',['check'],{schema:valid.schema},(...args)=>{called=args;return result(valid);});
 assert.equal(called[0],'/usr/local/bin/prismpm');assert.deepEqual(called[1],['--project','/tmp/fixture','--json','check']);assert.equal(called[2].env.CARGO_NET_OFFLINE,'true');
 for(const bad of [{...result(valid),status:1},{...result(valid),signal:'SIGTERM'},result({schema:valid.schema}),result({...valid,extra:true}),{...result(valid),stdout:'noise\n{}'},{...result(valid),error:new Error('deadline')}])assert.throws(()=>cli('/tmp/fixture',['check'],{schema:valid.schema},()=>bad));
 for(const code of ['PP2001','PP5006','PP6101']){cli('/tmp/fixture',['verify'],{code},()=>error(code));assert.throws(()=>cli('/tmp/fixture',['verify'],{code},()=>({...error(code),status:0})));assert.throws(()=>cli('/tmp/fixture',['verify'],{code},()=>error('PP1001')));}
});
test('source mutations preserve canonical single-module bytes',t=>{
 const root=temporary(t),path='src/Probe.lex.tex',before='\\semanticdata{{"declarations":[{"body":{"kind":"var","name":"value"},"name":"identity"}]}}\n';put(root,path,before);
 assert.equal(mutateModule(root,module=>{module.declarations[0].body={value:[0,255],kind:'bytes'};}),before);assert.match(readFileSync(join(root,path),'utf8'),/"body":\{"kind":"bytes"/);
 put(root,path,before+before);assert.throws(()=>mutateModule(root,()=>{}));
});
test('bounded byte and tree readers reject aliases and include unexpected outputs',t=>{
 const root=temporary(t);put(root,'source','original');const before=tree(root);assert.equal(regularBytes(join(root,'source')).toString(),'original');assert.throws(()=>regularBytes(join(root,'source'),2));mkdirSync(join(root,'unexpected'));assert.notDeepEqual(tree(root),before);rmSync(join(root,'unexpected'),{recursive:true});symlinkSync('source',join(root,'alias'));assert.throws(()=>tree(root));assert.throws(()=>regularBytes(join(root,'alias')));
});
test('shared held byte reader conserves native identity and refuses external links and live replacement',t=>{
 const root=temporary(t),outside=temporary(t),path=join(root,'source');put(root,'source','original');
 assert.equal(regularBytes(new URL('file://'+path),8).toString(),'original');
 const nested=join(root,'données','子','preuve.txt');put(root,'données/子/preuve.txt','original');
 assert.equal(regularBytes(nested,8).toString(),'original');
 assert.equal(regularBytes(new URL('file://'+nested),8).toString(),'original');
 symlinkSync(join(root,'données'),join(root,'parent-alias'));
 assert.throws(()=>regularBytes(join(root,'parent-alias','子','preuve.txt'),8),/aliased/);
 rmSync(join(root,'parent-alias'));
 put(root,'empty','');assert.equal(regularBytes(join(root,'empty'),0).length,0);
 for(const limit of [-1,1.5,Number.MAX_SAFE_INTEGER,288*1024*1024+1])assert.throws(()=>regularBytes(path,limit));
 linkSync(path,join(outside,'outside-capture'));assert.throws(()=>regularBytes(path),/single-link/);assert.throws(()=>tree(root),/single-link/);rmSync(join(outside,'outside-capture'));
 const script=`import assert from 'node:assert/strict';import fs from 'node:fs';import {syncBuiltinESMExports} from 'node:module';
 const [module,path,operation]=process.argv.slice(1),{regularBytes}=await import(module),read=fs.readSync,close=fs.closeSync;let changed=false,closed=false;
 fs.readSync=(...args)=>{const count=read(...args);if(!changed){changed=true;if(operation==='link')fs.linkSync(path,path+'-link');else if(operation==='replace'){fs.renameSync(path,path+'-original');fs.writeFileSync(path,'original');}else if(operation==='read-close')throw new Error('read boundary failure');}return count;};
 fs.closeSync=fd=>{close(fd);closed=true;if(operation==='read-close')throw new Error('close boundary failure');};syncBuiltinESMExports();
 let error;try{regularBytes(path);}catch(value){error=value;}assert(error);assert(closed);
 if(operation==='read-close'){assert(error instanceof AggregateError);assert.deepEqual(error.errors.map(value=>value.message),['read boundary failure','close boundary failure']);}
 else assert.match(error.message,/file changed|single-link|held file pathname/);`;
 for(const operation of ['link','replace','read-close']){
  const selected=temporary(t);put(selected,'source','original');
  const output=spawnSync(process.execPath,['--input-type=module','-e',script,new URL('./library-sdk-check.mjs',import.meta.url).href,join(selected,'source'),operation],{encoding:'utf8',timeout:15000,maxBuffer:1024*1024});
  assert.equal(output.error,undefined);assert.equal(output.signal,null);assert.equal(output.status,0,output.stderr);
 }
});

// Coherently resealed parser fixtures are test-only and never passed to run().

test('closed binary parser rejects coherently resealed scope, vector, mode and process mutations',t=>{
 const root=temporary(t);checkAccepted(root,parserFixture(root));
 for(const change of [
  ({model})=>{model.application=null;},({model})=>{model.library={};},({model})=>{model.program.acceptance_vectors[1].request=[1];},({model})=>{model.security.assets=['unmodeled'];},
  ({acceptance})=>{acceptance.extra=true;},({acceptance})=>{delete acceptance.io_coverage;},({acceptance})=>{acceptance.io_coverage.platform='darwin';},({acceptance})=>{acceptance.io_coverage.output_write='not-exercised-empty-responses';},({acceptance})=>{acceptance.io_coverage.extra=true;},({acceptance})=>{acceptance.executions.pop();},({acceptance})=>{acceptance.executions[3].mode='cli-file';},({acceptance})=>{acceptance.executions[0].vector_count=1;},({acceptance})=>{acceptance.program.request_maximum=1024;},({acceptance})=>{acceptance.unclaimed.pop();},
  ({manifest})=>{manifest.extra=true;},({manifest})=>{manifest.processes.pop();},({manifest})=>{manifest.processes.find(row=>row.tool==='binary-std-acceptance').stdout='[]\n';},({manifest})=>{manifest.processes.at(-1).stdout='[]\n';},({manifest})=>{const record=manifest.processes.at(-1),value=JSON.parse(record.stdout);delete value.io_coverage;record.stdout=JSON.stringify(value);},({manifest})=>{const record=manifest.processes.at(-1),value=JSON.parse(record.stdout);value.io_coverage.output_write='not-exercised-empty-responses';record.stdout=JSON.stringify(value);},({manifest})=>{manifest.processes.find(row=>row.tool==='binary-cli-allocation-acceptance').stdout='test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s\n';},({manifest})=>{manifest.processes.find(row=>row.tool==='binary-cli-allocation-acceptance').argv=[];},({manifest})=>{manifest.processes[0].exit_code=1;},({manifest})=>{manifest.processes[0].argv=[42];},({manifest})=>{manifest.processes[0].executable_sha256='invalid';},
  ({paths})=>{paths.pop();},({paths})=>{paths.push('binary/extra.json');},({inputs})=>{inputs.schema='prismpm/build-inputs/3';},({inputs})=>{inputs.extra=true;},({binding})=>{binding.program.acceptance_vectors.pop();},
 ]){const bad=temporary(t);assert.throws(()=>checkAccepted(bad,parserFixture(bad,change)));}
});
test('complete proof and byte-bound acceptance reject missing changed or resealed evidence',t=>{
 for(const change of [({lex})=>{lex.lexlean.compiler_semantics='9'.repeat(64);},({lex})=>{lex.build_manifest.sha256='9'.repeat(64);},({lex})=>{lex.declarations.pop();},({lex})=>{lex.declarations.push(lex.declarations[0]);},({lex})=>{lex.declarations[0].result='failed';},({lex})=>{lex.declarations[0].observed=['sorryAx'];},({lex,modules})=>{lex.declarations[0].policy={kind:'allow',axioms:['sorryAx']};modules[0].declarations[0].axiom_policy=lex.declarations[0].policy;},({acceptance})=>{acceptance.lexlean_attestation_id='f'.repeat(64);}]){const root=temporary(t);assert.throws(()=>checkAccepted(root,parserFixture(root,change)));}
 const root=temporary(t),receipt=parserFixture(root);
 for(const name of ['lexlean-attestation.json','binary-acceptance.json'])rejectChangedEvidence(root,receipt,join(root,receipt.verified_root,name));
 rejectChangedEvidence(root,receipt,join(root,'.prism/build',receipt.build_id,'binary/package/src/lib.rs'));
 for(const change of [(r,v)=>rmSync(join(r,'.prism/build',v.build_id,'manifest.json')),(r,v)=>put(r,'.prism/build/'+v.build_id+'/unexpected','extra')]){const root=temporary(t),receipt=parserFixture(root);change(root,receipt);assert.throws(()=>checkAccepted(root,receipt));}
 {const aliased=temporary(t),receipt=parserFixture(aliased);renameSync(join(aliased,'.prism/build'),join(aliased,'outside-build'));symlinkSync('../outside-build',join(aliased,'.prism/build'));assert.throws(()=>checkAccepted(aliased,receipt),/directory alias/);}
 for(const value of [{},{schema:'prismpm/verify-result/1',build_id:'a'.repeat(64),attestation_id:'b'.repeat(64),verified_root:'../../outside'}])assert.throws(()=>checkAccepted(root,value));
});
test('binary current acceptance binds both complete owned phases to independent compiler source and original replay',t=>{
 const valid=temporary(t),receipt=parserFixture(valid);checkAccepted(valid,receipt);
 for(const authority of [undefined,{...sourceAuthorityFixture,archive_sha256:'9'.repeat(64)},{...sourceAuthorityFixture,toolchain:'latest'}])assert.throws(()=>checkWithAuthority(valid,receipt,authority));
 for(const change of [
  owner=>{owner.phases.pop();},owner=>{owner.phases.reverse();},owner=>{owner.phases[0].role='library';},
  owner=>{owner.phases[0].model_sha256='9'.repeat(64);},owner=>{for(const phase of owner.phases)phase.lexlean_manifest_sha256='9'.repeat(64);},
  owner=>{for(const phase of owner.phases)phase.artifacts.pop();},owner=>{for(const phase of owner.phases)phase.processes.pop();},
  owner=>{owner.phases[0].processes[2].exporter.source_archive_sha256='9'.repeat(64);},
  owner=>{for(const phase of owner.phases)phase.processes[2].argv[3]='Wrong.Module';},
  owner=>{for(const phase of owner.phases)phase.processes[0].executable_sha256='9'.repeat(64);},
  owner=>{for(const phase of owner.phases)phase.processes[4].argv=['generate-lockfile'];},
  owner=>{for(const phase of owner.phases)phase.processes[5].executable_sha256='9'.repeat(64);},
  owner=>{owner.phases[1].processes[3].stdout='invented replay';},
  (owner,manifest)=>{manifest.schema='prismpm/binary-verification-manifest/1';delete manifest.exporter_owner;},
 ]){const root=temporary(t);assert.throws(()=>checkAccepted(root,parserFixture(root,undefined,change)));}
});
test('outer result requires current image inventory source and every non-skipped owning test',()=>{
 const sourceSha='1'.repeat(64),inventorySha='2'.repeat(64),args=[image,revision,'amd64',sourceSha,inventorySha];
 const value={schema:'prismpm/installed-binary-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:image,source_revision:revision,source_sha256:sourceSha,inventory_sha256:inventorySha,architecture:'amd64',build_id:'3'.repeat(64),attestation_id:'4'.repeat(64),checks:[...completedChecks],unclaimed:[...unclaimed]};verifyResult(value,...args);
 for(const change of [v=>v.checks.pop(),v=>v.checks.reverse(),v=>v.extra=true,v=>v.scope='production-release',v=>v.inventory_sha256='9'.repeat(64),v=>v.source_revision='9'.repeat(40),v=>v.architecture='arm64']){const bad=structuredClone(value);change(bad);assert.throws(()=>verifyResult(bad,...args));}
 const tap='TAP version 13\n'+Array.from({length:20},(_,i)=>'ok '+(i+1)+' - gate '+i+'\n').join('')+'1..20\n# tests 20\n# suites 0\n# pass 20\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n';testOutput({status:0,signal:null,stdout:tap});for(const stdout of ['',tap.replace('# skipped 0','# skipped 1'),tap.replace('# tests 20','# tests 18')])assert.throws(()=>testOutput({status:0,signal:null,stdout}));
});
test('release retains existing mandatory gates and adds binary on both native platforms',()=>{
 const workflow=readFileSync(new URL('../.github/workflows/release.yml',import.meta.url),'utf8');
 for(const name of ['browser-api','library','binary','product'])assert.ok(workflow.includes('/scripts/'+name+'-sdk-check.sh'));
 assert.match(workflow,/name: Verify exact current SDK binary-package closure offline and read-only\n\s+if: matrix.image == 'sdk'/);assert.match(workflow,/os: ubuntu-24.04-arm/);assert.match(workflow,/name: binary-sdk-\$\{\{ matrix.name \}\}/);
 assert.match(workflow,/name: binary-sdk-sdk-amd64/);assert.match(workflow,/name: binary-sdk-sdk-arm64/);assert.match(workflow,/\$\{\{ matrix.name \}\}-binary-sdk\/\n          include-hidden-files: true/);
 const wrapper=readFileSync(new URL('./binary-sdk-check.sh',import.meta.url),'utf8');for(const required of ['--user 1000:1000 --read-only --network none','--cap-drop ALL','--security-opt no-new-privileges','--tmpfs /tmp:rw,exec,nosuid,nodev,size=8g','PRISMPM_EPHEMERAL_HOME=1','node scripts/binary-sdk-check.mjs run'])assert.ok(wrapper.includes(required));assert.ok(!wrapper.includes('--volume')&&!wrapper.includes('--mount'));
 const qualification=readFileSync(new URL('../.github/workflows/binary-sdk-qualification.yml',import.meta.url),'utf8');
 for(const required of ['branches: [main]','branches: [main, work/compression-m0-reconcile]','runner: ubuntu-24.04','runner: ubuntu-24.04-arm','contents: read','persist-credentials: false','needs: source-format','bash scripts/binary-sdk-format.sh','include-hidden-files: true','ref: ${{ github.event.pull_request.head.sha || github.sha }}','node scripts/binary-sdk-check.mjs tests','node scripts/sdk-image-inputs.mjs build','bash scripts/binary-sdk-qualify.sh'])assert.ok(qualification.includes(required),required);
 for(const forbidden of ['packages: write','contents: write','--push','push: true','continue-on-error'])assert.ok(!qualification.includes(forbidden),forbidden);
 // Supersede old commits on the same ref without cancelling matrix siblings.
 // Source and evidence identities remain commit-bound, not concurrency-bound.
 const binaryCapacity=value=>{
  // PR updates qualify the exact head once; main pushes qualify the merged source.
  assert.match(value,/^on:\n  push:\n    branches: \[main\]\n  pull_request:\n    branches: \[main, work\/compression-m0-reconcile\]\n    paths:\n/m);
  assert.deepEqual(value.match(/^concurrency:\n(?:  [^\n]+\n)+/gm),[
   'concurrency:\n  group: binary-sdk-${{ github.repository }}-${{ github.ref }}\n  cancel-in-progress: true\n',
  ]);
  const sections=value.split('\njobs:\n');assert.equal(sections.length,2);
  assert.deepEqual([...sections[0].matchAll(/^  ([a-z][a-z_-]+):\n/gm)].map(match=>match[1]),
   ['push','pull_request','workflow_dispatch']);
  assert.deepEqual([...sections[1].matchAll(/^  ([a-z][a-z_-]+):\n/gm)].map(match=>match[1]),
   ['source-format','native-binary']);
  assert.deepEqual([...value.matchAll(/^    timeout-minutes: (\d+)$/gm)].map(match=>Number(match[1])),[30,180]);
  assert.deepEqual(value.match(/^    needs: .+$/gm),['    needs: source-format']);
  assert.match(value,/^    strategy:\n      fail-fast: false\n      max-parallel: 2\n      matrix:\n        include:\n          - runner: ubuntu-24.04\n            architecture: amd64\n          - runner: ubuntu-24.04-arm\n            architecture: arm64\n    runs-on: \$\{\{ matrix.runner \}\}\n/m);
  for(const field of ['SOURCE_REVISION:','ref:'])assert.equal(value.split(field+' ${{ github.event.pull_request.head.sha || github.sha }}').length-1,2);
  assert.deepEqual([...value.matchAll(/^      - uses: (.+)$/gm)].map(match=>match[1]),[
   'actions/checkout@11d5960a326750d5838078e36cf38b85af677262',
   'actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38',
   'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02',
   'actions/checkout@11d5960a326750d5838078e36cf38b85af677262',
   'actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38',
   'docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f',
   'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02',
  ]);
  const runs=[...value.matchAll(/^        run: \|\n((?:          [^\n]*\n)+)/gm)].map(match=>match[1]);
  assert.equal(runs.length,4);
  for(const [index,command] of [
   [0,'bash scripts/binary-sdk-format.sh "$SOURCE_REVISION" "$RUNNER_TEMP/binary-format-review"'],
   [1,'test "$(git rev-parse HEAD)" = "$SOURCE_REVISION"'],
   [1,'test -z "$(git status --porcelain)"'],
   [1,'bash -n scripts/binary-sdk-check.sh scripts/binary-sdk-qualify.sh'],
   [1,'node scripts/binary-sdk-check.mjs tests'],
   [1,'bash scripts/sdk-candidate.sh install-oras "$RUNNER_TEMP/binary-tools"'],
   [2,'node scripts/sdk-image-inputs.mjs build . "$SOURCE_REVISION" runtime'],
   [2,'--platform "linux/$ARCHITECTURE"'],
   [2,'--provenance=false --sbom=false --build-arg SOURCE_DATE_EPOCH=0'],
   [3,'bash scripts/binary-sdk-qualify.sh'],
   [3,'"$RUNNER_TEMP/binary-qualification/sdk.oci.tar" "$ARCHITECTURE" "$SOURCE_REVISION"'],
   [3,'"$RUNNER_TEMP/binary-qualification/evidence"'],
  ])assert.ok(runs[index].includes(command),command);
  for(const run of runs)assert.ok(run.startsWith('          set -euo pipefail\n'));
  assert.doesNotMatch(value,/continue-on-error:|^    if:|^    concurrency:/m);
 };
 binaryCapacity(qualification);
 for(const [before,after] of [
  ['  push:\n    branches: [main]\n','  push:\n    branches: [work/binary-program-prerequisite]\n'],
  ['  push:\n    branches: [main]\n','  push:\n    branches: [main, work/binary-program-prerequisite]\n'],
  ['  pull_request:\n    branches: [main, work/compression-m0-reconcile]\n','  pull_request:\n    branches: [main]\n'],
  ['  pull_request:\n    branches: [main, work/compression-m0-reconcile]\n','  pull_request:\n    branches: [work/compression-m0-reconcile]\n'],
  ['  pull_request:\n','  pull_request_target:\n'],
  ['  workflow_dispatch:\n',''],
  ['binary-sdk-${{ github.repository }}-${{ github.ref }}','binary-sdk-${{ github.sha }}'],
  ['binary-sdk-${{ github.repository }}-${{ github.ref }}','binary-sdk-${{ github.run_id }}'],
  ['cancel-in-progress: true','cancel-in-progress: false'],
  ['    name: Binary package /','    concurrency: binary-sdk-${{ github.repository }}-${{ github.ref }}\n    name: Binary package /'],
  ['      max-parallel: 2\n',''],['max-parallel: 2','max-parallel: 3'],
  ['fail-fast: false','fail-fast: true'],['architecture: arm64','architecture: amd64'],
  ['needs: source-format','needs: skipped-format'],['timeout-minutes: 180','timeout-minutes: 360'],
  ['SOURCE_REVISION: ${{ github.event.pull_request.head.sha || github.sha }}','SOURCE_REVISION: main'],
  ['ref: ${{ github.event.pull_request.head.sha || github.sha }}','ref: main'],
  ['node scripts/binary-sdk-check.mjs tests','echo omitted owning tests'],
  ['--platform "linux/$ARCHITECTURE"','--platform linux/amd64'],
  ['bash scripts/binary-sdk-qualify.sh','echo omitted native qualification'],
  ['actions/checkout@11d5960a326750d5838078e36cf38b85af677262','actions/checkout@main'],
 ]){assert.ok(qualification.includes(before));assert.throws(()=>binaryCapacity(qualification.replace(before,after)),undefined,'policy mutation: '+before);}
 const formatter=readFileSync(new URL('./binary-sdk-format.sh',import.meta.url),'utf8');
 for(const required of ['ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21','--user 1000:1000 --read-only --network none','--tmpfs /tmp:rw,exec,nosuid,nodev,size=4g','PRISMPM_EPHEMERAL_HOME=1',': > /tmp/prismpm-format-ready; exec sleep infinity','docker exec "$container" test -f /tmp/prismpm-format-ready','cargo fmt --check','if test -s "$evidence/format.patch"; then'])assert.ok(formatter.includes(required),required);
 assert.ok(!formatter.includes('cargo fmt --all'));
 assert.ok(!formatter.includes('--mount')&&!formatter.includes('--volume')&&!formatter.includes('--entrypoint')&&!formatter.includes('git push'));
 const transport=readFileSync(new URL('./binary-sdk-qualify.sh',import.meta.url),'utf8');
 for(const required of ['--publish 127.0.0.1::5000','--from-oci-layout --to-plain-http','"$endpoint/sdk@$digest"','"$root/scripts/binary-sdk-check.sh" "$image" "$revision" "$evidence/accepted"'])assert.ok(transport.includes(required),required);
 for(const name of ['ci-parallel','reproducibility']){
  const value=readFileSync(new URL('../.github/workflows/'+name+'.yml',import.meta.url),'utf8');
  const unfiltered=workflow=>{assert.match(workflow,/^  pull_request:\n(?=  [a-z]|\n\S)/m);assert.doesNotMatch(workflow,/^  pull_request_target:/m);};
  unfiltered(value);assert.throws(()=>unfiltered(value.replace('  pull_request:\n','  pull_request:\n    branches: [main]\n')));
  assert.throws(()=>unfiltered(value.replace('  pull_request:\n','  pull_request:\n    branches: [main, work/compression-m0-reconcile]\n')));
 }
 const xtask=readFileSync(new URL('../xtask/src/main.rs',import.meta.url),'utf8');for(const path of ['scripts/binary-sdk-check.test.mjs','scripts/binary-sdk-check-shell.test.mjs'])assert.ok(xtask.includes('"'+path+'"'));
});
test('generated review is deliberate, exact-source, writer-owned and never acceptance',()=>{
 const workflow=readFileSync(new URL('../.github/workflows/binary-generated-review.yml',import.meta.url),'utf8');validateGenerationWorkflow(workflow);
 for(const [before,after] of [
  ['work/binary-generated-review','work/binary-program-prerequisite'],['contents: read','contents: write'],
  ['test "$SOURCE_REVISION" = "$GITHUB_SHA"','true'],['just golden-check','echo skipped golden'],
  ['just stdlib-package-check','echo skipped stdlib'],['just check-fixtures','echo skipped fixtures'],
  ['push: never','push: always'],['devcontainers/ci@513af61f4de4f75d37e4438f184ba4358f0fc1ca','devcontainers/ci@main'],
 ]){assert.ok(workflow.includes(before));assert.throws(()=>validateGenerationWorkflow(workflow.replace(before,after)));}
});
test('source acquisition validates every immutable archive member before writing',t=>{
 const makeArchive=`import copy,io,json,pathlib,sys,tarfile
root=pathlib.Path(sys.argv[1]); mode=sys.argv[2]
manifest=json.loads((root/'source.json').read_text()); rows=[]
for row in manifest['files']:
    info=tarfile.TarInfo(row['path'].rstrip('/')); payload=None
    if row['kind']=='directory': info.type=tarfile.DIRTYPE;info.mode=0o555
    elif row['kind']=='symlink': info.type=tarfile.SYMTYPE;info.linkname=row['target']
    else: info.type=tarfile.REGTYPE;info.mode=0o444;payload=b'abc';info.size=len(payload)
    rows.append((info,payload))
if mode=='duplicate': rows.append(copy.deepcopy(rows[2]))
elif mode=='omitted': rows.pop(2)
elif mode=='extra': info=tarfile.TarInfo('picked/extra');rows.append((info,b''))
elif mode=='traversal': rows[2][0].name='../escape'
elif mode=='absolute': rows[2][0].name='/escape'
elif mode=='tampered': rows[2]=(rows[2][0],b'bad')
elif mode=='wrong-size': rows[2][0].size=4;rows[2]=(rows[2][0],b'abcd')
elif mode=='wrong-type': rows[2][0].type=tarfile.DIRTYPE;rows[2][0].size=0;rows[2]=(rows[2][0],None)
elif mode=='hardlink': rows[2][0].type=tarfile.LNKTYPE;rows[2][0].size=0;rows[2][0].linkname='picked/link';rows[2]=(rows[2][0],None)
elif mode=='fifo': rows[2][0].type=tarfile.FIFOTYPE;rows[2][0].size=0;rows[2]=(rows[2][0],None)
elif mode=='changed-alias': rows[3][0].linkname='../../escape'
elif mode=='omitted-root':
    manifest['files'].append(dict(manifest['files'][2],path='elsewhere'))
    (root/'source.json').write_text(json.dumps(manifest))
elif mode=='alias-descendant':
    info=tarfile.TarInfo('picked/link/child');info.size=3;rows.append((info,b'abc'))
    manifest['files'].append(dict(manifest['files'][2],path='picked/link/child'))
    (root/'source.json').write_text(json.dumps(manifest))
with tarfile.open(root/'source.tar','w',format=tarfile.PAX_FORMAT) as archive:
    for info,payload in rows: archive.addfile(info,io.BytesIO(payload) if payload is not None else None)
`;
 const failures={duplicate:'duplicate source archive member',omitted:'source archive members omitted',extra:'unexpected source archive member',traversal:'unsafe source member path',absolute:'unsafe source member path',tampered:'source archive member digest differs','wrong-size':'source archive member size differs','wrong-type':'source archive member type differs',hardlink:'special source archive member refused',fifo:'special source archive member refused','changed-alias':'source archive alias differs','alias-descendant':'source member beneath symlink ancestor','destination-alias':'source destination already exists','omitted-root':'complete non-overlapping source roots required','duplicate-root':'duplicate selected source root','overlapping-root':'complete non-overlapping source roots required'};
 for(const mode of ['valid',...Object.keys(failures)]){
  const root=temporary(t),destination=join(root,'copy');mkdirSync(destination);
  put(root,'source.json',canonical({revision,files:[{path:'picked/',kind:'directory'},{path:'picked/nested/',kind:'directory'},{path:'picked/nested/input',kind:'file',size:3,sha256:hash('abc')},{path:'picked/link',kind:'symlink',target:'nested/input'}]}));
  const packed=spawnSync('python3',['-c',makeArchive,root,mode],{encoding:'utf8'});assert.equal(packed.status,0,packed.stderr);
  if(mode==='destination-alias'){mkdirSync(join(root,'outside'));symlinkSync(join(root,'outside'),join(destination,'picked'));}
  const args=[new URL('./binary-sdk-source-extract.py',import.meta.url).pathname,destination,join(root,'source.json'),join(root,'source.tar'),'picked'];if(['duplicate-root','overlapping-root'].includes(mode))args.push(join(root,'source.tar'),mode==='duplicate-root'?'picked':'picked/nested');
  const output=spawnSync('python3',args,{encoding:'utf8'});assert.equal(output.error,undefined);assert.equal(output.signal,null);
  if(mode==='valid'){assert.equal(output.status,0,output.stderr);assert.equal(readFileSync(join(destination,'picked/nested/input'),'utf8'),'abc');assert.equal(readFileSync(join(destination,'picked/link'),'utf8'),'abc');}
  else{assert.equal(output.status,1,mode);assert.ok(output.stderr.endsWith('ValueError: '+failures[mode]+'\n'),mode+': '+output.stderr);assert.deepEqual(readdirSync(destination),mode==='destination-alias'?['picked']:[],mode+' must fail before writing');if(mode==='destination-alias')assert.deepEqual(readdirSync(join(root,'outside')),[]);}
 }
});
test('owning parser tests kill an omitted process-exit guard',t=>{
 const root=temporary(t),module=readFileSync(new URL('./binary-sdk-check.mjs',import.meta.url),'utf8'),guard='assert.equal(row.exit_code,0);';assert.equal(module.split(guard).length,2);
 for(const name of ['binary-sdk-check.mjs','binary-sdk-check.test.mjs','binary-sdk-check-fixtures.mjs','library-sdk-check.mjs','browser-api-sdk-check.mjs'])put(root,'scripts/'+name,name==='binary-sdk-check.mjs'?module.replace(guard,''):readFileSync(new URL('./'+name,import.meta.url)));
 for(const name of ['library-sdk-fixture.mjs','installed-exporter-concurrency.mjs','sdk-command-owner.mjs','sdk-command-owner.py'])put(root,'scripts/'+name,readFileSync(new URL('./'+name,import.meta.url)));
 cpSync(new URL('../sdk',import.meta.url),join(root,'sdk'),{recursive:true});
 const env={...process.env};delete env.NODE_TEST_CONTEXT;const output=spawnSync(process.execPath,['--test','--test-reporter=tap','--test-name-pattern=closed binary parser rejects',join(root,'scripts/binary-sdk-check.test.mjs')],{encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});assert.equal(output.error,undefined);assert.equal(output.status,1);assert.match(output.stdout,/Missing expected exception/);
});
