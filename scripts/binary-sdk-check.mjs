// DK-29 installed binary-package qualification. Never full SDK/product acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {constants,closeSync,cpSync,existsSync,fstatSync,lstatSync,mkdirSync,mkdtempSync,openSync,readFileSync,readdirSync,readSync,rmSync,chmodSync,writeFileSync} from 'node:fs';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {tree,cli,mutateModule,sourceRoots as libraryRoots,sourceAliases,verifyImage} from './library-sdk-check.mjs';
import {verifyTap} from './browser-api-sdk-check.mjs';
import {validateInventory} from '../sdk/platform-lock.mjs';
export {tree,cli,mutateModule,sourceAliases,verifyImage};

export const sourceRoots=Object.freeze([...libraryRoots,
 'crates/prismpm/tests/binary_program.rs','crates/conformance/src/cases/binary_program.rs',
 'tests/fixtures/binary/binary-program/project','scripts/binary-sdk-check.mjs',
 'scripts/binary-sdk-check.sh','scripts/binary-sdk-check.test.mjs',
 'scripts/binary-sdk-check-fixtures.mjs','scripts/binary-sdk-check-shell.test.mjs','scripts/binary-sdk-check.md','scripts/binary-sdk-qualify.sh','scripts/binary-sdk-format.sh','scripts/binary-sdk-format-shell.test.mjs','scripts/sdk-candidate.sh','scripts/sdk-candidate.mjs','scripts/sdk-candidate-sbom.mjs','Justfile',
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
export function regularBytes(path,maximum=64*1024*1024){
 const before=lstatSync(path);assert.ok(before.isFile()&&!before.isSymbolicLink()&&before.size<=maximum,'bounded regular file: '+path);
 const fd=openSync(path,constants.O_RDONLY|constants.O_NOFOLLOW|constants.O_NONBLOCK);
 try{
  const opened=fstatSync(fd);for(const key of ['dev','ino','size','mtimeMs','ctimeMs'])assert.equal(opened[key],before[key],'file changed');
  const bytes=Buffer.alloc(opened.size+1);let size=0;
  while(size<bytes.length){const count=readSync(fd,bytes,size,bytes.length-size,null);if(!count)break;size+=count;}
  assert.equal(size,opened.size,'file length changed');
  for(const after of [fstatSync(fd),lstatSync(path)]){assert.ok(after.isFile()&&!after.isSymbolicLink());for(const key of ['dev','ino','size','mtimeMs','ctimeMs'])assert.equal(after[key],opened[key],'file changed');}
  return bytes.subarray(0,size);
 }finally{closeSync(fd);}
}
const document=path=>{const bytes=regularBytes(path),value=JSON.parse(bytes);assert.ok(bytes.equals(Buffer.from(canonical(value))),'canonical artifact bytes: '+path);return value;};
export function capture(root,sourceRevision){revision(sourceRevision);return{revision:sourceRevision,files:tree(root,sourceRoots,sourceAliases)};}
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
export function checkAccepted(project,receipt){
 keys(receipt,['schema','build_id','attestation_id','verified_root']);assert.equal(receipt.schema,'prismpm/verify-result/1');hex(receipt.build_id);hex(receipt.attestation_id);
 assert.equal(receipt.verified_root,'.prism/verified/'+receipt.attestation_id);
 const verified=join(project,receipt.verified_root),build=join(project,'.prism/build',receipt.build_id);
 for(const relative of [receipt.verified_root,'.prism/build/'+receipt.build_id]){let selected=resolve(project);for(const part of ['',...relative.split('/')]){if(part)selected=join(selected,part);const stat=lstatSync(selected);assert.ok(stat.isDirectory()&&!stat.isSymbolicLink(),'evidence directory alias');}}
 same(tree(verified).map(row=>row.path),['/','binary-acceptance.json','lexlean-attestation.json','manifest.json'],'unexpected verified outputs');
 const manifestBytes=regularBytes(join(verified,'manifest.json')),manifest=document(join(verified,'manifest.json'));
 const acceptanceBytes=regularBytes(join(verified,'binary-acceptance.json')),acceptance=document(join(verified,'binary-acceptance.json'));
 keys(manifest,['acceptance_sha256','artifacts','build_id','lexlean_attestation_sha256','model_sha256','processes','schema','scope']);
 assert.equal(hash(manifestBytes),receipt.attestation_id);assert.equal(manifest.schema,'prismpm/binary-verification-manifest/1');assert.equal(manifest.scope,'binary-package-only');assert.equal(manifest.build_id,receipt.build_id);
 const lexBytes=regularBytes(join(verified,'lexlean-attestation.json')),lexAttestation=JSON.parse(lexBytes);hex(lexAttestation.attestation_id);
 assert.equal(manifest.acceptance_sha256,hash(acceptanceBytes));assert.equal(manifest.lexlean_attestation_sha256,hash(lexBytes));
 const modelBytes=regularBytes(join(build,'model.prism.json')),model=document(join(build,'model.prism.json'));
 keys(model,['architecture','program','provenance','quality','schema','security','standards_profile']);assert.equal(model.schema,'prismpm/model-document/5');
 same(model.architecture,{component_kinds:[],components:[],concerns:[],edge_kinds:[],edges:[],model_kinds:[],stakeholders:[],viewpoints:[],views:[]},'no application architecture claim');
 same(model.quality,{characteristics:[],measures:[],requirements:[],subcharacteristics:[]},'no application quality claim');
 same(model.security,{activities:[],assets:[],controls:[],impacts:[],likelihoods:[],measurements:[],risks:[],threats:[]},'no application security claim');same(model.standards_profile,[]);
 keys(model.provenance,['compiler_semantics_id','emitter_semantics_id','facet_packages','semantic_id','snapshot_id','source_id']);same(model.provenance.facet_packages,[]);
 for(const [key,value] of Object.entries(model.provenance))if(key!=='facet_packages')hex(value);
 same(model.program,fixtureProgram,'exact whole fixture program descriptor');assert.equal(manifest.model_sha256,hash(modelBytes));
 same(acceptance,{build_id:receipt.build_id,io_coverage:{platform:'linux',output_write:'passed'},executions:modes.map(mode=>({mode,vector_count:fixtureProgram.acceptance_vectors.length,status:'passed'})),program:fixtureProgram,lexlean_attestation_id:lexAttestation.attestation_id,model_id:hash(modelBytes),profile:'prismpm/binary-program/1',regeneration:'byte-identical',schema:'prismpm/binary-acceptance/1',scope:'binary-package-only',status:'passed',unclaimed},'binary acceptance scope or execution differs');
 assert.ok(Array.isArray(manifest.processes));same(manifest.processes.map(row=>row.tool),processTools,'complete ordered binary verification processes');
 for(const row of manifest.processes){keys(row,['tool','argv','executable_sha256','exit_code','stdout','stderr']);assert.equal(row.exit_code,0);hex(row.executable_sha256);assert.ok(Array.isArray(row.argv)&&row.argv.every(arg=>typeof arg==='string'));assert.equal(typeof row.stdout,'string');assert.equal(typeof row.stderr,'string');}
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
 assert.equal(lexAttestation.spec,'lexlean/attestation/1');assert.equal(lexAttestation.status,'verified');assert.equal(lexAttestation.build_id,inputs.lexlean_build_id);assert.equal(lexAttestation.lexlean?.compiler_semantics,model.provenance.compiler_semantics_id);assert.equal(lexAttestation.build_manifest?.sha256,hash(regularBytes(join(build,'lexlean/build/manifest.json'))));assert.equal(lexAttestation.source_id,inputs.lexlean_source_id);assert.equal(lexAttestation.semantic_id,inputs.lexlean_semantic_id);
 assert.ok(Array.isArray(lexAttestation.declarations));same(lexAttestation.declarations.map(row=>row.name).sort(),names,'complete published declaration audit');
 for(const row of lexAttestation.declarations){keys(row,['name','observed','policy','result']);assert.equal(row.result,'ok');same(row.policy,declarations.find(declaration=>declaration.name===row.name).policy);same(row.policy,{axioms:[],kind:'none'});same(row.observed,[],'fixture has no observed axioms');}
 const binary=artifacts.filter(row=>row.path.startsWith('binary/'));same(binary.map(row=>row.path),binaryPaths,'complete fixture binary artifact set');
 same(manifest.artifacts,binary.map(row=>({path:row.path,byte_length:row.size,sha256:row.sha256})),'complete exact binary replay artifacts');
 same(document(join(build,'binary/model-binding.json')),{model_id:hash(modelBytes),program:fixtureProgram,profile:'prismpm/binary-program/1',schema:'prismpm/binary-build-binding/1',scope:'binary-package-only'},'whole binary model binding');
 return{build,model_id:hash(modelBytes)};
}
function noVerification(project){const path=join(project,'.prism/verified');assert.ok(!existsSync(path)||readdirSync(path).length===0,'unaccepted source published verification');}
const declaration=(module,name)=>{const rows=module.declarations.filter(row=>row.name===name);assert.equal(rows.length,1);return rows[0];};
const rootField=(module,name)=>{const rows=declaration(module,'probeProgram').body.fields.filter(row=>row.field===name);assert.equal(rows.length,1);return rows[0].value;};
export function rejectChangedEvidence(project,receipt,path){
 const original=regularBytes(path);
 try{rmSync(path);assert.throws(()=>checkAccepted(project,receipt));writeFileSync(path,Buffer.concat([original,Buffer.from([0])]));assert.throws(()=>checkAccepted(project,receipt));}
 finally{writeFileSync(path,original);}
 checkAccepted(project,receipt);
}
export function run(root,image,sourceRevision,sourceSha256,inventorySha256){
 assert.equal(process.getuid(),1000,'native SDK gate must run non-root');assert.match(image,imagePattern);revision(sourceRevision);hex(sourceSha256);hex(inventorySha256);
 assert.equal(hash(canonical(capture(root,sourceRevision))),sourceSha256,'installed current source binding');
 const inventory=inventoryEvidence(regularBytes('/opt/prismpm/share/inventory.json'),sourceRevision);assert.equal(inventory.sha256,inventorySha256);
 assert.equal(hash(regularBytes('/usr/local/bin/prismpm',256*1024*1024)),inventory.cli_sha256,'actual installed CLI matches immutable inventory');
 const work=mkdtempSync('/tmp/prismpm-binary-sdk-'),source=join(root,'tests/fixtures/binary/binary-program/project');let sequence=0;
 function fixture(){const destination=join(work,'fixture-'+sequence++);cpSync(source,destination,{recursive:true,errorOnExist:true,force:false});for(const row of tree(destination))chmodSync(join(destination,row.path),row.kind==='directory'?0o700:0o600);assert.ok(!existsSync(join(destination,'.prism')));return destination;}
 const positive=(project,args)=>cli(project,args,{schema:'prismpm/'+args[0]+'-result/1'});
 try{
  const first=fixture(),before=tree(first),checked=positive(first,['check']);same(tree(first),before,'check modified source');assert.equal(checked.entity_count,1);
  const accepted=positive(first,['verify']),evidence=checkAccepted(first,accepted);assert.equal(checked.model_id,evidence.model_id);
  const firstBuild=tree(evidence.build),second=fixture(),secondBuild=positive(second,['build']);assert.equal(secondBuild.build_id,accepted.build_id);
  same(tree(join(second,'.prism/build',secondBuild.build_id)),firstBuild,'complete build differs across fresh absolute roots');noVerification(second);
  const secondVerify=positive(second,['verify']);assert.equal(secondVerify.build_id,accepted.build_id);checkAccepted(second,secondVerify);
  const beforeProduct=tree(first);cli(first,['build','--locked','-t','ghcr.io/uor-foundation/prismpm-binary-probe:0.1.0'],{code:'PP6101'});same(tree(first),beforeProduct,'product refusal wrote outputs');
  const descriptor=regularBytes(join(source,'src/Foundation/Binary/V1/Model.lex.tex')).toString('utf8'),rows=descriptor.split('\n').filter(line=>line.startsWith('\\semanticdata{'));assert.equal(rows.length,1);const nominal=JSON.parse(rows[0].slice('\\semanticdata{'.length,-1));
  const mutations=[
   module=>{rootField(module,'entryRoot').value='LibraryProbe.Probe.missing';rootField(module,'exportRoots').head.value='LibraryProbe.Probe.missing';},
   module=>{declaration(module,'identity').result={kind:'nat'};declaration(module,'identity').body={kind:'nat',value:'0'};},
   module=>{declaration(module,'identity').parameters.push({name:'unused',type:{kind:'nat'}});},
   module=>{module.declarations.unshift(...nominal.declarations);delete declaration(module,'probeProgram').result.member.module;delete declaration(module,'probeProgram').body.type.module;},
  ];
  for(const change of mutations){const invalid=fixture();mutateModule(invalid,change);const before=tree(invalid);cli(invalid,['check'],{code:'PP2001'});same(tree(invalid),before,'invalid check wrote outputs');noVerification(invalid);}
  const mutant=fixture(),original=mutateModule(mutant,module=>{rootField(module,'acceptanceVectors').head.fields.find(row=>row.field==='response').value={kind:'bytes',hex:'01'};});
  const changed=positive(mutant,['check']);assert.notEqual(changed.semantic_id,checked.semantic_id);cli(mutant,['verify'],{code:'PP5006'});noVerification(mutant);
  writeFileSync(join(mutant,'src/Probe.lex.tex'),original);const restored=positive(mutant,['verify']);assert.equal(restored.build_id,accepted.build_id);checkAccepted(mutant,restored);
  for(const name of ['lexlean-attestation.json','binary-acceptance.json'])rejectChangedEvidence(first,accepted,join(first,accepted.verified_root,name));
  rejectChangedEvidence(first,accepted,join(evidence.build,'binary/package/src/lib.rs'));
  // A prior passing receipt is never an input to current verification.
  const freshlyVerified=positive(first,['verify']);checkAccepted(first,freshlyVerified);assert.equal(freshlyVerified.build_id,accepted.build_id);
  const retained='/tmp/prismpm-binary-evidence';assert.ok(!existsSync(retained),'fresh evidence output');mkdirSync(join(retained,'.prism/build'),{recursive:true});mkdirSync(join(retained,'.prism/verified'),{recursive:true});
  cpSync(evidence.build,join(retained,'.prism/build',accepted.build_id),{recursive:true,errorOnExist:true,force:false});
  cpSync(join(first,freshlyVerified.verified_root),join(retained,freshlyVerified.verified_root),{recursive:true,errorOnExist:true,force:false});writeFileSync(join(retained,'receipt.json'),canonical(freshlyVerified));
  return{schema:'prismpm/installed-binary-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:image,source_revision:sourceRevision,source_sha256:sourceSha256,inventory_sha256:inventory.sha256,architecture:({x64:'amd64',arm64:'arm64'}[process.arch]),build_id:accepted.build_id,attestation_id:freshlyVerified.attestation_id,checks:completedChecks,unclaimed};
 }finally{rmSync(work,{recursive:true,force:true});}
}
export function verifyResult(value,image,sourceRevision,architecture,sourceSha256,inventorySha256){
 assert.match(image,imagePattern);revision(sourceRevision);assert.ok(['amd64','arm64'].includes(architecture));hex(sourceSha256);hex(inventorySha256);hex(value.build_id);hex(value.attestation_id);
 same(value,{schema:'prismpm/installed-binary-check/1',scope:'installed-binary-package-only',status:'passed',sdk_image:image,source_revision:sourceRevision,source_sha256:sourceSha256,inventory_sha256:inventorySha256,architecture,build_id:value.build_id,attestation_id:value.attestation_id,checks:completedChecks,unclaimed},'complete installed binary result');
}
export function testOutput(output){assert.equal(output.error,undefined);assert.equal(output.signal,null);assert.equal(output.status,0);assert.equal(verifyTap(output.stdout,16),16,'complete owning binary gate test count');}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const [mode,...args]=process.argv.slice(2);
 if(mode==='roots'&&args.length===0)console.log(sourceRoots.join('\n'));
 else if(mode==='capture'&&args.length===2)console.log(canonical(capture(...args)));
 else if(mode==='verify'&&args.length===2)verifySource(args[0],JSON.parse(readFileSync(args[1])));
 else if(mode==='hash'&&args.length===1)console.log(hash(canonical(JSON.parse(readFileSync(args[0])))));
 else if(mode==='image'&&args.length===3)verifyImage(JSON.parse(readFileSync(0)),...args);
 else if(mode==='inventory'&&args.length===2)console.log(inventoryEvidence(regularBytes(args[0]),args[1]).sha256);
 else if(mode==='result'&&args.length===6)verifyResult(JSON.parse(readFileSync(args[0])),...args.slice(1));
 else if(mode==='evidence'&&args.length===2){const receipt=document(join(args[0],'receipt.json')),result=JSON.parse(readFileSync(args[1]));assert.equal(receipt.build_id,result.build_id);assert.equal(receipt.attestation_id,result.attestation_id);checkAccepted(args[0],receipt);}
 else if(mode==='format-image'&&args.length===2){const value=JSON.parse(readFileSync(args[0]));assert.equal(args[1],'ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21');verifyImage(value,args[1],'amd64',value[0]?.Config?.Labels?.['org.opencontainers.image.revision']);}
 else if(mode==='format-evidence'&&args.length===3){const [directory,sourceRevision,image]=args;revision(sourceRevision);assert.match(image,imagePattern);const inventory=regularBytes(join(directory,'inventory.json'));validateInventory(inventory);const patch=regularBytes(join(directory,'format.patch')),paths=regularBytes(join(directory,'changed-paths.txt')).toString().trim().split('\n').filter(Boolean);assert.ok(paths.every(path=>!path.startsWith('/')&&!path.split('/').some(part=>['','..','.'].includes(part))&&path.endsWith('.rs')),'formatter changed non-Rust source');assert.equal(patch.length===0,paths.length===0);writeFileSync(join(directory,'format-result.json'),canonical({schema:'prismpm/source-format-review/1',scope:'source-format-only',source_revision:sourceRevision,sdk_image:image,inventory_sha256:hash(inventory),patch_sha256:hash(patch),formatter_log_sha256:hash(regularBytes(join(directory,'formatter.log'))),changed_paths:paths,status:patch.length?'review-required':'passed',unclaimed:['compilation','proof-verification','binary-package-acceptance','sdk-release','deployment']}));}
 else if(mode==='tests'&&args.length===0){const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),env={...process.env};delete env.NODE_TEST_CONTEXT;const output=spawnSync(process.execPath,['--test','--test-concurrency=1','--test-reporter=tap','--test-timeout=120000','scripts/binary-sdk-check.test.mjs','scripts/binary-sdk-check-shell.test.mjs','scripts/binary-sdk-format-shell.test.mjs'],{cwd:root,env,encoding:'utf8',timeout:150000,maxBuffer:16*1024*1024});process.stdout.write(output.stdout??'');process.stderr.write(output.stderr??'');testOutput(output);}
 else if(mode==='run'&&args.length===4)console.log(canonical(run(resolve(dirname(fileURLToPath(import.meta.url)),'..'),...args)));
 else throw Error('closed installed binary gate command');
}
