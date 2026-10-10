// Synthetic parser/boundary tests are not installed CLI acceptance evidence.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,renameSync,symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {test} from 'node:test';
import {capture,verifySource,sourceRoots,sourceAliases,cli,checkAccepted,mutateModule,tree,verifyImage,verifyResult,testOutput,hash,canonical,fixtureProgram,modes,processTools,binaryPaths,completedChecks,unclaimed,inventoryEvidence,regularBytes,rejectChangedEvidence} from './binary-sdk-check.mjs';
import {inventory,parserFixture} from './binary-sdk-check-fixtures.mjs';
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
test('outer result requires current image inventory source and every non-skipped owning test',()=>{
 const sourceSha='1'.repeat(64),inventorySha='2'.repeat(64),args=[image,revision,'amd64',sourceSha,inventorySha];
 const value={schema:'prismpm/installed-binary-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:image,source_revision:revision,source_sha256:sourceSha,inventory_sha256:inventorySha,architecture:'amd64',build_id:'3'.repeat(64),attestation_id:'4'.repeat(64),checks:[...completedChecks],unclaimed:[...unclaimed]};verifyResult(value,...args);
 for(const change of [v=>v.checks.pop(),v=>v.checks.reverse(),v=>v.extra=true,v=>v.scope='production-release',v=>v.inventory_sha256='9'.repeat(64),v=>v.source_revision='9'.repeat(40),v=>v.architecture='arm64']){const bad=structuredClone(value);change(bad);assert.throws(()=>verifyResult(bad,...args));}
 const tap='TAP version 13\n'+Array.from({length:16},(_,i)=>'ok '+(i+1)+' - gate '+i+'\n').join('')+'1..16\n# tests 16\n# suites 0\n# pass 16\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n';testOutput({status:0,signal:null,stdout:tap});for(const stdout of ['',tap.replace('# skipped 0','# skipped 1'),tap.replace('# tests 16','# tests 14')])assert.throws(()=>testOutput({status:0,signal:null,stdout}));
});
test('release retains existing mandatory gates and adds binary on both native platforms',()=>{
 const workflow=readFileSync(new URL('../.github/workflows/release.yml',import.meta.url),'utf8');
 for(const name of ['browser-api','library','binary','product'])assert.ok(workflow.includes('/scripts/'+name+'-sdk-check.sh'));
 assert.match(workflow,/name: Verify exact current SDK binary-package closure offline and read-only\n\s+if: matrix.image == 'sdk'/);assert.match(workflow,/os: ubuntu-24.04-arm/);assert.match(workflow,/name: binary-sdk-\$\{\{ matrix.name \}\}/);
 assert.match(workflow,/name: binary-sdk-sdk-amd64/);assert.match(workflow,/name: binary-sdk-sdk-arm64/);assert.match(workflow,/\$\{\{ matrix.name \}\}-binary-sdk\/\n          include-hidden-files: true/);
 const wrapper=readFileSync(new URL('./binary-sdk-check.sh',import.meta.url),'utf8');for(const required of ['--user 1000:1000 --read-only --network none','--cap-drop ALL','--security-opt no-new-privileges','--tmpfs /tmp:rw,exec,nosuid,nodev,size=8g','PRISMPM_EPHEMERAL_HOME=1','node scripts/binary-sdk-check.mjs run'])assert.ok(wrapper.includes(required));assert.ok(!wrapper.includes('--volume')&&!wrapper.includes('--mount'));
 const qualification=readFileSync(new URL('../.github/workflows/binary-sdk-qualification.yml',import.meta.url),'utf8');
 for(const required of ['branches: [work/binary-program-prerequisite]','branches: [main, work/compression-m0-reconcile]','runner: ubuntu-24.04','runner: ubuntu-24.04-arm','contents: read','persist-credentials: false','needs: source-format','bash scripts/binary-sdk-format.sh','include-hidden-files: true','ref: ${{ github.event.pull_request.head.sha || github.sha }}','node scripts/binary-sdk-check.mjs tests','node scripts/sdk-image-inputs.mjs build','bash scripts/binary-sdk-qualify.sh'])assert.ok(qualification.includes(required),required);
 for(const forbidden of ['packages: write','contents: write','--push','push: true','continue-on-error'])assert.ok(!qualification.includes(forbidden),forbidden);
 const formatter=readFileSync(new URL('./binary-sdk-format.sh',import.meta.url),'utf8');
 for(const required of ['ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21','--user 1000:1000 --read-only --network none','--tmpfs /tmp:rw,exec,nosuid,nodev,size=4g','PRISMPM_EPHEMERAL_HOME=1',': > /tmp/prismpm-format-ready; exec sleep infinity','docker exec "$container" test -f /tmp/prismpm-format-ready','cargo fmt --all --check','if test -s "$evidence/format.patch"; then'])assert.ok(formatter.includes(required),required);
 assert.ok(!formatter.includes('--mount')&&!formatter.includes('--volume')&&!formatter.includes('--entrypoint')&&!formatter.includes('git push'));
 const transport=readFileSync(new URL('./binary-sdk-qualify.sh',import.meta.url),'utf8');
 for(const required of ['--publish 127.0.0.1::5000','--from-oci-layout --to-plain-http','"$endpoint/sdk@$digest"','"$root/scripts/binary-sdk-check.sh" "$image" "$revision" "$evidence/accepted"'])assert.ok(transport.includes(required),required);
 for(const name of ['ci-parallel','reproducibility'])assert.match(readFileSync(new URL('../.github/workflows/'+name+'.yml',import.meta.url),'utf8'),/pull_request:\n    branches: \[main, work\/compression-m0-reconcile\]/);
 const xtask=readFileSync(new URL('../xtask/src/main.rs',import.meta.url),'utf8');for(const path of ['scripts/binary-sdk-check.test.mjs','scripts/binary-sdk-check-shell.test.mjs'])assert.ok(xtask.includes('"'+path+'"'));
});
test('owning parser tests kill an omitted process-exit guard',t=>{
 const root=temporary(t),module=readFileSync(new URL('./binary-sdk-check.mjs',import.meta.url),'utf8'),guard='assert.equal(row.exit_code,0);';assert.equal(module.split(guard).length,2);
 for(const name of ['binary-sdk-check.mjs','binary-sdk-check.test.mjs','binary-sdk-check-fixtures.mjs','library-sdk-check.mjs','browser-api-sdk-check.mjs'])put(root,'scripts/'+name,name==='binary-sdk-check.mjs'?module.replace(guard,''):readFileSync(new URL('./'+name,import.meta.url)));
 put(root,'sdk/platform-lock.mjs',readFileSync(new URL('../sdk/platform-lock.mjs',import.meta.url)));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;const output=spawnSync(process.execPath,['--test','--test-reporter=tap','--test-name-pattern=closed binary parser rejects',join(root,'scripts/binary-sdk-check.test.mjs')],{encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});assert.equal(output.error,undefined);assert.equal(output.status,1);assert.match(output.stdout,/Missing expected exception/);
});
