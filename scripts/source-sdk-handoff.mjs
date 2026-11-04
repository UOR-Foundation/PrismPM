// Authenticated image transport for the original source owner, never acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdirSync,readFileSync,realpathSync,writeFileSync} from 'node:fs';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {validateConfig} from './sdk-candidate.mjs';
import {releaseTransportPolicy,trustRootDigest,verificationArguments} from './sdk-candidate-sbom.mjs';
import {inspectLoadedImage,selectPlatform,validateImageMetadata} from './sdk-vv-check.mjs';
import {executeOwnedSdkCommand} from './sdk-command-owner.mjs';

const sourceRoot=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const sha=bytes=>'sha256:'+createHash('sha256').update(bytes).digest('hex');
const repository='ghcr.io/uor-foundation/prismpm-sdk';
const trustPath='standards/trust/sigstore-trusted-root-cosign-3.1.3.json';

export function sourceSdkVerificationArguments(image,revision,ref,event,trustedRoot){
 assert.match(image,new RegExp('^'+repository.replaceAll('.','\\.')+'@sha256:[0-9a-f]{64}$'));
 return verificationArguments({source_revision:revision},image.split('@')[1],trustedRoot,
  releaseTransportPolicy(repository,ref,event));
}

export function validateSourceSdkMetadata(index,manifest,config,image,arch,revision){
 const chain=validateImageMetadata(index,manifest,image,arch),parsed=JSON.parse(manifest);
 assert.equal(parsed.config.mediaType,'application/vnd.oci.image.config.v1+json');
 assert(Buffer.isBuffer(config)&&config.length<=4*1024**2);
 assert.equal(config.length,parsed.config.size,'platform config byte length differs');
 assert.equal(sha(config),chain.config_digest,'platform config digest differs');
 validateConfig(JSON.parse(config),arch,revision);
 return chain;
}

// All Docker calls inherit the source devcontainer's actual daemon selection.
// No Docker context, daemon, registry, build or alternate source owner is made.
export async function verifySourceSdkHandoff({image,revision,arch,ref,event,source=sourceRoot},
 launch=(command,args,options)=>executeOwnedSdkCommand(command,args,{...options,profile:'vv',interruptions:true})){
 assert.equal(process.platform,'linux');
 assert.equal(arch,process.arch==='x64'?'amd64':process.arch,'native source architecture required');
 source=realpathSync(source);
 const trustedRoot=join(source,trustPath);
 const verification=sourceSdkVerificationArguments(image,revision,ref,event,trustedRoot);
 assert.equal(sha(readFileSync(trustedRoot)),trustRootDigest,'pinned Sigstore trust root differs');
 const call=async(command,args,timeout=120000,limit=8*1024**2)=>{
  const result=await launch(command,args,{environment:process.env,cwd:source,timeout,limit});
  assert.equal(result.signal,null,command+' interrupted');
  assert.equal(result.status,0,command+' failed: '+result.stderr.toString().slice(-4096));
  assert(Buffer.isBuffer(result.stdout)&&result.stdout.length<=limit,'bounded command bytes required');
  return result.stdout;
 };
 const sourceIdentity=async()=>{
  assert.equal((await call('git',['rev-parse','HEAD'])).toString().trim(),revision,'source revision differs');
  assert.equal((await call('git',['status','--porcelain'])).length,0,'clean original source required');
 };
 await sourceIdentity();
 const signature=await call('cosign',verification,300000);
 const index=await call('docker',['buildx','imagetools','inspect','--raw',image]);
 // The signed release SDK names exactly these two native platforms. Admit the
 // complete same-source graph before allowing even the local native pull.
 const descriptors=['amd64','arm64'].map(platform=>selectPlatform(index,image.split('@')[1],platform));
 assert.equal(JSON.parse(index).manifests.length,2,'complete dual-platform SDK index required');
 const platforms={};
 for(const [position,platform]of ['amd64','arm64'].entries()){
  const childImage=repository+'@'+descriptors[position].digest;
  const manifest=await call('docker',['buildx','imagetools','inspect','--raw',childImage]);
  const config=await call('oras',['manifest','fetch-config',childImage]);
  const chain=validateSourceSdkMetadata(index,manifest,config,image,platform,revision);
  platforms[platform]={architecture:platform,index_base64:index.toString('base64'),
   manifest_base64:manifest.toString('base64'),config_base64:config.toString('base64'),chain};
 }
 const {chain,manifest_base64,config_base64}=platforms[arch];
 await call('docker',['pull','--platform','linux/'+arch,image],600000);
 // Reuse both classic and containerd-store identity rules, against this daemon.
 const loaded=await inspectLoadedImage(async args=>({status:0,signal:null,stderr:Buffer.alloc(0),
  stdout:await call('docker',args)}),chain,arch,revision);
 await sourceIdentity();
 return {schema:'prismpm/source-sdk-handoff/1',scope:'authenticated-transport-only-not-source-or-sdk-acceptance',
  source_revision:revision,architecture:arch,image_reference:image,ref,event,
  trust_root_digest:trustRootDigest,verification_arguments:verification,
  signature_verification:signature.toString('utf8'),
  index_base64:index.toString('base64'),manifest_base64,config_base64,
  platforms,chain,loaded,unclaimed:['source-vv','SDK-acceptance','release-promotion','consumer-adoption']};
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 assert.equal(process.argv.length,7,'usage: source-sdk-handoff.mjs IMAGE REV ARCH REF EVENT');
 const [image,revision,arch,ref,event]=process.argv.slice(2);
 const receipt=await verifySourceSdkHandoff({image,revision,arch,ref,event});
 const target=join(sourceRoot,'target');mkdirSync(target,{recursive:true});
 assert.equal(realpathSync(target),target,'unaliased source target required');
 const directory=join(target,'source-sdk-handoff');mkdirSync(directory,{recursive:true});
 assert.equal(realpathSync(directory),directory,'unaliased handoff evidence required');
 // A fresh receipt cannot overwrite any earlier source invocation's handoff.
 const path=join(directory,revision+'-'+arch+'-'+process.pid+'.json');
 writeFileSync(path,JSON.stringify(receipt)+'\n',{flag:'wx',mode:0o444});
 process.stdout.write(path+'\n');
}
