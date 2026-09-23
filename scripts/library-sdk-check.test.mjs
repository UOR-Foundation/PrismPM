import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,renameSync,symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {test} from 'node:test';
import {capture,verifySource,sourceRoots,sourceAliases,cli,checkAccepted,mutateModule,tree,verifyImage,verifyResult,testOutput} from './library-sdk-check.mjs';

const revision='a'.repeat(40),image='ghcr.io/uor-foundation/prismpm-sdk@sha256:'+'b'.repeat(64);
const temporary=t=>{const root=mkdtempSync(join(tmpdir(),'prismpm-library-gate-test-'));t.after(()=>rmSync(root,{recursive:true,force:true}));return root;};
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
const error=(code,message)=>({status:code==='PP6101'?5:1,signal:null,stdout:JSON.stringify({schema:'prismpm/error-result/1',diagnostic:{code,message}}),stderr:''});

test('closed shipped-library source binding includes implementation, fixtures, schemas and gate itself',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);verifySource(root,expected);
 for(const path of ['crates/prismpm/src','crates/prismpm/schemas','tests/fixtures/library/native-library/project','scripts/library-sdk-check.mjs','scripts/library-sdk-check.test.mjs']){
  assert.ok(sourceRoots.includes(path));put(root,path+'/input','mutated');assert.throws(()=>verifySource(root,expected),/source closure/);put(root,path+'/input','source');
 }
 put(root,'crates/prismpm/src/nested/target/extra','ordinary source');assert.throws(()=>verifySource(root,expected),/source closure/);
});

test('source capture refuses symlinks, missing members and duplicate or altered evidence',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision);
 for(const change of [v=>v.files.pop(),v=>v.files.push(v.files[0]),v=>v.files.reverse(),v=>v.files[0].sha256='c'.repeat(64),v=>v.extra=true]){
  const changed=structuredClone(expected);change(changed);assert.throws(()=>verifySource(root,changed));
 }
 const path=join(root,sourceRoots[0]+'/input');rmSync(path);assert.throws(()=>verifySource(root,expected));symlinkSync('/etc/hosts',path);assert.throws(()=>capture(root,revision),/regular|symlink/);
 assert.throws(()=>capture(root,'main'));
});

test('only the exact reviewed source aliases and their canonical target bytes are accepted',t=>{
 const root=temporary(t);source(root);const expected=capture(root,revision),path=join(root,'crates/prismpm/model');
 rmSync(path);symlinkSync('../../schemas',path);assert.throws(()=>capture(root,revision),/changed source alias/);rmSync(path);symlinkSync('../../model',path);
 put(root,'tests/hologram-oracle/Cargo.lock','changed target');assert.throws(()=>verifySource(root,expected),/source closure/);
});

test('the immutable installed image must identify the exact current native source',()=>{
 const base=[{Os:'linux',Architecture:'amd64',RepoDigests:[image],Config:{Entrypoint:['/usr/local/bin/prismpm-devcontainer-init'],Volumes:null,Labels:{'org.opencontainers.image.revision':revision,'org.opencontainers.image.source':'https://github.com/UOR-Foundation/PrismPM','org.opencontainers.image.version':'0.3.0'}}}];
 verifyImage(base,image,'amd64',revision);
 for(const change of [v=>v[0].Architecture='arm64',v=>v[0].Config.Labels['org.opencontainers.image.revision']='c'.repeat(40),v=>v[0].RepoDigests=[],v=>v[0].Config.Volumes={'/opt/prismpm':{}}]){const bad=structuredClone(base);change(bad);assert.throws(()=>verifyImage(bad,image,'amd64',revision));}
 assert.throws(()=>verifyImage(base,'ghcr.io/uor-foundation/prismpm-sdk:latest','amd64',revision));
});

test('CLI transport invokes the installed executable and rejects failed or malformed success',()=>{
 const valid={schema:'prismpm/check-result/1',semantic_id:'a'.repeat(64),snapshot_id:'b'.repeat(64),model_id:'c'.repeat(64),entity_count:0};
 let called;const launch=(...args)=>{called=args;return result(valid);};
 cli('/tmp/fixture',['check'],{schema:'prismpm/check-result/1'},launch);
 assert.equal(called[0],'/usr/local/bin/prismpm');assert.deepEqual(called[1],['--project','/tmp/fixture','--json','check']);
 assert.equal(called[2].env.CARGO_NET_OFFLINE,'true');assert.ok(called[2].timeout>0);assert.ok(called[2].maxBuffer<=16*1024*1024);
 for(const bad of [{...result(valid),status:1},{...result(valid),signal:'SIGTERM'},result({schema:'prismpm/check-result/1'}),result({...valid,extra:true}),result({...valid,entity_count:-1}),result({schema:'prismpm/verify-result/1'}),{...result({}),stdout:'noise\n{}'}, {...result({}),error:new Error('deadline')}])assert.throws(()=>cli('/tmp/fixture',['check'],{schema:'prismpm/check-result/1'},()=>bad));
 const buildId='d'.repeat(64),attestation='e'.repeat(64);
 for(const success of [valid,{schema:'prismpm/build-result/1',build_id:buildId,source_id:'a'.repeat(64),semantic_id:'b'.repeat(64),model_path:'.prism/build/'+buildId+'/model.prism.json',manifest_path:'.prism/build/'+buildId+'/manifest.json'},{schema:'prismpm/verify-result/1',build_id:buildId,attestation_id:attestation,verified_root:'.prism/verified/'+attestation}]){
  const expected={schema:success.schema};cli('/tmp/fixture',['check'],expected,()=>result(success));
  for(const field of Object.keys(success)){const changed={...success};delete changed[field];assert.throws(()=>cli('/tmp/fixture',['check'],expected,()=>result(changed)));}
  assert.throws(()=>cli('/tmp/fixture',['check'],expected,()=>result({...success,extra:true})));
 }
});

test('negative CLI probes require the owning diagnostic and expected exit class',()=>{
 cli('/tmp/fixture',['check'],{code:'PP2001',message:'facet closure is not exact'},()=>error('PP2001','facet closure is not exact'));
 cli('/tmp/fixture',['verify'],{code:'PP5006'},()=>error('PP5006','modeled acceptance failed'));
 cli('/tmp/fixture',['build','--locked','-t','ghcr.io/uor-foundation/prismpm-library-probe:0.1.0'],{code:'PP6101'},()=>error('PP6101','native library cannot release'));
 for(const bad of [result({schema:'prismpm/error-result/1',diagnostic:{code:'PP2001',message:'facet closure is not exact'}}),error('PP1001','facet closure is not exact'),error('PP2001','different failure'),{...error('PP2001','facet closure is not exact'),status:101}])assert.throws(()=>cli('/tmp/fixture',['check'],{code:'PP2001',message:'facet closure is not exact'},()=>bad));
});

test('source mutation preserves canonical LexLean module structure and changes the real identity body',t=>{
 const root=temporary(t),path='src/Probe.lex.tex',before='\\semanticdata{{"declarations":[{"body":{"kind":"var","name":"value"},"name":"identity"}]}}\n';put(root,path,before);
 mutateModule(root,module=>{module.declarations[0].body={right:{kind:'nat',value:'1'},left:{kind:'var',name:'value'},kind:'add'};});
 const after=readFileSync(join(root,path),'utf8');assert.match(after,/"body":\{"kind":"add","left"/);assert.notEqual(after,before);
 put(root,path,before+before);assert.throws(()=>mutateModule(root,()=>{}));
});

test('tree comparison includes empty directories, extra outputs and byte changes',t=>{
 const root=temporary(t);put(root,'source','original');const before=tree(root);mkdirSync(join(root,'unexpected'));assert.notDeepEqual(tree(root),before);rmSync(join(root,'unexpected'),{recursive:true});mkdirSync(join(root,'.prism'));assert.notDeepEqual(tree(root),before);rmSync(join(root,'.prism'),{recursive:true});put(root,'source','changed');assert.notDeepEqual(tree(root),before);symlinkSync('source',join(root,'alias'));assert.throws(()=>tree(root));
});

test('library evidence cannot be accepted without genuinely published closed artifacts',t=>{
 const root=temporary(t);for(const value of [{},{schema:'prismpm/verify-result/1',build_id:'a'.repeat(64),attestation_id:'b'.repeat(64),verified_root:'../../outside'},{schema:'prismpm/verify-result/1',build_id:'a'.repeat(64),attestation_id:'b'.repeat(64),verified_root:'.prism/verified/'+'b'.repeat(64)}])assert.throws(()=>checkAccepted(root,value));
});

test('outer acceptance refuses absent or partial run results and incomplete or skipped test suites',()=>{
 const value={scope:'installed-native-library-only',build_id:'d'.repeat(64),checks:['read-only-check','std','no_std','exact-package-replay','two-root-reproduction','product-refusal','missing-root','wrong-result-root','parameterized-root','nominal-impostor','false-generated-acceptance','restored-acceptance'],unclaimed:['application','browser','holo','production-release','deployment']};verifyResult(value);
 for(const change of [v=>v.checks.pop(),v=>v.checks.reverse(),v=>v.extra=true,v=>v.scope='production-release',v=>v.build_id='mutable']){const bad=structuredClone(value);change(bad);assert.throws(()=>verifyResult(bad));}
 const tap='TAP version 13\n'+Array.from({length:15},(_,i)=>'ok '+(i+1)+' - gate '+i+'\n').join('')+'1..15\n# tests 15\n# suites 0\n# pass 15\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n';
 testOutput({status:0,signal:null,stdout:tap});for(const stdout of ['',tap.replace('# skipped 0','# skipped 1'),tap.replace('# tests 15','# tests 14')])assert.throws(()=>testOutput({status:0,signal:null,stdout}));
});

// Synthetic parser fixtures exercise resealed malformed evidence. These bytes
// never enter the installed CLI gate and are not generated execution evidence.
function parserFixture(root,change=()=>{}){
 const digest=bytes=>createHash('sha256').update(bytes).digest('hex'),encode=value=>JSON.stringify(value,(_,v)=>v&&typeof v==='object'&&!Array.isArray(v)?Object.fromEntries(Object.keys(v).sort().map(key=>[key,v[key]])):v);
 const lexId='b'.repeat(64),build='.prism/build/staging';
 const acceptanceRoot='LibraryProbe.Probe.acceptance',roots=[acceptanceRoot,'LibraryProbe.Probe.identity'];
 const model={schema:'prismpm/model-document/3',architecture:{component_kinds:[],components:[],concerns:[],edge_kinds:[],edges:[],model_kinds:[],stakeholders:[],viewpoints:[],views:[]},quality:{characteristics:[],measures:[],requirements:[],subcharacteristics:[]},security:{activities:[],assets:[],controls:[],impacts:[],likelihoods:[],measurements:[],risks:[],threats:[]},standards_profile:[],provenance:{compiler_semantics_id:'c'.repeat(64),emitter_semantics_id:'d'.repeat(64),facet_packages:[],semantic_id:'e'.repeat(64),snapshot_id:'f'.repeat(64),source_id:'1'.repeat(64)},library:{profile:'prismpm/native-library/1',name:'Library probe',cargo_name:'prism-library-probe',cargo_version:'0.1.0',cargo_description:'Finite native-library acceptance fixture',cargo_repository:'https://github.com/UOR-Foundation/PrismPM',cargo_homepage:'https://github.com/UOR-Foundation/PrismPM',export_roots:roots,acceptance_roots:[acceptanceRoot]}};
 const paths=['coverage.json','kernel.ir','model-binding.json','package/Cargo.lock','package/Cargo.toml','package/LICENSE-APACHE','package/LICENSE-MIT','package/README.md','package/generation-manifest.json','package/src/lib.rs','prism-library-probe-0.1.0.crate','roots.json'];
 const acceptance={build_id:'',executions:['std','no_std'].map(mode=>({mode,roots:[acceptanceRoot],status:'passed'})),export_roots:roots,lexlean_attestation_id:lexId,model_id:'',profile:'prismpm/native-library/1',regeneration:'byte-identical',schema:'prismpm/library-acceptance/1',scope:'native-library-only',status:'passed',unclaimed:['application','browser','holo','production-release','deployment']};
 const processes=['lean-version','lake-version','rustfmt-version','rustc-version','timeout-version','lake-build-generated','lean4-prod-build','prod-export','native-library-package','native-library-std-lock','native-library-std-acceptance','native-library-no_std-lock','native-library-no_std-acceptance'].map(tool=>({tool,argv:[],executable_sha256:'2'.repeat(64),exit_code:0,stdout:tool.endsWith('-acceptance')?encode({roots:[acceptanceRoot],status:'passed'}):'',stderr:''}));
 const manifest={acceptance_sha256:'',artifacts:[],build_id:'',lexlean_attestation_sha256:'',model_sha256:'',processes,schema:'prismpm/library-verification-manifest/1',scope:'native-library-only'};
 const modules=[{lean_module:'LibraryProbe.Foundation.Library.V1.Model',declarations:[{lean_name:'NativeLibrary',axiom_policy:{kind:'none',axioms:[]}}]},{lean_module:'LibraryProbe.Probe',declarations:['acceptance','identity','probeLibrary'].map(lean_name=>({lean_name,axiom_policy:{kind:'none',axioms:[]}}))}];
 const lex={attestation_id:lexId,build_id:'3'.repeat(64),source_id:model.provenance.source_id,semantic_id:model.provenance.semantic_id,spec:'lexlean/attestation/1',status:'verified',declarations:modules.flatMap(module=>module.declarations.map(row=>({name:module.lean_module+'.'+row.lean_name,observed:[],policy:row.axiom_policy,result:'ok'})))};
 const inputs={schema:'prismpm/build-inputs/3',application_generator_sha256:'4'.repeat(64),dependency_register_sha256:'5'.repeat(64),emitter_semantics_id:model.provenance.emitter_semantics_id,lexlean_build_id:lex.build_id,lexlean_semantic_id:lex.semantic_id,lexlean_source_id:lex.source_id,library_artifacts_sha256:'',library_generator_sha256:'6'.repeat(64),model_id:'',system_id:null};
 change({model,acceptance,manifest,paths,lex,inputs});
 const modelBytes=encode(model);put(root,build+'/model.prism.json',modelBytes);manifest.model_sha256=digest(modelBytes);acceptance.model_id=manifest.model_sha256;
 for(const path of paths){const bytes=path==='model-binding.json'?encode({acceptance_roots:[acceptanceRoot],export_roots:roots,model_id:manifest.model_sha256,profile:'prismpm/native-library/1',schema:'prismpm/library-build-binding/1',scope:'native-library-only'}):'parser-only '+path;put(root,build+'/library/'+path,bytes);manifest.artifacts.push({path:'library/'+path,byte_length:Buffer.byteLength(bytes),sha256:digest(bytes)});}
 put(root,build+'/lexlean/build/manifest.json','parser only');put(root,build+'/lexlean/snapshot.json',encode({modules}));
 for(const name of ['Foundation/Library/V1/Model','Probe'])for(const path of ['coverage/LibraryProbe/'+name+'.coverage.json','lexicons/'+name.replaceAll('/','.')+'.closure.json','maps/LibraryProbe/'+name+'.map.json','modules/LibraryProbe/'+name+'.lean','modules/LibraryProbe/'+name+'.tex'])put(root,build+'/lexlean/build/'+path,'parser only');
 const rows=tree(join(root,build)).filter(row=>row.kind==='file').map(row=>({byte_length:row.size,kind:'artifact',path:row.path,sha256:row.sha256}));
 inputs.model_id=manifest.model_sha256;inputs.library_artifacts_sha256=digest(encode(rows));const buildId=digest(encode(inputs));put(root,build+'/manifest.json',encode({schema:'prismpm/build-manifest/1',files:rows,inputs}));renameSync(join(root,build),join(root,'.prism/build/'+buildId));
 manifest.build_id=buildId;acceptance.build_id=buildId;const lexBytes=encode(lex);manifest.lexlean_attestation_sha256=digest(lexBytes);
 const acceptanceBytes=encode(acceptance);manifest.acceptance_sha256=digest(acceptanceBytes);const manifestBytes=encode(manifest),attestation=digest(manifestBytes),verified='.prism/verified/'+attestation;
 put(root,verified+'/manifest.json',manifestBytes);put(root,verified+'/library-acceptance.json',acceptanceBytes);put(root,verified+'/lexlean-attestation.json',lexBytes);
 return{schema:'prismpm/verify-result/1',build_id:buildId,attestation_id:attestation,verified_root:verified};
}

test('closed native evidence parser rejects coherently resealed model, acceptance and process mutations',t=>{
 const valid=temporary(t),receipt=parserFixture(valid);checkAccepted(valid,receipt);
 for(const change of [
  ({manifest})=>{manifest.extra=true;},({manifest})=>{delete manifest.scope;},
  ({manifest})=>{manifest.processes[0].extra=true;},({manifest})=>{delete manifest.processes[0].argv;},
  ({manifest})=>{manifest.processes[0].argv=[42];},({manifest})=>{manifest.processes.pop();},
  ({manifest})=>{manifest.processes[10].stdout='{}';},({manifest})=>{manifest.processes[0].executable_sha256='invalid';},
  ({acceptance})=>{acceptance.extra=true;},({acceptance})=>{acceptance.executions.pop();},
  ({acceptance})=>{acceptance.lexlean_attestation_id=createHash('sha256').update(JSON.stringify({attestation_id:'b'.repeat(64)})).digest('hex');},
  ({model})=>{model.extra=true;},({model})=>{model.application=null;},({model})=>{model.provenance.extra=true;},
  ({model})=>{model.library.extra=true;},({model})=>{model.security.assets=['unmodeled'];},
  ({paths})=>{paths.pop();},({paths})=>{paths.push('extra.json');},
  ({inputs})=>{inputs.schema='prismpm/build-inputs/1';},({inputs})=>{inputs.extra=true;},
  ({lex})=>{lex.declarations.pop();},({lex})=>{lex.declarations.push(lex.declarations[0]);},
  ({lex})=>{lex.declarations[0].result='failed';},({lex})=>{lex.declarations[0].observed=['sorryAx'];},
 ]){const root=temporary(t);assert.throws(()=>checkAccepted(root,parserFixture(root,change)));}
 for(const change of [
  (root,receipt)=>rmSync(join(root,'.prism/build',receipt.build_id,'manifest.json')),
  (root,receipt)=>put(root,'.prism/build/'+receipt.build_id+'/manifest.json','{}'),
  (root,receipt)=>put(root,'.prism/build/'+receipt.build_id+'/unexpected','extra output'),
 ]){const root=temporary(t),receipt=parserFixture(root);change(root,receipt);assert.throws(()=>checkAccepted(root,receipt));}
 put(valid,'.prism/build/'+receipt.build_id+'/library/package/src/lib.rs','tampered actual bytes');assert.throws(()=>checkAccepted(valid,receipt));
});

test('release job keeps both native library and browser gates mandatory for each SDK architecture',()=>{
 const workflow=readFileSync(new URL('../.github/workflows/release.yml',import.meta.url),'utf8');
 assert.match(workflow,/bash root-a\/scripts\/browser-api-sdk-check\.sh/);
 assert.match(workflow,/name: Verify exact current SDK native-library closure offline and read-only\n\s+if: matrix.image == 'sdk'/);
 assert.match(workflow,/bash root-a\/scripts\/library-sdk-check\.sh "\$\(cat \.shipped-image\/sdk-image.txt\)" "\$GITHUB_SHA"/);
 assert.match(workflow,/os: ubuntu-24.04-arm/);assert.match(workflow,/name: library-sdk-\$\{\{ matrix.name \}\}/);
 const wrapper=readFileSync(new URL('./library-sdk-check.sh',import.meta.url),'utf8');
 for(const required of ['--user 1000:1000 --read-only --network none','--cap-drop ALL','--security-opt no-new-privileges','--tmpfs /tmp:rw,exec,nosuid,nodev,size=8g','PRISMPM_EPHEMERAL_HOME=1','node scripts/library-sdk-check.mjs run'])assert.ok(wrapper.includes(required),required);
 assert.ok(!wrapper.includes('--volume')&&!wrapper.includes('--mount'));
});

test('owning CLI test kills a removed process-exit guard',t=>{
 const root=temporary(t),module=readFileSync(new URL('./library-sdk-check.mjs',import.meta.url),'utf8');
 const guard='assert.equal(output.status,expected.code ? exits[expected.code] : 0, "CLI exit class");';assert.equal(module.split(guard).length,2);
 put(root,'library-sdk-check.mjs',module.replace(guard,''));put(root,'browser-api-sdk-check.mjs',readFileSync(new URL('./browser-api-sdk-check.mjs',import.meta.url)));put(root,'library-sdk-check.test.mjs',readFileSync(new URL('./library-sdk-check.test.mjs',import.meta.url)));
 const env={...process.env};delete env.NODE_TEST_CONTEXT;
 const output=spawnSync(process.execPath,['--test','--test-reporter=tap','--test-name-pattern=CLI transport invokes',join(root,'library-sdk-check.test.mjs')],{encoding:'utf8',env,timeout:15000,maxBuffer:1024*1024});assert.equal(output.error,undefined);assert.equal(output.status,1);assert.match(output.stdout,/Missing expected exception/);
});
