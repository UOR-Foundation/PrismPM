// Installed native-profile acceptance only; not SDK release or product acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {constants,closeSync,cpSync,existsSync,fstatSync,lstatSync,mkdtempSync,openSync,readFileSync,readlinkSync,readdirSync,readSync,rmSync,chmodSync,writeFileSync} from 'node:fs';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {verifyImage,verifyTap} from './browser-api-sdk-check.mjs';
export {verifyImage};

export const sourceRoots=Object.freeze([
 '.cargo','Cargo.toml','Cargo.lock','rust-toolchain.toml','lean-toolchain',
 'crates/prismpm/Cargo.toml','crates/prismpm/src','crates/prismpm/model','crates/prismpm/schemas',
 'crates/prismpm/LICENSE-MIT','crates/prismpm/LICENSE-APACHE','crates/prismpm/vendor',
 'crates/prismpm/stdlib','crates/prismpm/sdk','crates/prismpm/standards','crates/prismpm/standards.lock',
 'crates/prismpm/adapters','crates/prismpm/language',
 'crates/prismpm/tests/native_library.rs','crates/conformance/src/cases/native_library.rs',
 'model','language','schemas','stdlib','sdk','standards','standards.lock','adapters','LICENSE-MIT','LICENSE-APACHE',
 'tests/fixtures/library/native-library/project','tests/hologram-oracle','vendor',
 'scripts/browser-api-sdk-check.mjs','scripts/library-sdk-check.mjs','scripts/library-sdk-check.sh',
 'scripts/library-sdk-check.test.mjs','scripts/library-sdk-check-shell.test.mjs','.github/workflows/release.yml',
]);
// Tracked compiler include aliases only. Their complete target roots are bound
// separately above; unknown, changed or escaping aliases are never followed.
export const sourceAliases=Object.freeze({
 'crates/prismpm/model':'../../model',
 'crates/prismpm/schemas':'../../schemas',
 'crates/prismpm/LICENSE-MIT':'../../LICENSE-MIT',
 'crates/prismpm/LICENSE-APACHE':'../../LICENSE-APACHE',
 'crates/prismpm/vendor':'../../vendor',
 'crates/prismpm/stdlib':'../../stdlib',
 'crates/prismpm/sdk':'../../sdk',
 'crates/prismpm/standards':'../../standards',
 'crates/prismpm/standards.lock':'../../standards.lock',
 'crates/prismpm/adapters':'../../adapters',
 'crates/prismpm/language':'../../language',
 'crates/prismpm/src/embedded/hologram-oracle.main.rs':'../../../../tests/hologram-oracle/src/main.rs',
 'crates/prismpm/src/embedded/hologram-oracle.Cargo.toml':'../../../../tests/hologram-oracle/Cargo.toml',
 'crates/prismpm/src/embedded/hologram-oracle.Cargo.lock':'../../../../tests/hologram-oracle/Cargo.lock',
 'crates/prismpm/src/embedded/lean4-prod-rust.MANIFEST.sha256':'../../../../vendor/lean4-prod/rust/MANIFEST.sha256',
});
const acceptanceRoot='LibraryProbe.Probe.acceptance',identityRoot='LibraryProbe.Probe.identity';
const roots=[acceptanceRoot,identityRoot],unclaimed=['application','browser','holo','production-release','deployment'];
const exits=Object.freeze({PP2001:1,PP5006:1,PP6101:5});
const completedChecks=Object.freeze(['read-only-check','std','no_std','exact-package-replay','two-root-reproduction','product-refusal','missing-root','wrong-result-root','parameterized-root','nominal-impostor','false-generated-acceptance','restored-acceptance']);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const hex=value=>{assert.equal(typeof value,'string');assert.match(value,/^[0-9a-f]{64}$/);return value;};
const canonical=value=>JSON.stringify(value,(_,v)=>v&&typeof v==='object'&&!Array.isArray(v)?Object.fromEntries(Object.keys(v).sort().map(key=>[key,v[key]])):v);
const same=(actual,expected,message)=>assert.ok(canonical(actual)===canonical(expected),message);
const keys=(value,names)=>{assert.ok(value&&typeof value==='object'&&!Array.isArray(value),'closed object');same(Object.keys(value).sort(),names.slice().sort(),'closed object fields');};
function regularBytes(path){
 const before=lstatSync(path);assert.ok(before.isFile()&&!before.isSymbolicLink()&&before.size<=64*1024*1024,'bounded regular file: '+path);
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

export function tree(root,selected=[''],aliases={}){
 root=resolve(root);assert.ok(lstatSync(root).isDirectory()&&!lstatSync(root).isSymbolicLink(),'source root alias');
 const files=[];
 function walk(relative){
  assert.ok(files.length<100000,'source entry limit');const path=join(root,relative),stat=lstatSync(path);
  if(Object.hasOwn(aliases,relative)){
   assert.ok(stat.isSymbolicLink(),'required source alias: '+relative);assert.equal(readlinkSync(path),aliases[relative],'changed source alias');
   const target=resolve(dirname(path),aliases[relative]);assert.ok(target.startsWith(root+'/')&&existsSync(target),'missing or escaping alias target');
   files.push({path:relative,kind:'symlink',target:aliases[relative]});return;
  }
  assert.ok(!stat.isSymbolicLink(),'source symlink: '+relative);
  if(stat.isDirectory()){
   files.push({path:relative+'/',kind:'directory'});
   for(const name of readdirSync(path).sort()){
    // Only generated top-level caches of the selected dependency are excluded.
    if(['vendor/lexlean','vendor/lean4-prod/rust'].includes(relative)&&['target','.lake','.lexlean','.prism'].includes(name))continue;
    walk(relative?relative+'/'+name:name);
   }
  }else{const bytes=regularBytes(path);files.push({path:relative,kind:'file',size:bytes.length,sha256:hash(bytes)});}
 }
 for(const relative of selected){
  let parent=root;for(const part of relative.split('/').slice(0,-1)){parent=join(parent,part);const stat=lstatSync(parent);assert.ok(stat.isDirectory()&&!stat.isSymbolicLink(),'source parent alias');}
  walk(relative);
 }
 files.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0);assert.equal(new Set(files.map(row=>row.path)).size,files.length,'duplicate source entries');
 same(files.filter(row=>row.kind==='symlink').map(row=>row.path),Object.keys(aliases).sort(),'complete tracked source aliases');return files;
}
export function capture(root,revision){assert.equal(typeof revision,'string');assert.match(revision,/^[0-9a-f]{40}$/);return{revision,files:tree(root,sourceRoots,sourceAliases)};}
export function verifySource(root,expected){same(capture(root,expected.revision),expected,'installed SDK source closure differs');}

export function cli(project,args,expected,launch=spawnSync){
 const env={...process.env,CARGO_NET_OFFLINE:'true'};delete env.CARGO_TARGET_DIR;
 const output=launch('/usr/local/bin/prismpm',['--project',project,'--json',...args],{encoding:'utf8',env,timeout:900000,maxBuffer:16*1024*1024});
 assert.equal(output.error,undefined,'CLI execution failed');assert.equal(output.signal,null,'CLI terminated');
 if(expected.code)assert.ok(Object.hasOwn(exits,expected.code),'declared diagnostic exit class');
 assert.equal(output.status,expected.code ? exits[expected.code] : 0, "CLI exit class");
 let value;try{value=JSON.parse(output.stdout);}catch{throw new Error('CLI did not emit exactly one JSON result');}
 if(expected.code){assert.equal(value.schema,'prismpm/error-result/1');assert.equal(value.diagnostic?.code,expected.code);if(expected.message)assert.equal(value.diagnostic.message,expected.message);}
 else{
  assert.equal(value.schema,expected.schema,'CLI success schema');
  const fields={
   'prismpm/check-result/1':['schema','semantic_id','snapshot_id','model_id','entity_count'],
   'prismpm/build-result/1':['schema','build_id','source_id','semantic_id','model_path','manifest_path'],
   'prismpm/verify-result/1':['schema','build_id','attestation_id','verified_root'],
  };
  assert.ok(Object.hasOwn(fields,value.schema),'declared result contract');keys(value,fields[value.schema]);
  for(const [name,item] of Object.entries(value))if(name.endsWith('_id'))hex(item);
  if(value.schema==='prismpm/check-result/1')assert.ok(Number.isSafeInteger(value.entity_count)&&value.entity_count>=0);
  if(value.schema==='prismpm/build-result/1'){assert.equal(value.model_path,'.prism/build/'+value.build_id+'/model.prism.json');assert.equal(value.manifest_path,'.prism/build/'+value.build_id+'/manifest.json');}
  if(value.schema==='prismpm/verify-result/1')assert.equal(value.verified_root,'.prism/verified/'+value.attestation_id);
 }
 return value;
}
function document(path){const bytes=regularBytes(path),value=JSON.parse(bytes);assert.ok(bytes.equals(Buffer.from(canonical(value))),'canonical artifact bytes: '+path);return value;}
function noVerification(project){const path=join(project,'.prism/verified');assert.ok(!existsSync(path)||readdirSync(path).length===0,'unaccepted source published verification');}
export function checkAccepted(project,receipt){
 keys(receipt,['schema','build_id','attestation_id','verified_root']);
 assert.equal(receipt.schema,'prismpm/verify-result/1');hex(receipt.build_id);hex(receipt.attestation_id);
 assert.equal(receipt.verified_root,'.prism/verified/'+receipt.attestation_id);
 const verified=join(project,receipt.verified_root),build=join(project,'.prism/build',receipt.build_id);
 same(tree(verified).map(row=>row.path),['/','lexlean-attestation.json','library-acceptance.json','manifest.json'],'unexpected verified outputs');
 const manifestBytes=regularBytes(join(verified,'manifest.json')),manifest=JSON.parse(manifestBytes);
 const acceptanceBytes=regularBytes(join(verified,'library-acceptance.json')),acceptance=JSON.parse(acceptanceBytes);
 assert.ok(manifestBytes.equals(Buffer.from(canonical(manifest))),'canonical verification manifest');assert.ok(acceptanceBytes.equals(Buffer.from(canonical(acceptance))),'canonical library acceptance');
 keys(manifest,['acceptance_sha256','artifacts','build_id','lexlean_attestation_sha256','model_sha256','processes','schema','scope']);
 assert.equal(hash(manifestBytes),receipt.attestation_id);assert.equal(manifest.schema,'prismpm/library-verification-manifest/1');assert.equal(manifest.scope,'native-library-only');assert.equal(manifest.build_id,receipt.build_id);
 const lexBytes=regularBytes(join(verified,'lexlean-attestation.json')),lexAttestation=JSON.parse(lexBytes);
 assert.equal(manifest.acceptance_sha256,hash(acceptanceBytes));assert.equal(manifest.lexlean_attestation_sha256,hash(lexBytes));hex(lexAttestation.attestation_id);
 const modelBytes=regularBytes(join(build,'model.prism.json')),model=JSON.parse(modelBytes);
 assert.ok(modelBytes.equals(Buffer.from(canonical(model))),'canonical model document');
 keys(model,['architecture','library','provenance','quality','schema','security','standards_profile']);
 same(model.architecture,{component_kinds:[],components:[],concerns:[],edge_kinds:[],edges:[],model_kinds:[],stakeholders:[],viewpoints:[],views:[]},'no application architecture claim');
 same(model.quality,{characteristics:[],measures:[],requirements:[],subcharacteristics:[]},'no application quality claim');
 same(model.security,{activities:[],assets:[],controls:[],impacts:[],likelihoods:[],measurements:[],risks:[],threats:[]},'no application security claim');same(model.standards_profile,[],'no application standards claim');
 keys(model.provenance,['compiler_semantics_id','emitter_semantics_id','facet_packages','semantic_id','snapshot_id','source_id']);same(model.provenance.facet_packages,[],'no application facet packages');
 for(const [key,value] of Object.entries(model.provenance))if(key!=='facet_packages')hex(value);
 same(model.library,{profile:'prismpm/native-library/1',name:'Library probe',cargo_name:'prism-library-probe',cargo_version:'0.1.0',cargo_description:'Finite native-library acceptance fixture',cargo_repository:'https://github.com/UOR-Foundation/PrismPM',cargo_homepage:'https://github.com/UOR-Foundation/PrismPM',export_roots:roots,acceptance_roots:[acceptanceRoot]},'exact fixture library descriptor');
 assert.equal(manifest.model_sha256,hash(modelBytes));assert.equal(model.schema,'prismpm/model-document/3');assert.equal(model.application,undefined);assert.equal(model.library.profile,'prismpm/native-library/1');
 same(acceptance,{build_id:receipt.build_id,executions:['std','no_std'].map(mode=>({mode,roots:[acceptanceRoot],status:'passed'})),export_roots:roots,lexlean_attestation_id:lexAttestation.attestation_id,model_id:hash(modelBytes),profile:'prismpm/native-library/1',regeneration:'byte-identical',schema:'prismpm/library-acceptance/1',scope:'native-library-only',status:'passed',unclaimed},'library acceptance scope or execution differs');
 assert.ok(Array.isArray(manifest.processes)&&manifest.processes.length>0);assert.ok(manifest.processes.every(row=>row.exit_code===0),'failed recorded execution');
 same(manifest.processes.map(row=>row.tool),['lean-version','lake-version','rustfmt-version','rustc-version','timeout-version','lake-build-generated','lean4-prod-build','prod-export','native-library-package','native-library-std-lock','native-library-std-acceptance','native-library-no_std-lock','native-library-no_std-acceptance'],'complete ordered native verification processes');
 for(const row of manifest.processes){keys(row,['tool','argv','executable_sha256','exit_code','stdout','stderr']);hex(row.executable_sha256);assert.ok(Array.isArray(row.argv)&&row.argv.every(arg=>typeof arg==='string'));assert.equal(typeof row.stdout,'string');assert.equal(typeof row.stderr,'string');}
 for(const mode of ['std','no_std']){const rows=manifest.processes.filter(row=>row.tool==='native-library-'+mode+'-acceptance');assert.equal(rows.length,1);same(JSON.parse(rows[0].stdout),{roots:[acceptanceRoot],status:'passed'},'actual native execution transcript');}
 const artifacts=tree(build).filter(row=>row.kind==='file');assert.ok(artifacts.every(row=>!row.path.endsWith('.holo')&&!row.path.startsWith('application/')),'native library claims application artifacts');
 const buildManifest=document(join(build,'manifest.json'));keys(buildManifest,['files','inputs','schema']);assert.equal(buildManifest.schema,'prismpm/build-manifest/1');
 const inputs=buildManifest.inputs;keys(inputs,['application_generator_sha256','dependency_register_sha256','emitter_semantics_id','lexlean_build_id','lexlean_semantic_id','lexlean_source_id','library_artifacts_sha256','library_generator_sha256','model_id','schema','system_id']);assert.equal(inputs.schema,'prismpm/build-inputs/3');assert.equal(inputs.system_id,null);
 for(const [name,value] of Object.entries(inputs))if(!['schema','system_id'].includes(name))hex(value);
 assert.equal(hash(canonical(inputs)),receipt.build_id,'artifact-bound build identity');assert.equal(inputs.model_id,hash(modelBytes));assert.equal(inputs.emitter_semantics_id,model.provenance.emitter_semantics_id);assert.equal(inputs.lexlean_semantic_id,model.provenance.semantic_id);assert.equal(inputs.lexlean_source_id,model.provenance.source_id);
 assert.ok(Array.isArray(buildManifest.files));for(const row of buildManifest.files){keys(row,['byte_length','kind','path','sha256']);assert.ok(['lean','latex','source-map','coverage','lexicon-closure','lexlean-manifest','semantic-snapshot','artifact'].includes(row.kind));}
 same(buildManifest.files.map(({kind,...row})=>row),artifacts.filter(row=>row.path!=='manifest.json').map(row=>({path:row.path,byte_length:row.size,sha256:row.sha256})),'complete build manifest file closure');
 assert.equal(inputs.library_artifacts_sha256,hash(canonical(buildManifest.files)),'complete artifact row digest');
 const lexPaths=['lexlean/build/manifest.json','lexlean/snapshot.json'];
 for(const name of ['Foundation/Library/V1/Model','Probe']){lexPaths.push('lexlean/build/coverage/LibraryProbe/'+name+'.coverage.json','lexlean/build/lexicons/'+name.replaceAll('/','.')+'.closure.json','lexlean/build/maps/LibraryProbe/'+name+'.map.json','lexlean/build/modules/LibraryProbe/'+name+'.lean','lexlean/build/modules/LibraryProbe/'+name+'.tex');}
 same(artifacts.filter(row=>!row.path.startsWith('library/')).map(row=>row.path),[...lexPaths,'manifest.json','model.prism.json'].sort(),'complete fixture non-library artifacts');
 // LexLean owns its snapshot serialization; the exact bytes are bound above.
 const snapshot=JSON.parse(regularBytes(join(build,'lexlean/snapshot.json')));assert.ok(Array.isArray(snapshot.modules));const declarations=snapshot.modules.flatMap(module=>module.declarations.map(declaration=>({name:module.lean_module+'.'+declaration.lean_name,policy:declaration.axiom_policy})));
 const names=['LibraryProbe.Foundation.Library.V1.Model.NativeLibrary','LibraryProbe.Probe.acceptance','LibraryProbe.Probe.identity','LibraryProbe.Probe.probeLibrary'];same(declarations.map(row=>row.name).sort(),names,'complete selected declaration closure');
 assert.equal(lexAttestation.spec,'lexlean/attestation/1');assert.equal(lexAttestation.status,'verified');assert.equal(lexAttestation.build_id,inputs.lexlean_build_id);assert.equal(lexAttestation.source_id,inputs.lexlean_source_id);assert.equal(lexAttestation.semantic_id,inputs.lexlean_semantic_id);
 assert.ok(Array.isArray(lexAttestation.declarations));same(lexAttestation.declarations.map(row=>row.name).sort(),names,'complete published declaration audit');
 for(const row of lexAttestation.declarations){keys(row,['name','observed','policy','result']);assert.equal(row.result,'ok');same(row.policy,declarations.find(declaration=>declaration.name===row.name).policy,'exact declaration axiom policy');same(row.policy,{axioms:[],kind:'none'},'fixture has no admitted axioms');same(row.observed,[],'fixture has no observed axioms');}
 const library=artifacts.filter(row=>row.path.startsWith('library/'));
 same(library.map(row=>row.path),['library/coverage.json','library/kernel.ir','library/model-binding.json','library/package/Cargo.lock','library/package/Cargo.toml','library/package/LICENSE-APACHE','library/package/LICENSE-MIT','library/package/README.md','library/package/generation-manifest.json','library/package/src/lib.rs','library/prism-library-probe-0.1.0.crate','library/roots.json'],'complete fixture native artifact set');
 same(manifest.artifacts,library.map(row=>({path:row.path,byte_length:row.size,sha256:row.sha256})),'complete exact native package replay artifacts');
 same(document(join(build,'library/model-binding.json')),{acceptance_roots:[acceptanceRoot],export_roots:roots,model_id:hash(modelBytes),profile:'prismpm/native-library/1',schema:'prismpm/library-build-binding/1',scope:'native-library-only'},'closed native model binding');
 return {build,model_id:hash(modelBytes)};
}
export function mutateModule(project,change){
 const path=join(project,'src/Probe.lex.tex'),before=regularBytes(path).toString('utf8'),lines=before.trimEnd().split('\n');
 const indices=lines.flatMap((line,index)=>line.startsWith('\\semanticdata{')?[index]:[]);assert.equal(indices.length,1,'one semantic module');
 const index=indices[0];assert.ok(lines[index].endsWith('}'));const module=JSON.parse(lines[index].slice('\\semanticdata{'.length,-1));change(module);
 lines[index]='\\semanticdata{'+canonical(module)+'}';writeFileSync(path,lines.join('\n')+'\n');return before;
}
const declaration=(module,name)=>{const rows=module.declarations.filter(row=>row.name===name);assert.equal(rows.length,1);return rows[0];};
const rootField=(module,name)=>{const rows=declaration(module,'probeLibrary').body.fields.filter(row=>row.field===name);assert.equal(rows.length,1);return rows[0].value;};

export function run(root){
 assert.equal(process.getuid(),1000,'native SDK gate must run non-root');
 const work=mkdtempSync('/tmp/prismpm-library-sdk-'),source=join(root,'tests/fixtures/library/native-library/project');let sequence=0;
 function fixture(){
  const destination=join(work,'fixture-'+sequence++);cpSync(source,destination,{recursive:true,errorOnExist:true,force:false});
  for(const row of tree(destination))chmodSync(join(destination,row.path),row.kind==='directory'?0o700:0o600);
  assert.ok(!existsSync(join(destination,'.prism')),'fixture must be fresh');return destination;
 }
 const positive=(project,args)=>cli(project,args,{schema:'prismpm/'+args[0]+'-result/1'});
 try{
  const first=fixture(),before=tree(first),checked=positive(first,['check']);same(tree(first),before,'check modified source');
  const accepted=positive(first,['verify']),evidence=checkAccepted(first,accepted);assert.equal(checked.model_id,evidence.model_id);
  const firstBuild=tree(evidence.build),second=fixture(),secondBuild=positive(second,['build']);assert.equal(secondBuild.build_id,accepted.build_id);
  same(tree(join(second,'.prism/build',secondBuild.build_id)),firstBuild,'complete build differs across fresh absolute roots');noVerification(second);
  const secondVerify=positive(second,['verify']);assert.equal(secondVerify.build_id,accepted.build_id);checkAccepted(second,secondVerify);
  const beforeProduct=tree(first);cli(first,['build','--locked','-t','ghcr.io/uor-foundation/prismpm-library-probe:0.1.0'],{code:'PP6101',message:'native-library acceptance is not product-release or deployment acceptance'});same(tree(first),beforeProduct,'product refusal wrote outputs');
  const descriptor=regularBytes(join(source,'src/Foundation/Library/V1/Model.lex.tex')).toString('utf8');const descriptorRow=descriptor.split('\n').filter(line=>line.startsWith('\\semanticdata{'));assert.equal(descriptorRow.length,1);const nominal=JSON.parse(descriptorRow[0].slice('\\semanticdata{'.length,-1));
  const mutations=[
   [module=>{rootField(module,'exportRoots').tail.head.value='LibraryProbe.Probe.missing';},'native-library export LibraryProbe.Probe.missing is not defined'],
   [module=>{rootField(module,'acceptanceRoots').head.value=identityRoot;},'native-library acceptance root LibraryProbe.Probe.identity must have type Bool with no parameters'],
   [module=>{declaration(module,'acceptance').parameters=[{name:'unused',type:{kind:'nat'}}];},'native-library acceptance root LibraryProbe.Probe.acceptance must have type Bool with no parameters'],
   [module=>{module.declarations.unshift(nominal.declarations[0]);delete declaration(module,'probeLibrary').result.member.module;delete declaration(module,'probeLibrary').body.type.module;},'facet closure is not exact'],
  ];
  for(const [change,message] of mutations){const invalid=fixture();mutateModule(invalid,change);const before=tree(invalid);cli(invalid,['check'],{code:'PP2001',message});same(tree(invalid),before,'invalid check wrote outputs');noVerification(invalid);}
  const mutant=fixture(),original=mutateModule(mutant,module=>{declaration(module,'identity').body={kind:'add',left:{kind:'var',name:'value'},right:{kind:'nat',value:'1'}};});
  const changed=positive(mutant,['check']);assert.notEqual(changed.semantic_id,checked.semantic_id);cli(mutant,['verify'],{code:'PP5006'});noVerification(mutant);
  writeFileSync(join(mutant,'src/Probe.lex.tex'),original);const restored=positive(mutant,['verify']);assert.equal(restored.build_id,accepted.build_id);checkAccepted(mutant,restored);
  return {scope:'installed-native-library-only',build_id:accepted.build_id,checks:completedChecks,unclaimed};
 }finally{rmSync(work,{recursive:true,force:true});}
}

export function verifyResult(value){hex(value.build_id);same(value,{scope:'installed-native-library-only',build_id:value.build_id,checks:completedChecks,unclaimed},'complete installed native-library result');}
export function testOutput(output){assert.equal(output.error,undefined);assert.equal(output.signal,null);assert.equal(output.status,0);assert.equal(verifyTap(output.stdout,15),15,'complete owning gate test count');}

if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const [mode,...args]=process.argv.slice(2);
 if(mode==='roots'&&args.length===0)console.log(sourceRoots.join('\n'));
 else if(mode==='capture'&&args.length===2)console.log(JSON.stringify(capture(...args)));
 else if(mode==='verify'&&args.length===2)verifySource(args[0],JSON.parse(readFileSync(args[1])));
 else if(mode==='image'&&args.length===3)verifyImage(JSON.parse(readFileSync(0)),...args);
 else if(mode==='result'&&args.length===1)verifyResult(JSON.parse(readFileSync(args[0])));
 else if(mode==='tests'&&args.length===0){const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),env={...process.env};delete env.NODE_TEST_CONTEXT;const output=spawnSync(process.execPath,['--test','--test-concurrency=1','--test-reporter=tap','--test-timeout=120000','scripts/library-sdk-check.test.mjs','scripts/library-sdk-check-shell.test.mjs'],{cwd:root,env,encoding:'utf8',timeout:150000,maxBuffer:16*1024*1024});process.stdout.write(output.stdout??'');process.stderr.write(output.stderr??'');testOutput(output);}
 else if(mode==='run'&&args.length===0)console.log(JSON.stringify(run(resolve(dirname(fileURLToPath(import.meta.url)),'..'))));
 else throw Error('closed installed native-library gate command');
}
