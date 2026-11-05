// DK-36 installed binary-package qualification. Never full SDK/product acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {cpSync,existsSync,lstatSync,mkdirSync,mkdtempSync,readFileSync,readdirSync,rmSync,chmodSync,writeFileSync} from 'node:fs';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {readSemantic,runtimeAudit} from './semantic-source.mjs';
import {tree,cli,mutateModule,regularBytes,boundRuntimeAudit,validateVerificationOwner,verifyExporterProcess,sourceRoots as libraryRoots,sourceAliases,verifyImage as verifySharedImage} from './library-sdk-check.mjs';
import {verifyTap} from './browser-api-sdk-check.mjs';
import {validateInventory} from '../sdk/platform-lock.mjs';
export {tree,cli,mutateModule,regularBytes,sourceAliases};
export function verifyImage(value,image,architecture,revision){
 return verifySharedImage(value,image,architecture,revision,'0.4.0');
}

export const sourceRoots=Object.freeze([...libraryRoots,
 'scripts/semantic-source-bootstrap.sh','scripts/semantic-source-bootstrap.test.mjs','scripts/source-oracles-bootstrap.sh','tools.lock',
 'crates/prismpm/tests/binary_program.rs','crates/conformance/src/cases/binary_program.rs',
 'tests/fixtures/binary/binary-program/project','tests/fixtures/binary/binary-program-language-1.2/project','scripts/binary-sdk-check.mjs','scripts/binary-language12-reader-replay.mjs',
 'scripts/binary-sdk-check.sh','scripts/sdk-owned-session.sh','scripts/sdk-owned-session.test.mjs','scripts/binary-sdk-source-extract.py','scripts/binary-sdk-check.test.mjs',
 'scripts/binary-sdk-check-fixtures.mjs','scripts/binary-sdk-language12.test.mjs','scripts/binary-sdk-check-shell.test.mjs','scripts/binary-sdk-check.md','scripts/binary-sdk-qualify.sh','scripts/binary-sdk-format.sh','scripts/binary-sdk-format-shell.test.mjs','scripts/sdk-candidate.sh','scripts/sdk-candidate.mjs','scripts/sdk-candidate-sbom.mjs','Justfile',
 '.devcontainer','xtask','crates/conformance/src/golden.rs','crates/conformance/src/golden','scripts/reconcile-lexlean.mjs','scripts/package-release-crates.sh','.github/workflows/binary-generated-review.yml',
 '.github/workflows/binary-sdk-qualification.yml','.github/workflows/ci-parallel.yml','.github/workflows/reproducibility.yml',
].sort());
export const modes=Object.freeze(['std','no_std','core-wasm','cli-stdio','cli-file','cli-mixed']);
export const unclaimed=Object.freeze(['application','browser','holo','production-release','deployment']);
export const completedChecks=Object.freeze(['read-only-check',...modes,'adapter-allocation-capacity-overflow','exact-package-replay',
 'two-root-reproduction','product-refusal','missing-root','wrong-result-root',
 'parameterized-root','nominal-impostor','false-generated-acceptance',
 'missing-proof-refused','changed-proof-refused','missing-acceptance-refused',
 'changed-acceptance-refused','changed-package-refused','restored-acceptance']);
export const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
export const canonical=value=>JSON.stringify(value,(_,v)=>v&&typeof v==='object'&&!Array.isArray(v)?Object.fromEntries(Object.keys(v).sort().map(key=>[key,v[key]])):v);
const same=(actual,expected,message)=>assert.equal(canonical(actual),canonical(expected),message);
const keys=(value,names)=>{assert.ok(value&&typeof value==='object'&&!Array.isArray(value),'closed object');same(Object.keys(value).sort(),names.slice().sort(),'closed object fields');};
const hex=value=>{assert.equal(typeof value,'string');assert.match(value,/^[0-9a-f]{64}$/);return value;};
const revision=value=>{assert.equal(typeof value,'string');assert.match(value,/^[0-9a-f]{40}$/);return value;};
const imagePattern=/^[a-z0-9][a-z0-9./:_-]*@sha256:[0-9a-f]{64}$/;
const document=path=>{const bytes=regularBytes(path),value=JSON.parse(bytes);assert.ok(bytes.equals(Buffer.from(canonical(value))),'canonical artifact bytes: '+path);return value;};
export function capture(root,sourceRevision){revision(sourceRevision);assert.equal(lstatSync(join(root,'crates/conformance/build.rs'),{throwIfNoEntry:false}),undefined,'unregistered conformance build script');return{revision:sourceRevision,files:tree(root,sourceRoots,sourceAliases)};}
export function verifySource(root,expected){same(capture(root,expected.revision),expected,'installed SDK source closure differs');}
export function inventoryEvidence(bytes,sourceRevision){
 revision(sourceRevision);const inventory=validateInventory(bytes);
 const sources=inventory.artifacts.filter(row=>row.id==='sdk-vv-source');assert.equal(sources.length,1);assert.equal(sources[0].version,sourceRevision,'inventory current source revision');
 const binaries=inventory.artifacts.filter(row=>row.id==='prismpm');assert.equal(binaries.length,1);
 const commands=inventory.commands.filter(row=>row.command==='prismpm');assert.equal(commands.length,1);assert.equal(commands[0].executable,'/usr/local/bin/prismpm');
 assert.equal(binaries[0].digest,'sha256:'+commands[0].sha256,'inventory CLI identity');return{sha256:hash(bytes),cli_sha256:commands[0].sha256};
}
export const fixtureProgram=Object.freeze({
 profile:'prismpm/binary-program/1',name:'Binary probe',cargo_name:'prism-binary-probe',cargo_version:'0.1.0',
 cargo_description:'Finite arbitrary-byte package acceptance fixture',cargo_repository:'https://github.com/UOR-Foundation/PrismPM',cargo_homepage:'https://github.com/UOR-Foundation/PrismPM',
 export_roots:['LibraryProbe.Probe.identity'],entry_root:'LibraryProbe.Probe.identity',request_maximum:256,response_maximum:256,memory_pages:32,
 acceptance_vectors:[[],[0],Array.from({length:256},(_,n)=>n),[192,175,255,128,237,160,128]].map(value=>({request:value,response:value})),cli:{profile:'prismpm/raw-file-cli/1'},
});
// Exact ordered execution tools are part of the owning verification contract.
export const processTools=Object.freeze(['lean-version','lake-version','rustfmt-version','rustc-version','timeout-version','lake-build-generated','lean4-prod-build','prod-export',
 'binary-program-package','binary-cli-lock','binary-cli-build','binary-cli-package','binary-core-wasm-build','binary-std-lock','binary-std-acceptance','binary-no_std-lock','binary-no_std-acceptance','binary-cli-allocation-acceptance','binary-cli-acceptance-build','binary-transports-acceptance']);
export const binaryPaths=Object.freeze([
 'binary/acceptance-runner.mjs','binary/memory-inspector.mjs','binary/core-wasm/.cargo/config.toml','binary/core-wasm/Cargo.lock','binary/core-wasm/generation-manifest.json','binary/cli/Cargo.lock','binary/cli/Cargo.toml','binary/cli/src/main.rs','binary/core-wasm/Cargo.toml','binary/core-wasm/src/lib.rs','binary/core.wasm',
 'binary/coverage.json','binary/kernel.ir','binary/model-binding.json','binary/package/Cargo.lock','binary/package/Cargo.toml',
 'binary/package/LICENSE-APACHE','binary/package/LICENSE-MIT','binary/package/README.md','binary/package/generation-manifest.json',
 'binary/package/src/lib.rs','binary/prism-binary-probe-0.1.0.crate','binary/prism-binary-probe-cli-0.1.0.crate','binary/roots.json',
].sort());
export function checkAccepted(project,receipt,sourceAuthority,language='1.1'){
 assert(['1.1','1.2'].includes(language),'closed qualification language');
 keys(receipt,['schema','build_id','attestation_id','verified_root']);assert.equal(receipt.schema,'prismpm/verify-result/1');hex(receipt.build_id);hex(receipt.attestation_id);
 assert.equal(receipt.verified_root,'.prism/verified/'+receipt.attestation_id);
 const verified=join(project,receipt.verified_root),build=join(project,'.prism/build',receipt.build_id);
 for(const relative of [receipt.verified_root,'.prism/build/'+receipt.build_id]){let selected=resolve(project);for(const part of ['',...relative.split('/')]){if(part)selected=join(selected,part);const stat=lstatSync(selected);assert.ok(stat.isDirectory()&&!stat.isSymbolicLink(),'evidence directory alias');}}
 same(tree(verified).map(row=>row.path),['/','binary-acceptance.json','lexlean-attestation.json','manifest.json'],'unexpected verified outputs');
 const manifestBytes=regularBytes(join(verified,'manifest.json')),manifest=document(join(verified,'manifest.json'));
 const acceptanceBytes=regularBytes(join(verified,'binary-acceptance.json')),acceptance=document(join(verified,'binary-acceptance.json'));
 keys(manifest,['acceptance_sha256','artifacts','build_id','exporter_owner','lexlean_attestation_sha256','model_sha256','processes','schema','scope']);
 assert.equal(hash(manifestBytes),receipt.attestation_id);assert.equal(manifest.schema,'prismpm/binary-verification-manifest/2');assert.equal(manifest.scope,'binary-package-only');assert.equal(manifest.build_id,receipt.build_id);
 const lexBytes=regularBytes(join(verified,'lexlean-attestation.json')),lexAttestation=JSON.parse(lexBytes);hex(lexAttestation.attestation_id);
 assert.equal(manifest.acceptance_sha256,hash(acceptanceBytes));assert.equal(manifest.lexlean_attestation_sha256,hash(lexBytes));
 const modelBytes=regularBytes(join(build,'model.prism.json')),model=document(join(build,'model.prism.json'));
 keys(model,['architecture','program','provenance','quality','schema','security','standards_profile']);assert.equal(model.schema,'prismpm/model-document/7');
 same(model.architecture,{component_kinds:[],components:[],concerns:[],edge_kinds:[],edges:[],model_kinds:[],stakeholders:[],viewpoints:[],views:[]},'no application architecture claim');
 same(model.quality,{characteristics:[],measures:[],requirements:[],subcharacteristics:[]},'no application quality claim');
 same(model.security,{activities:[],assets:[],controls:[],impacts:[],likelihoods:[],measurements:[],risks:[],threats:[]},'no application security claim');same(model.standards_profile,[]);
 keys(model.provenance,['compiler_semantics_id','emitter_semantics_id','facet_packages','semantic_id','snapshot_id','source_id']);same(model.provenance.facet_packages,[]);
 for(const [key,value] of Object.entries(model.provenance))if(key!=='facet_packages')hex(value);
 same(model.program,fixtureProgram,'exact whole fixture program descriptor');assert.equal(manifest.model_sha256,hash(modelBytes));
 same(acceptance,{build_id:receipt.build_id,io_coverage:{platform:'linux',output_write:'passed'},executions:modes.map(mode=>({mode,vector_count:fixtureProgram.acceptance_vectors.length,status:'passed'})),program:fixtureProgram,lexlean_attestation_id:lexAttestation.attestation_id,model_id:hash(modelBytes),profile:'prismpm/binary-program/1',regeneration:'byte-identical',schema:'prismpm/binary-acceptance/1',scope:'binary-package-only',status:'passed',unclaimed},'binary acceptance scope or execution differs');
 assert.ok(Array.isArray(manifest.processes));same(manifest.processes.map(row=>row.tool),processTools,'complete ordered binary verification processes');
 for(const row of manifest.processes){
  keys(row,['tool','argv','executable_sha256','exit_code','stdout','stderr',...(row.tool==='prod-export'?['exporter']:[])]);
  assert.equal(row.exit_code,0);hex(row.executable_sha256);assert.ok(Array.isArray(row.argv)&&row.argv.every(arg=>typeof arg==='string'));assert.equal(typeof row.stdout,'string');assert.equal(typeof row.stderr,'string');
  if(row.tool==='prod-export')verifyExporterProcess(row,'prod-export','cold',undefined,sourceAuthority);
 }
 for(const mode of ['std','no_std']){const record=manifest.processes.find(row=>row.tool==='binary-'+mode+'-acceptance');assert.equal(record.stdout,JSON.stringify(fixtureProgram.acceptance_vectors.map(row=>row.response))+'\n','actual native byte transcript');assert.equal(record.stderr,'');}
 const allocation=manifest.processes.find(row=>row.tool==='binary-cli-allocation-acceptance');
 same(allocation.argv,['test','--locked','--offline','--release','--','--exact','tests::adapter_allocation_maps_real_capacity_overflow'],'mandatory actual allocation test command');
 assert.equal((allocation.stdout.match(/^test tests::adapter_allocation_maps_real_capacity_overflow \.\.\. ok$/gm)||[]).length,1,'actual capacity-overflow test ran once');
 assert.equal((allocation.stdout.match(/^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in .+$/gm)||[]).length,1,'exactly one passing non-skipped allocation test');
 const transports=manifest.processes.find(row=>row.tool==='binary-transports-acceptance');same(JSON.parse(transports.stdout),{executions:modes.slice(2).map(mode=>({mode,vector_count:fixtureProgram.acceptance_vectors.length,status:'passed'})),io_coverage:{platform:'linux',output_write:'passed'}},'actual transport transcript');assert.equal(transports.stderr,'');
 const artifacts=tree(build).filter(row=>row.kind==='file');assert.ok(artifacts.every(row=>!row.path.endsWith('.holo')&&!row.path.startsWith('application/')));
 const buildManifest=document(join(build,'manifest.json'));keys(buildManifest,['files','inputs','schema']);assert.equal(buildManifest.schema,'prismpm/build-manifest/1');
 const inputs=buildManifest.inputs;keys(inputs,['application_generator_sha256','binary_artifacts_sha256','binary_generator_sha256','dependency_register_sha256','emitter_semantics_id','lexlean_build_id','lexlean_semantic_id','lexlean_source_id','model_id','schema','system_id']);assert.equal(inputs.schema,'prismpm/build-inputs/4');assert.equal(inputs.system_id,null);
 for(const [name,value] of Object.entries(inputs))if(!['schema','system_id'].includes(name))hex(value);
 assert.equal(hash(canonical(inputs)),receipt.build_id,'artifact-bound build identity');assert.equal(inputs.model_id,hash(modelBytes));assert.equal(inputs.emitter_semantics_id,model.provenance.emitter_semantics_id);assert.equal(inputs.lexlean_semantic_id,model.provenance.semantic_id);assert.equal(inputs.lexlean_source_id,model.provenance.source_id);
 assert.ok(Array.isArray(buildManifest.files));for(const row of buildManifest.files){keys(row,['byte_length','kind','path','sha256']);assert.ok(['lean','latex','source-map','coverage','lexicon-closure','lexlean-manifest','semantic-snapshot','artifact'].includes(row.kind));}
 same(buildManifest.files.map(({kind,...row})=>row),artifacts.filter(row=>row.path!=='manifest.json').map(row=>({path:row.path,byte_length:row.size,sha256:row.sha256})),'complete build manifest file closure');
 assert.equal(inputs.binary_artifacts_sha256,hash(canonical(buildManifest.files)),'complete artifact row digest');
 const lexPaths=['lexlean/build/manifest.json','lexlean/snapshot.json'];
 for(const name of ['Foundation/Binary/V1/Model','Probe']){lexPaths.push('lexlean/build/coverage/LibraryProbe/'+name+'.coverage.json','lexlean/build/lexicons/'+name.replaceAll('/','.')+'.closure.json','lexlean/build/maps/LibraryProbe/'+name+'.map.json','lexlean/build/modules/LibraryProbe/'+name+'.lean','lexlean/build/modules/LibraryProbe/'+name+'.tex');}
 same(artifacts.filter(row=>!row.path.startsWith('binary/')).map(row=>row.path),[...lexPaths,'manifest.json','model.prism.json'].sort(),'complete fixture non-binary artifacts');
 const snapshot=JSON.parse(regularBytes(join(build,'lexlean/snapshot.json')));assert.ok(Array.isArray(snapshot.modules));
 const declarations=snapshot.modules.flatMap(module=>module.declarations.map(declaration=>({name:module.lean_module+'.'+declaration.lean_name,policy:declaration.axiom_policy})));
 const names=[...['AdapterFailure','AdapterFailureResult','BinaryAcceptanceVector','BinaryProgram','PublicationOutcome','RawFileCli','adapterFailureResult','failurePublication'].map(name=>'LibraryProbe.Foundation.Binary.V1.Model.'+name),'LibraryProbe.Probe.identity','LibraryProbe.Probe.probeProgram'].sort();
 same(declarations.map(row=>row.name).sort(),names,'complete selected declaration closure');
 assert.equal(lexAttestation.spec,language==='1.1'?'lexlean/attestation/1':'lexlean/attestation/2','exact qualification attestation version');assert.equal(lexAttestation.status,'verified');assert.equal(lexAttestation.build_id,inputs.lexlean_build_id);assert.equal(lexAttestation.lexlean?.compiler_semantics,model.provenance.compiler_semantics_id);assert.equal(lexAttestation.build_manifest?.sha256,hash(regularBytes(join(build,'lexlean/build/manifest.json'))));assert.equal(lexAttestation.source_id,inputs.lexlean_source_id);assert.equal(lexAttestation.semantic_id,inputs.lexlean_semantic_id);
 const runtime=boundRuntimeAudit(build,artifacts);if(language==='1.1')same(runtime,[],'original frozen fixture has no imported runtime');else assert(runtime.length>0,'language-1.2 fixture must exercise imported runtime');
 assert.ok(Array.isArray(lexAttestation.declarations));same(lexAttestation.declarations.map(row=>row.name).sort(),[...names,...runtime.map(row=>row.name)].sort(),'complete published declaration audit');
 for(const row of lexAttestation.declarations){keys(row,['name','observed','policy','result']);assert.equal(row.result,'ok');const declaration=declarations.find(declaration=>declaration.name===row.name);if(declaration){same(row.policy,declaration.policy);same(row.policy,{axioms:[],kind:'none'});same(row.observed,[],'fixture has no observed axioms');}else same(row,runtime.find(expected=>expected.name===row.name),'exact source-owned runtime axiom audit');}
 const binary=artifacts.filter(row=>row.path.startsWith('binary/'));same(binary.map(row=>row.path),binaryPaths,'complete fixture binary artifact set');
 same(manifest.artifacts,binary.map(row=>({path:row.path,byte_length:row.size,sha256:row.sha256})),'complete exact binary replay artifacts');
 const owner=validateVerificationOwner(manifest,'binary','cold',undefined,sourceAuthority);
 const leanManifest=regularBytes(join(build,'lexlean/build/manifest.json')),outputs=JSON.parse(leanManifest).outputs;
 assert(Array.isArray(outputs));const modules=outputs.filter(row=>row.kind==='lean').map(row=>{
  assert.match(row.path,/^modules\/[A-Za-z0-9_/]+\.lean$/);return row.path.slice(8,-5).replaceAll('/','.');
 }).sort();assert(modules.length>0&&new Set(modules).size===modules.length);
 const argv=['exe','prod-export',...modules.flatMap(module=>['--module',module]),...fixtureProgram.export_roots.flatMap(root=>['--root',root]),
  '--ir-module',fixtureProgram.cargo_name.replaceAll('-','_'),'--out','$BINARY_WORK/export'];
 for(const phase of owner.phases){assert.equal(phase.lexlean_manifest_sha256,hash(leanManifest));same(phase.processes[2].argv,argv);}
 same(document(join(build,'binary/model-binding.json')),{model_id:hash(modelBytes),program:fixtureProgram,profile:'prismpm/binary-program/1',schema:'prismpm/binary-build-binding/1',scope:'binary-package-only'},'whole binary model binding');
 return{build,model_id:hash(modelBytes)};
}
function noVerification(project){const path=join(project,'.prism/verified');assert.ok(!existsSync(path)||readdirSync(path).length===0,'unaccepted source published verification');}
const declaration=(module,name)=>{const rows=module.declarations.filter(row=>row.name===name);assert.equal(rows.length,1);return rows[0];};
const rootField=(module,name)=>{const rows=declaration(module,'probeProgram').body.fields.filter(row=>row.field===name);assert.equal(rows.length,1);return rows[0].value;};
export function rejectChangedEvidence(project,receipt,path,sourceAuthority,language='1.1'){
 const original=regularBytes(path);
 try{rmSync(path);assert.throws(()=>checkAccepted(project,receipt,sourceAuthority,language));writeFileSync(path,Buffer.concat([original,Buffer.from([0])]));assert.throws(()=>checkAccepted(project,receipt,sourceAuthority,language));}
 finally{writeFileSync(path,original);}
 checkAccepted(project,receipt,sourceAuthority,language);
}
export const runtimeAuditChecks=Object.freeze(['runtime-row-omission','runtime-row-duplication','runtime-unknown-name',
 'runtime-invented-policy','runtime-wrong-policy-kind','runtime-erased-nonempty-policy','runtime-observed-axiom','runtime-module-omission','runtime-known-unimported-module','project-observed-axiom','wrong-attestation-version']);
export const qualificationChecks=language=>language==='1.1'?completedChecks:[...completedChecks,'language-1.2-source',...runtimeAuditChecks];
// Exact LexLean framed identity, anchored to the genuine original before any
// owned negative copy is resealed. This never creates positive compiler evidence.
function auditIdentity(value){
 const {attestation_id,...body}=value,data=Buffer.from(canonical(body)),label=Buffer.from('attestation-body');
 const labelLength=Buffer.alloc(4),bodyLength=Buffer.alloc(8);labelLength.writeUInt32BE(label.length);bodyLength.writeBigUInt64BE(BigInt(data.length));
 return createHash('sha256').update('lexlean-attestation-v1\0').update(labelLength).update(label).update(bodyLength).update(data).digest('hex');
}
export function rejectRuntimeAuditMutations(project,receipt,sourceAuthority){
 checkAccepted(project,receipt,sourceAuthority,'1.2');
 const original=join(project,receipt.verified_root),manifestBytes=regularBytes(join(original,'manifest.json')),
  auditBytes=regularBytes(join(original,'lexlean-attestation.json')),acceptanceBytes=regularBytes(join(original,'binary-acceptance.json'));
 const manifest=JSON.parse(manifestBytes),audit=JSON.parse(auditBytes),acceptance=JSON.parse(acceptanceBytes);
 assert.equal(auditIdentity(audit),audit.attestation_id,'source-defined framed attestation identity');
 const snapshot=JSON.parse(regularBytes(join(project,'.prism/build',receipt.build_id,'lexlean/snapshot.json'))),
  projectNames=new Set(snapshot.modules.flatMap(module=>module.declarations.map(row=>module.lean_module+'.'+row.lean_name))),
  names=new Set(audit.declarations.map(row=>row.name)),runtime=value=>value.declarations.find(row=>!projectNames.has(row.name)),
  extra=runtimeAudit(['import LexLeanRuntime.Laws\n']).filter(row=>!names.has(row.name));
 assert(runtime(audit)&&extra.length,'the genuine fixture imports a proper runtime subset');
 const mutations=[
  value=>{value.declarations.splice(value.declarations.indexOf(runtime(value)),1);},
  value=>{value.declarations.push(structuredClone(runtime(value)));},
  value=>{runtime(value).name='LexLeanRuntime.undeclaredBinaryAuditProbe';},
  value=>{const row=runtime(value);row.policy.axioms=['sorryAx'];row.observed=['sorryAx'];},
  value=>{const row=runtime(value);row.policy={kind:'none',axioms:[]};row.observed=[];},
  value=>{const row=value.declarations.find(row=>!projectNames.has(row.name)&&row.observed.length>0);assert(row);row.policy.axioms=[];row.observed=[];},
  value=>{runtime(value).observed=['sorryAx'];},
  value=>{value.declarations=value.declarations.filter(row=>projectNames.has(row.name));},
  value=>{value.declarations.push(...structuredClone(extra));},
  value=>{value.declarations.find(row=>projectNames.has(row.name)).observed=['sorryAx'];},
  value=>{value.spec='lexlean/attestation/1';},
 ];
 assert.equal(mutations.length,runtimeAuditChecks.length);
 const expectedMessages=['complete published declaration audit','complete published declaration audit','complete published declaration audit',
  'exact source-owned runtime axiom audit','exact source-owned runtime axiom audit','exact source-owned runtime axiom audit','exact source-owned runtime axiom audit',
  'complete published declaration audit','complete published declaration audit','fixture has no observed axioms','exact qualification attestation version'];
 assert.equal(expectedMessages.length,mutations.length);
 for(const [index,mutate]of mutations.entries()){
  const changedAudit=structuredClone(audit);mutate(changedAudit);changedAudit.attestation_id=auditIdentity(changedAudit);
  const changedAcceptance=structuredClone(acceptance);changedAcceptance.lexlean_attestation_id=changedAudit.attestation_id;
  const changed=structuredClone(manifest),auditBody=Buffer.from(canonical(changedAudit)),acceptanceBody=Buffer.from(canonical(changedAcceptance));
  changed.lexlean_attestation_sha256=hash(auditBody);changed.acceptance_sha256=hash(acceptanceBody);
  const body=Buffer.from(canonical(changed)),id=hash(body),directory=join(project,'.prism/verified',id);
  assert(!existsSync(directory),'fresh owned negative output');mkdirSync(directory);
  try{
   writeFileSync(join(directory,'manifest.json'),body,{flag:'wx'});writeFileSync(join(directory,'lexlean-attestation.json'),auditBody,{flag:'wx'});
   writeFileSync(join(directory,'binary-acceptance.json'),acceptanceBody,{flag:'wx'});
   assert.throws(()=>checkAccepted(project,{...receipt,attestation_id:id,verified_root:'.prism/verified/'+id},sourceAuthority,'1.2'),error=>error instanceof assert.AssertionError&&error.message.includes(expectedMessages[index]),runtimeAuditChecks[index]);
  }finally{rmSync(directory,{recursive:true});}
 }
 assert(regularBytes(join(original,'manifest.json')).equals(manifestBytes));assert(regularBytes(join(original,'lexlean-attestation.json')).equals(auditBytes));
 assert(regularBytes(join(original,'binary-acceptance.json')).equals(acceptanceBytes));checkAccepted(project,receipt,sourceAuthority,'1.2');
 return runtimeAuditChecks.length;
}
export function run(root,image,sourceRevision,sourceSha256,inventorySha256,language='1.1'){
 assert(['1.1','1.2'].includes(language),'closed qualification language');
 assert.equal(process.getuid(),1000,'native SDK gate must run non-root');assert.match(image,imagePattern);revision(sourceRevision);hex(sourceSha256);hex(inventorySha256);
 assert.equal(hash(canonical(capture(root,sourceRevision))),sourceSha256,'installed current source binding');
 const inventory=inventoryEvidence(regularBytes('/opt/prismpm/share/inventory.json'),sourceRevision);assert.equal(inventory.sha256,inventorySha256);
 const sourceAuthority={archive_sha256:hash(regularBytes(join(root,'vendor/lean4-prod/lean.tar'))),toolchain:regularBytes(join(root,'lean-toolchain')).toString().trim()};
 assert.equal(hash(regularBytes('/usr/local/bin/prismpm',256*1024*1024)),inventory.cli_sha256,'actual installed CLI matches immutable inventory');
 const work=mkdtempSync('/tmp/prismpm-binary-sdk-'),source=join(root,language==='1.1'?'tests/fixtures/binary/binary-program/project':'tests/fixtures/binary/binary-program-language-1.2/project');let sequence=0;
 function fixture(){const destination=join(work,'fixture-'+sequence++);cpSync(source,destination,{recursive:true,errorOnExist:true,force:false});for(const row of tree(destination))chmodSync(join(destination,row.path),row.kind==='directory'?0o700:0o600);assert.ok(!existsSync(join(destination,'.prism')));return destination;}
 const positive=(project,args)=>cli(project,args,{schema:'prismpm/'+args[0]+'-result/1'});
 try{
  const first=fixture(),before=tree(first),checked=positive(first,['check']);same(tree(first),before,'check modified source');assert.equal(checked.entity_count,1);
  const accepted=positive(first,['verify']),evidence=checkAccepted(first,accepted,sourceAuthority,language);assert.equal(checked.model_id,evidence.model_id);
  const firstBuild=tree(evidence.build),second=fixture(),secondBuild=positive(second,['build']);assert.equal(secondBuild.build_id,accepted.build_id);
  same(tree(join(second,'.prism/build',secondBuild.build_id)),firstBuild,'complete build differs across fresh absolute roots');noVerification(second);
  const secondVerify=positive(second,['verify']);assert.equal(secondVerify.build_id,accepted.build_id);checkAccepted(second,secondVerify,sourceAuthority,language);
  const beforeProduct=tree(first);cli(first,['build','--locked','-t','ghcr.io/uor-foundation/prismpm-binary-probe:0.1.0'],{code:'PP6101'});same(tree(first),beforeProduct,'product refusal wrote outputs');
  const nominal=readSemantic(regularBytes(join(source,'src/Foundation/Binary/V1/Model.lex.tex')));
  const mutations=[
   module=>{rootField(module,'entryRoot').value='LibraryProbe.Probe.missing';rootField(module,'exportRoots').head.value='LibraryProbe.Probe.missing';},
   module=>{declaration(module,'identity').result={kind:'nat'};declaration(module,'identity').body={kind:'nat',value:'0'};},
   module=>{declaration(module,'identity').parameters.push({name:'unused',type:{kind:'nat'}});},
   module=>{module.declarations.unshift(...nominal.declarations);delete declaration(module,'probeProgram').result.member.module;delete declaration(module,'probeProgram').body.type.module;},
  ];
  for(const change of mutations){const invalid=fixture();mutateModule(invalid,change);const before=tree(invalid);cli(invalid,['check'],{code:'PP2001'});same(tree(invalid),before,'invalid check wrote outputs');noVerification(invalid);}
  const mutant=fixture(),original=mutateModule(mutant,module=>{rootField(module,'acceptanceVectors').head.fields.find(row=>row.field==='response').value={kind:'bytes',hex:'01'};});
  const changed=positive(mutant,['check']);assert.notEqual(changed.semantic_id,checked.semantic_id);cli(mutant,['verify'],{code:'PP5006'});noVerification(mutant);
  writeFileSync(join(mutant,'src/Probe.lex.tex'),original);const restored=positive(mutant,['verify']);assert.equal(restored.build_id,accepted.build_id);checkAccepted(mutant,restored,sourceAuthority,language);
  for(const name of ['lexlean-attestation.json','binary-acceptance.json'])rejectChangedEvidence(first,accepted,join(first,accepted.verified_root,name),sourceAuthority,language);
  rejectChangedEvidence(first,accepted,join(evidence.build,'binary/package/src/lib.rs'),sourceAuthority,language);
  // A prior passing receipt is never an input to current verification.
  const freshlyVerified=positive(first,['verify']);checkAccepted(first,freshlyVerified,sourceAuthority,language);assert.equal(freshlyVerified.build_id,accepted.build_id);
  if(language==='1.2')rejectRuntimeAuditMutations(first,freshlyVerified,sourceAuthority);
  const retained='/tmp/prismpm-binary-evidence';assert.ok(!existsSync(retained),'fresh evidence output');mkdirSync(join(retained,'.prism/build'),{recursive:true});mkdirSync(join(retained,'.prism/verified'),{recursive:true});
  cpSync(evidence.build,join(retained,'.prism/build',accepted.build_id),{recursive:true,errorOnExist:true,force:false});
  cpSync(join(first,freshlyVerified.verified_root),join(retained,freshlyVerified.verified_root),{recursive:true,errorOnExist:true,force:false});writeFileSync(join(retained,'receipt.json'),canonical(freshlyVerified));
  return{schema:language==='1.1'?'prismpm/installed-binary-check/1':'prismpm/installed-binary-language-1.2-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:image,source_revision:sourceRevision,source_sha256:sourceSha256,inventory_sha256:inventory.sha256,architecture:({x64:'amd64',arm64:'arm64'}[process.arch]),build_id:accepted.build_id,attestation_id:freshlyVerified.attestation_id,checks:qualificationChecks(language),unclaimed};
 }finally{rmSync(work,{recursive:true,force:true});}
}
export function verifyResult(value,image,sourceRevision,architecture,sourceSha256,inventorySha256,language='1.1'){
 assert(['1.1','1.2'].includes(language),'closed qualification language');
 assert.match(image,imagePattern);revision(sourceRevision);assert.ok(['amd64','arm64'].includes(architecture));hex(sourceSha256);hex(inventorySha256);hex(value.build_id);hex(value.attestation_id);
 same(value,{schema:language==='1.1'?'prismpm/installed-binary-check/1':'prismpm/installed-binary-language-1.2-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:image,source_revision:sourceRevision,source_sha256:sourceSha256,inventory_sha256:inventorySha256,architecture,build_id:value.build_id,attestation_id:value.attestation_id,checks:qualificationChecks(language),unclaimed},'complete installed binary result');
}
export function validateHelperBootstrapWorkflow(workflow,generated=false){
 assert([false,true,'browser-library','oci-native'].includes(generated),'closed host prerequisite context');
 const release=typeof generated==='string',job=release?generated:generated?'prepare':'native-binary';
 const selected=new RegExp('^  '+job+':\\n[\\s\\S]*?(?=^  [a-z][a-z0-9-]*:\\n|(?![\\s\\S]))','gm');
 const scopes=[...workflow.matchAll(selected)];assert.equal(scopes.length,1,'one owning host prerequisite job');
 const scope=scopes[0][0],starts=[...scope.matchAll(/^      - /gm)].map(match=>match.index);
 const steps=starts.map((start,index)=>scope.slice(start,starts[index+1]??scope.length));
 const named=name=>{const matches=steps.filter(step=>step.startsWith('      - name: '+name+'\n'));assert.equal(matches.length,1,'one '+name);return matches[0];};
 const evidence=release?'$RUNNER_TEMP/release-semantic-evidence':generated?'$RUNNER_TEMP/binary-helper-evidence':'$RUNNER_TEMP/binary-qualification/setup';
 const target='$RUNNER_TEMP/'+(release?'release-semantic-target':'binary-helper-target');
 const bootstrap=named(release?generated==='browser-library'?'Prepare the same-source semantic helper before native SDK owners':'Prepare the same-source semantic helper before retained proof readers':'Build the same-source semantic helper before offline boundary tests');
 let previous=-1;
 for(const required of [
  '          CARGO_HOME: ${{ runner.temp }}/'+(release?'release-semantic-cargo':'binary-helper-cargo')+'\n',
  '          CARGO_NET_OFFLINE: "true"\n','        run: |\n          set -euo pipefail\n',
  '          bash scripts/semantic-source-bootstrap.sh "'+(release?'$GITHUB_SHA':'$SOURCE_REVISION')+'" \\\n'+
   '            "'+target+'" "'+evidence+'"\n',
  '          helper=$(cat "'+evidence+'/semantic-source-path.txt")\n',
  '          test "$helper" = "'+target+'/debug/prismpm-semantic-source"\n',
  '          node_path=$(cat "'+evidence+'/node-oracles-path.txt")\n',
  '          test "$node_path" = "'+target+'-node-oracles/node_modules"\n',
  '          printf \'PRISMPM_SEMANTIC_SOURCE=%s\\nNODE_PATH=%s\\n\' "$helper" "$node_path" >> "$GITHUB_ENV"\n',
 ]){const next=bootstrap.indexOf(required);assert(next>previous,'ordered source-bound Rust and Node readiness: '+required);previous=next;}
 assert.doesNotMatch(bootstrap,/^        (?:if|continue-on-error):|\|\| true|\bif false\b|\bset \+e\b/m,'fail-closed compound prerequisite');
 if(generated==='browser-library')assert(bootstrap.includes('        working-directory: root-a\n'),'bootstrap uses the owning checkout');
 const prepared=scope.indexOf(bootstrap);
 assert(scope.slice(0,prepared).includes('          node-version: 22.23.2\n'),'pinned host Node precedes dependency readiness');
 const readers=steps.filter(step=>/\bnode (?:root-a\/)?scripts\/(?:binary-sdk-check|library-sdk-check|browser-api-sdk-check|binary-sdk-release-evidence|library-sdk-metadata-evidence|sdk-release-evidence)\.mjs\b/.test(step));
 assert(readers.length>0,'owning host readers required');
 for(const reader of readers){
  assert(scope.indexOf(reader)>prepared,'compound readiness precedes every host reader');
  assert.doesNotMatch(reader,/^        (?:if|continue-on-error):|\|\| true|\bif false\b|\bset \+e\b/m,'mandatory host reader');
 }
 const after=scope.slice(prepared+bootstrap.length);
 assert.doesNotMatch(after,/\b(?:NODE_PATH|PRISMPM_SEMANTIC_SOURCE)=/,'host readers cannot override admitted helper or Node environment');
 for(const [variable,suffix] of [['PRISMPM_SEMANTIC_SOURCE','/debug/prismpm-semantic-source'],['NODE_PATH','-node-oracles/node_modules']]){
  const expected='${{ runner.temp }}/'+(release?'release-semantic-target':'binary-helper-target')+suffix;
  for(const match of after.matchAll(new RegExp('^          '+variable+': (.*)$','gm')))assert.equal(match[1],expected,'host reader cannot replace admitted '+variable);
 }
 if(release){
  assert(scope.includes('      - name: Retain original semantic helper prerequisite diagnostics\n        if: always()'),'failed compound prerequisites retain diagnostics');
  assert(scope.includes('          path: ${{ runner.temp }}/release-semantic-evidence/\n'),'retain both prerequisite originals');
  return;
 }
 const boundary='      - name: '+(generated?'Check all owning boundary tests before running any writer':'Check exact source and all owning boundary tests')+'\n'+
  '        shell: bash\n        env:\n'+
  '          PRISMPM_SEMANTIC_SOURCE: ${{ runner.temp }}/binary-helper-target/debug/prismpm-semantic-source\n'+
  '          NODE_PATH: ${{ runner.temp }}/binary-helper-target-node-oracles/node_modules\n'+
  '          CARGO_NET_OFFLINE: "true"\n        run: |\n          set -euo pipefail\n';
 assert.equal(scope.split(boundary).length,2,'one offline owner bound to the built Rust and Node helpers');
 const tests=scope.indexOf(boundary),command=scope.indexOf('          node scripts/binary-sdk-check.mjs tests > "'+evidence+'/boundary-tests.log" 2>&1');
 const downstream=scope.indexOf(generated?'      - name: Run the unchanged owning writers':'      - name: Build exact native SDK OCI bytes');
 assert(prepared<tests&&tests<command&&command<downstream,'helper readiness precedes every owner and SDK/writer operation');
 assert.doesNotMatch(scope.slice(prepared,downstream),/^        if:|\|\| true|\bif false\b|\bset \+e\b/m,'fail-closed host prerequisite');
 if(!generated){
  const initialized=scope.indexOf('      - name: Initialize source-bound prerequisite diagnostics');
  assert(initialized>=0&&initialized<prepared,'diagnostics exist before acquisition');
  assert.ok(workflow.includes('      - name: Retain prerequisite and SDK phase status\n        if: always()'),'failed prerequisites retain status');
 }
 assert.ok(workflow.includes('          trap \'status=$?; printf \"%s\\n\" \"$status\" > \"'+evidence+'/boundary-step-exit-status.txt\"\' EXIT'),'owner exit status retained');
 assert.ok(workflow.includes('            ${{ runner.temp }}/'+(generated?'binary-helper-evidence/':'binary-qualification/setup/')),'upload host prerequisite logs on failure');
}
export function validateGenerationWorkflow(workflow){
 validateHelperBootstrapWorkflow(workflow,true);
 for(const required of [
  'branches: [work/binary-generated-review, work/binary-generated-review-521]','  workflow_dispatch:',
  '  contents: read','    runs-on: ubuntu-24.04','          ref: ${{ github.sha }}',
  '          persist-credentials: false','          node-version: 22.23.2',
  'test "$SOURCE_REVISION" = "$GITHUB_SHA"','test "$(git rev-parse HEAD)" = "$SOURCE_REVISION"',
  'test -z "$(git status --porcelain)"','node scripts/binary-sdk-check.mjs tests',
  'uses: devcontainers/ci@513af61f4de4f75d37e4438f184ba4358f0fc1ca','          push: never',
  'git rev-parse HEAD^{tree}','cp model/dependencies.toml','cp Cargo.lock',
  'Review-only generated-source proposal. Not SDK acceptance or a release.',
  '          include-hidden-files: true','          if-no-files-found: error',
 ])assert.ok(workflow.includes(required),'generated review boundary: '+required);
 for(const forbidden of ['work/binary-program-prerequisite','pull_request:','pull_request_target:',': write','continue-on-error','git push','--push','docker/login-action','/etc/os-release','LD_PRELOAD'])assert.ok(!workflow.includes(forbidden),'generated review forbidden: '+forbidden);
 let previous=-1;
 for(const command of [
  'just codegen','just fixtures-write','just golden-write "BinaryProgram prerequisite PR #186: exact source $source_revision"',
  'just stdlib-package-write','CARGO_NET_OFFLINE=true cargo package --locked --offline --target-dir target',
  'just golden-check','just stdlib-package-check','just check-fixtures','just package-release-crates-check',
  'git diff --binary HEAD','node scripts/reconcile-lexlean.mjs --validate-generated',
 ]){const next=workflow.indexOf(command);assert.ok(next>previous,'complete ordered owning generation: '+command);previous=next;}
 assert.ok(workflow.indexOf('if: always()')>previous,'failure diagnostics must not admit a partial patch');
 const generation='      - name: Run the unchanged owning writers and their checks in the pinned devcontainer\n';
 const preservation='      - name: Preserve only an inert review patch after all owning checks pass\n';
 const validation='      - name: Run the complete generated-evidence audit in the pinned devcontainer\n';
 const diagnostics='      - name: Preserve diagnostic status even when preparation fails\n';
 for(const name of [generation,preservation,validation,diagnostics])assert.equal(workflow.split(name).length,2,'one original preparation owner');
 const start=workflow.indexOf(generation),capture=workflow.indexOf(preservation),audit=workflow.indexOf(validation),end=workflow.indexOf(diagnostics);
 assert(start<capture&&capture<audit&&audit<end,'reviewable generation precedes the mandatory audit');
 assert.ok(workflow.slice(start,capture).startsWith(generation+'        id: generated_prefix\n        uses: devcontainers/ci@513af61f4de4f75d37e4438f184ba4358f0fc1ca\n        with:\n          push: never\n          runCmd: |\n            set -euo pipefail\n'));
 assert.equal(workflow.slice(audit,end),validation+
  '        # Preserve reviewable shared records before checking their native joins.\n'+
  '        # A failed proposal capture cannot suppress this mandatory audit.\n'+
  "        if: ${{ always() && !cancelled() && steps.generated_prefix.outcome == 'success' }}\n"+
  '        uses: devcontainers/ci@513af61f4de4f75d37e4438f184ba4358f0fc1ca\n'+
  '        with:\n          push: never\n          runCmd: |\n            set -euo pipefail\n'+
  '            exec > target/binary-generated-review/validation.log 2>&1\n'+
  '            node scripts/reconcile-lexlean.mjs --validate-generated\n');
 assert.equal(workflow.split('node scripts/reconcile-lexlean.mjs --validate-generated').length,2,'one complete audit');
 assert.doesNotMatch(workflow.slice(start,audit),/^        if:|\|\| true|\bif false\b|\bset \+e\b/m,'unconditional fail-closed preparation owners');
}
export function testOutput(output){assert.equal(output.error,undefined);assert.equal(output.signal,null);assert.equal(output.status,0);assert.equal(verifyTap(output.stdout,20),20,'complete original binary gate tests and added custody controls');}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const [mode,...args]=process.argv.slice(2);
 if(mode==='roots'&&args.length===0)console.log(sourceRoots.join('\n'));
 else if(mode==='capture'&&args.length===2)console.log(canonical(capture(...args)));
 else if(mode==='verify'&&args.length===2)verifySource(args[0],JSON.parse(readFileSync(args[1])));
 else if(mode==='hash'&&args.length===1)console.log(hash(canonical(JSON.parse(readFileSync(args[0])))));
 else if(mode==='image'&&args.length===3)verifyImage(JSON.parse(readFileSync(0)),...args);
 else if(mode==='inventory'&&args.length===2)console.log(inventoryEvidence(regularBytes(args[0]),args[1]).sha256);
 else if(mode==='result'&&[6,7].includes(args.length))verifyResult(JSON.parse(readFileSync(args[0])),...args.slice(1));
 else if(mode==='evidence'&&[2,3].includes(args.length)){
  const receipt=document(join(args[0],'receipt.json')),result=JSON.parse(readFileSync(args[1]));
  assert.equal(receipt.build_id,result.build_id);assert.equal(receipt.attestation_id,result.attestation_id);
  checkAccepted(args[0],receipt,{archive_sha256:hash(regularBytes(new URL('../vendor/lean4-prod/lean.tar',import.meta.url))),
   toolchain:regularBytes(new URL('../lean-toolchain',import.meta.url)).toString().trim()},args[2]??'1.1');
 }
 else if(mode==='format-image'&&args.length===2){const value=JSON.parse(readFileSync(args[0]));assert.equal(args[1],'ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21');verifySharedImage(value,args[1],'amd64',value[0]?.Config?.Labels?.['org.opencontainers.image.revision'],'0.3.0');}
 else if(mode==='format-evidence'&&args.length===3){const [directory,sourceRevision,image]=args;revision(sourceRevision);assert.match(image,imagePattern);const inventory=regularBytes(join(directory,'inventory.json'));validateInventory(inventory);const patch=regularBytes(join(directory,'format.patch')),paths=regularBytes(join(directory,'changed-paths.txt')).toString().trim().split('\n').filter(Boolean);assert.ok(paths.every(path=>!path.startsWith('/')&&!path.split('/').some(part=>['','..','.'].includes(part))&&/^(?:crates\/[^/]+\/|xtask\/).+\.rs$/.test(path)),'formatter changed non-owned Rust source');assert.equal(patch.length===0,paths.length===0);writeFileSync(join(directory,'format-result.json'),canonical({schema:'prismpm/source-format-review/1',scope:'source-format-only',source_revision:sourceRevision,sdk_image:image,inventory_sha256:hash(inventory),patch_sha256:hash(patch),formatter_log_sha256:hash(regularBytes(join(directory,'formatter.log'))),changed_paths:paths,status:patch.length?'review-required':'passed',unclaimed:['compilation','proof-verification','binary-package-acceptance','sdk-release','deployment']}));}
 else if(mode==='tests'&&args.length===0){
  const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),env={...process.env};delete env.NODE_TEST_CONTEXT;
  const suite=(files,sequential=true)=>{
   const output=spawnSync(process.execPath,['--test',...(sequential?['--test-concurrency=1']:[]),
    '--test-reporter=tap','--test-timeout=120000',...files],
   {cwd:root,env,encoding:'utf8',timeout:150000,maxBuffer:16*1024*1024});
   process.stdout.write(output.stdout??'');process.stderr.write(output.stderr??'');return output;
  };
  testOutput(suite(['scripts/binary-sdk-check.test.mjs','scripts/binary-sdk-check-shell.test.mjs','scripts/binary-sdk-format-shell.test.mjs']));
  for(const [files,count,message,sequential]of [
   [['scripts/binary-sdk-language12.test.mjs'],3,'additional language-1.2 controls',true],
   [['scripts/sdk-owned-session.test.mjs'],19,'complete additional owned-retirement controls',true],
   [['scripts/semantic-source-bootstrap.test.mjs'],13,'complete separate Rust and Node prerequisite controls',false],
   [['scripts/semantic-source-runtime.test.mjs'],6,'complete separate private installed-helper controls',false],
  ]){
   const output=suite(files,sequential);assert.equal(output.error,undefined);assert.equal(output.signal,null);assert.equal(output.status,0);
   assert.equal(verifyTap(output.stdout,count),count,message);
  }
 }
 else if(mode==='run'&&[4,5].includes(args.length))console.log(canonical(run(resolve(dirname(fileURLToPath(import.meta.url)),'..'),...args)));
 else throw Error('closed installed binary gate command');
}
