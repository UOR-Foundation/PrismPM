// Synthetic parser/recording-boundary data only; never an SDK execution claim.
import {createHash} from 'node:crypto';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const encode=value=>JSON.stringify(value,(_,item)=>item&&typeof item==='object'&&!Array.isArray(item)
 ?Object.fromEntries(Object.keys(item).sort().map(key=>[key,item[key]])):item);

export function seedManifestFixture(architecture, authority=sourceAuthorityFixture) {
 const directory=path=>({path,kind:'directory',mode:0o755});
 const file=path=>({path,kind:'file',mode:0o755,byte_length:12,sha256:hash(architecture+path)});
 const toolchain=authority.toolchain;
 return {schema:'prismpm/exporter-seed/1',platform:'linux/'+architecture,compiler_revision:'a'.repeat(40),
  archive_sha256:authority.archive_sha256,toolchain,
  configuration:{argv:['build','prod-export'],construction_root:'/tmp/prismpm-exporter-construction',temporary_directory:'private-bounded-tmpfs',
   environment:{PATH:'/usr/local/elan/toolchains/'+toolchain.replace('/','--').replace(':','---')+'/bin:/usr/bin:/bin',LANG:'C',LC_ALL:'C',ELAN_HOME:'/usr/local/elan',ELAN_TOOLCHAIN:toolchain,LEAN_NUM_THREADS:'2',SOURCE_DATE_EPOCH:'0'}},
  source_files:[directory('Prod'),file('Prod/Export.lean')],toolchain_files:[directory('bin'),file('bin/lake'),file('bin/lean')],
  runtime_files:[{selected:'/lib/libc.so.6',path:'/usr/lib/libc.so.6',mode:0o755,byte_length:12,sha256:hash('runtime')}],
  files:[directory('.lake'),directory('.lake/build'),directory('.lake/build/bin'),file('.lake/build/bin/prod-export')]};
}

export function lockFixture(authority=sourceAuthorityFixture){
 const standards=Buffer.from('synthetic library SDK standards\n'),platforms=[],children=[];
 for(const architecture of ['amd64','arm64']){
  const manifest=seedManifestFixture(architecture,authority);
  const artifacts=['adapter','base-image','binary','crate','dependency-lock','oracle','schema','test-corpus','trust-root','workflow']
   .map(kind=>({id:'test-'+kind,kind,version:'fixture',digest:'sha256:'+hash(architecture+kind)}));
  artifacts.push({id:'lean4-prod-exporter',kind:'binary',version:'a'.repeat(40),digest:'sha256:'+manifest.files.at(-1).sha256},
   {id:'lean4-prod-exporter-seed',kind:'dependency-lock',version:'1',digest:'sha256:'+hash(encode(manifest)+'\n')});
  artifacts.sort((a,b)=>a.id.localeCompare(b.id));
  const inventory_document=encode({schema:'prismpm/sdk-inventory/1',artifacts,
   commands:['cargo','devcontainer','docker','just','prismpm'].map(command=>({command,executable:'/usr/local/bin/'+command,sha256:hash(architecture+command)}))});
  const manifest_digest='sha256:'+hash(architecture+'manifest');
  children.push({digest:manifest_digest,size:100,mediaType:'application/vnd.oci.image.manifest.v1+json',platform:{os:'linux',architecture}});
  platforms.push({platform:'linux/'+architecture,manifest_digest,inventory_digest:'sha256:'+hash(inventory_document),inventory_document,inventory:artifacts});
 }
 const sdk_index=encode({schemaVersion:2,mediaType:'application/vnd.oci.image.index.v1+json',manifests:children});
 const image='example.invalid/library-sdk@sha256:'+hash(sdk_index);
 const lock={schema:'prismpm/sdk-lock/2',sdk_version:'0.3.0',sdk_image:image,sdk_index,standards_lock:'sha256:'+hash(standards),platforms};
 const bytes=Buffer.from(encode(lock));
 const binding=architecture=>({sdk_image:image,platform:'linux/'+architecture,lock_sha256:hash(bytes),
  inventory_sha256:hash(platforms.find(row=>row.platform==='linux/'+architecture).inventory_document),
  seed_manifest_sha256:hash(encode(seedManifestFixture(architecture,authority))+'\n'),exporter_sha256:seedManifestFixture(architecture,authority).files.at(-1).sha256,compiler_revision:'a'.repeat(40)});
 return {bytes,lock,image,standards,encode,hash,binding};
}

export const sourceAuthorityFixture={archive_sha256:'3'.repeat(64),toolchain:'leanprover/lean4:v4.32.1'};
export function qualificationFixture(binding,authority=sourceAuthorityFixture) {
 const manifest=seedManifestFixture(binding.platform.split('/')[1],authority),bytes=encode(manifest)+'\n';
 const lake=manifest.configuration.environment.PATH.split(':')[0]+'/lake';
 const base='/work/prismpm-exporter-qualification-fixture';
 const process=(argv,temporary)=>({argv,environment:{...manifest.configuration.environment,TMPDIR:temporary},executable_sha256:argv[0]===lake?manifest.toolchain_files[1].sha256:hash('tar'),exit_code:0,stdout:'synthetic boundary fixture',stderr:''});
 const construction={scope:'exporter-construction-only',manifest_sha256:hash(bytes),files:manifest.files.length,
  raw_construction:['a','b'].map(name=>({extraction:process(['/usr/bin/tar','--extract','--file',base+'/source-'+name+'/vendor/lean4-prod/lean.tar','--directory','/tmp/prismpm-exporter-construction/package'],'/tmp/prismpm-exporter-construction'),build:process([lake,'build','prod-export'],'/tmp/prismpm-exporter-construction')}))};
 const observations=Array.from({length:4},(_,index)=>{
  const acquisition=index%2?'relocated':'cold',root=base+'/exporter-relocation-check-fixture/'+(index<2?'first-root/':'independent-second-root/')+acquisition,temporary='/tmp/exporter-relocation-fixture';
  return {root,acquisition,invocation_milliseconds:1,changed_build_files:[],
   extraction:process(['/usr/bin/tar','--extract','--file',base+'/source-a/vendor/lean4-prod/lean.tar','--directory',root],temporary),
   build:process([lake,'build','prod-export'],temporary),module:process([lake,'build','Conformance.LexLean11'],temporary),kernel:process([lake,'env','leanchecker','Conformance.LexLean11'],temporary),
   exports:['a','b'].map(replay=>({process:process([lake,'exe','prod-export','--module','Conformance.LexLean11','--root','SemanticFixture.Main.allConsecutive','--ir-module','exporter_relocation','--out',root+'/export-'+replay],temporary),artifacts:{'kernel.ir':'synthetic IR','roots.json':'synthetic roots','coverage.json':'synthetic coverage'}}))};
 });
 return {schema:'prismpm/exporter-qualification/1',scope:'installed-native-compiler-only',binding,manifests:[bytes,bytes],construction,
  relocation:{scope:'compiler-relocation-measurement-only',manifest_sha256:hash(bytes),construction:{manifest_sha256:hash(bytes),construction:construction.raw_construction[0]},observations}};
}
export function resultFixture(binding,sourceAuthority=sourceAuthorityFixture){
 const build_id='d'.repeat(64);
 const tools=['lean-version','lake-version','rustfmt-version','rustc-version','timeout-version','lake-build-generated','lean4-prod-build','prod-export','native-library-package','native-library-std-lock','native-library-std-acceptance','native-library-no_std-lock','native-library-no_std-acceptance'];
 const runs=Array.from({length:4},(_,index)=>{
  const acquisition=index<2?'cold':'sdk-seed';
  const processes=tools.map(tool=>({tool,argv:[],executable_sha256:'e'.repeat(64),exit_code:0,stdout:'parser fixture',stderr:''}));
  processes[7].argv=['exe','prod-export'];
  processes[7].exporter={schema:'prismpm/exporter-execution/1',source_archive_sha256:sourceAuthority.archive_sha256,
   executable:{byte_length:1234,mode:0o755,sha256:index<2?'4'.repeat(64):binding.exporter_sha256},
   acquisition:index<2?{schema:'prismpm/exporter-acquisition/1',mode:'cold'}:{schema:'prismpm/exporter-acquisition/1',mode:'sdk-seed',
    platform:binding.platform,inventory_sha256:binding.inventory_sha256,manifest_sha256:binding.seed_manifest_sha256,
    executable_sha256:binding.exporter_sha256,compiler_revision:binding.compiler_revision,toolchain:sourceAuthority.toolchain,archive_sha256:sourceAuthority.archive_sha256}};
  const manifest=encode({schema:'prismpm/library-verification-manifest/1',scope:'native-library-only',build_id,processes,
   artifacts:['coverage.json','kernel.ir','model-binding.json','package/Cargo.lock','package/Cargo.toml','package/LICENSE-APACHE','package/LICENSE-MIT','package/README.md','package/generation-manifest.json','package/src/lib.rs','prism-library-probe-0.1.0.crate','roots.json'].map(path=>({path:'library/'+path,byte_length:12,sha256:hash(path)})),acceptance_sha256:'5'.repeat(64),lexlean_attestation_sha256:'6'.repeat(64),model_sha256:'7'.repeat(64)});
  return {root:'/tmp/prismpm-library-sdk-fixture/fixture-'+index,acquisition,manifest,manifest_sha256:hash(manifest)};
 });
 return {scope:'installed-native-library-only',build_id,binding,runs,checks:['read-only-check','std','no_std','exact-package-replay','two-root-reproduction','product-refusal','missing-root','wrong-result-root','parameterized-root','nominal-impostor','false-generated-acceptance','restored-acceptance','authenticated-seed-admission','cold-warm-two-root-equivalence'],unclaimed:['application','browser','holo','production-release','deployment']};
}
