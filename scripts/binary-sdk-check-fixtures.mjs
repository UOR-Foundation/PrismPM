// Synthetic parser/recording-Docker fixtures only; never installed execution evidence.
import {mkdirSync,writeFileSync,renameSync} from 'node:fs';
import {dirname,join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {canonical,hash,fixtureProgram,modes,processTools,binaryPaths,unclaimed,tree} from './binary-sdk-check.mjs';
const revision='a'.repeat(40);
const put=(root,path,bytes)=>{mkdirSync(dirname(join(root,path)),{recursive:true});writeFileSync(join(root,path),bytes);};
export function inventory(){
 const commands=['cargo','devcontainer','docker','just','prismpm'].map(command=>({command,executable:'/usr/local/bin/'+command,sha256:'c'.repeat(64)}));
 const kinds=['adapter','base-image','binary','crate','dependency-lock','oracle','schema','test-corpus','trust-root','workflow'];
 const artifacts=kinds.map(kind=>({id:kind,kind,version:'1',digest:'sha256:'+'d'.repeat(64)}));
 artifacts.push({id:'prismpm',kind:'binary',version:'0.3.0',digest:'sha256:'+'c'.repeat(64)},{id:'sdk-vv-source',kind:'test-corpus',version:revision,digest:'sha256:'+'e'.repeat(64)});
 artifacts.sort((a,b)=>a.id.localeCompare(b.id));return{schema:'prismpm/sdk-inventory/1',commands,artifacts};
}
export function parserFixture(root,change=()=>{}){
 const lexId='b'.repeat(64),build='.prism/build/staging';
 const model={schema:'prismpm/model-document/5',architecture:{component_kinds:[],components:[],concerns:[],edge_kinds:[],edges:[],model_kinds:[],stakeholders:[],viewpoints:[],views:[]},quality:{characteristics:[],measures:[],requirements:[],subcharacteristics:[]},security:{activities:[],assets:[],controls:[],impacts:[],likelihoods:[],measurements:[],risks:[],threats:[]},standards_profile:[],provenance:{compiler_semantics_id:'c'.repeat(64),emitter_semantics_id:'d'.repeat(64),facet_packages:[],semantic_id:'e'.repeat(64),snapshot_id:'f'.repeat(64),source_id:'1'.repeat(64)},program:structuredClone(fixtureProgram)};
 const paths=[...binaryPaths];
 const acceptance={build_id:'',io_coverage:{platform:'linux',output_write:'passed'},executions:modes.map(mode=>({mode,vector_count:fixtureProgram.acceptance_vectors.length,status:'passed'})),program:structuredClone(fixtureProgram),lexlean_attestation_id:lexId,model_id:'',profile:'prismpm/binary-program/1',regeneration:'byte-identical',schema:'prismpm/binary-acceptance/1',scope:'binary-package-only',status:'passed',unclaimed:[...unclaimed]};
 const processes=processTools.map(tool=>({tool,argv:tool==='binary-cli-allocation-acceptance'?['test','--locked','--offline','--release','--','--exact','tests::adapter_allocation_maps_real_capacity_overflow']:[],executable_sha256:'2'.repeat(64),exit_code:0,stdout:tool==='binary-cli-allocation-acceptance'?'running 1 test\ntest tests::adapter_allocation_maps_real_capacity_overflow ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n':['binary-std-acceptance','binary-no_std-acceptance'].includes(tool)?JSON.stringify(fixtureProgram.acceptance_vectors.map(row=>row.response))+'\n':tool==='binary-transports-acceptance'?JSON.stringify({executions:modes.slice(2).map(mode=>({mode,vector_count:fixtureProgram.acceptance_vectors.length,status:'passed'})),io_coverage:{platform:'linux',output_write:'passed'}})+'\n':'',stderr:''}));
 const manifest={acceptance_sha256:'',artifacts:[],build_id:'',lexlean_attestation_sha256:'',model_sha256:'',processes,schema:'prismpm/binary-verification-manifest/1',scope:'binary-package-only'};
 const modules=[{lean_module:'LibraryProbe.Foundation.Binary.V1.Model',declarations:['AdapterFailure','AdapterFailureResult','BinaryAcceptanceVector','BinaryProgram','PublicationOutcome','RawFileCli','adapterFailureResult','failurePublication'].map(lean_name=>({lean_name,axiom_policy:{kind:'none',axioms:[]}}))},{lean_module:'LibraryProbe.Probe',declarations:['identity','probeProgram'].map(lean_name=>({lean_name,axiom_policy:{kind:'none',axioms:[]}}))}];
 const lex={lexlean:{compiler_semantics:model.provenance.compiler_semantics_id},build_manifest:{sha256:hash('parser only')},attestation_id:lexId,build_id:'3'.repeat(64),source_id:model.provenance.source_id,semantic_id:model.provenance.semantic_id,spec:'lexlean/attestation/1',status:'verified',declarations:modules.flatMap(module=>module.declarations.map(row=>({name:module.lean_module+'.'+row.lean_name,observed:[],policy:row.axiom_policy,result:'ok'})))};
 const inputs={schema:'prismpm/build-inputs/4',application_generator_sha256:'4'.repeat(64),dependency_register_sha256:'5'.repeat(64),emitter_semantics_id:model.provenance.emitter_semantics_id,lexlean_build_id:lex.build_id,lexlean_semantic_id:lex.semantic_id,lexlean_source_id:lex.source_id,binary_artifacts_sha256:'',binary_generator_sha256:'6'.repeat(64),model_id:'',system_id:null};
 const binding={model_id:'',program:structuredClone(fixtureProgram),profile:'prismpm/binary-program/1',schema:'prismpm/binary-build-binding/1',scope:'binary-package-only'};
 change({model,acceptance,manifest,paths,lex,inputs,binding,modules});
 const modelBytes=canonical(model);put(root,build+'/model.prism.json',modelBytes);manifest.model_sha256=hash(modelBytes);acceptance.model_id=manifest.model_sha256;binding.model_id=manifest.model_sha256;
 for(const path of paths){const bytes=path==='binary/model-binding.json'?canonical(binding):'parser-only '+path;put(root,build+'/'+path,bytes);manifest.artifacts.push({path,byte_length:Buffer.byteLength(bytes),sha256:hash(bytes)});}
 put(root,build+'/lexlean/build/manifest.json','parser only');put(root,build+'/lexlean/snapshot.json',canonical({modules}));
 for(const name of ['Foundation/Binary/V1/Model','Probe'])for(const path of ['coverage/LibraryProbe/'+name+'.coverage.json','lexicons/'+name.replaceAll('/','.')+'.closure.json','maps/LibraryProbe/'+name+'.map.json','modules/LibraryProbe/'+name+'.lean','modules/LibraryProbe/'+name+'.tex'])put(root,build+'/lexlean/build/'+path,'parser only');
 const rows=tree(join(root,build)).filter(row=>row.kind==='file').map(row=>({byte_length:row.size,kind:'artifact',path:row.path,sha256:row.sha256}));inputs.model_id=manifest.model_sha256;inputs.binary_artifacts_sha256=hash(canonical(rows));const buildId=hash(canonical(inputs));put(root,build+'/manifest.json',canonical({schema:'prismpm/build-manifest/1',files:rows,inputs}));renameSync(join(root,build),join(root,'.prism/build/'+buildId));
 manifest.build_id=buildId;acceptance.build_id=buildId;const lexBytes=canonical(lex);manifest.lexlean_attestation_sha256=hash(lexBytes);const acceptanceBytes=canonical(acceptance);manifest.acceptance_sha256=hash(acceptanceBytes);const manifestBytes=canonical(manifest),attestation=hash(manifestBytes),verified='.prism/verified/'+attestation;
 put(root,verified+'/manifest.json',manifestBytes);put(root,verified+'/binary-acceptance.json',acceptanceBytes);put(root,verified+'/lexlean-attestation.json',lexBytes);
 return{schema:'prismpm/verify-result/1',build_id:buildId,attestation_id:attestation,verified_root:verified};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const [root]=process.argv.slice(2);if(process.argv.length!==3)throw Error('one test fixture root required');
 const receipt=parserFixture(root);put(root,'receipt.json',canonical(receipt));console.log(JSON.stringify(receipt));
}
