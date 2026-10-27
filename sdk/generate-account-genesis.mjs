// SDK construction only: never accepts a caller-supplied model/artifact/receipt.
import assert from 'node:assert/strict';
import {lstatSync,mkdirSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {renderAccountGenesisBinding,verifyAccountGenesisConstruction} from './account-genesis-artifact.mjs';
import {createCompilerOwner} from '../tests/browser-view/compiler-owner.mjs';
import {prepareAccountGenesis,describeAccountGenesisArtifact,repository,sha} from '../tests/browser-account-genesis/compile.mjs';
import {requireGeneratedWasm} from '../tests/browser-view/generated-wasm.mjs';
const [mode,...extra]=process.argv.slice(2);
assert(extra.length===0 && ['check','install'].includes(mode),'closed account-genesis construction command');
const compiler=createCompilerOwner('account-genesis');
let build,retired=false,completed=false,failed=false;
try {
build=prepareAccountGenesis(compiler);
const binding=describeAccountGenesisArtifact(build),source=renderAccountGenesisBinding(binding);
const artifact=requireGeneratedWasm(build.wasmOwners['account-genesis']);
const wasm=Buffer.from(artifact.bytes),bindingPath=join(repository,'sdk/browser/account-genesis-binding.mjs');
const wasmPath=join(repository,'sdk/browser/account-genesis.wasm');
const proofs=['attestation.json','build-manifest.json'].map(name=>[name,readFileSync(join(build.verified.root,name))]);
proofs.push(['kernel.ir',readFileSync(join(build.work,'export/kernel.ir'))]);
for(const path of [bindingPath,wasmPath]) {
  const stat=lstatSync(path);assert(stat.isFile()&&!stat.isSymbolicLink()&&stat.nlink===1,'single-link generated SDK artifact');
}
assert.equal(readFileSync(bindingPath,'utf8'),source,'binding must match actual complete model/kernel/package constructor');
assert.deepEqual(readFileSync(wasmPath),wasm,'committed artifact must equal both fresh generated Wasm builds');
build.unchanged();const retirement=compiler.close();retired=true;
assert.throws(()=>compiler.runDriver(['--help'],build.work),/compiler owner closed/);
build.unchangedSourcesAndProducts();
if(mode==='install') {
  const output='/opt/prismpm/browser/account-genesis.wasm',proofRoot='/opt/prismpm/share/account-genesis';
  writeFileSync(output,wasm,{flag:'wx',mode:0o444});
  mkdirSync(proofRoot,{recursive:false});
  const rows=[];
  for(const [name,bytes] of proofs) {
    assert(bytes.length<=64*1024**2);writeFileSync(join(proofRoot,name),bytes,{flag:'wx',mode:0o444});
    rows.push({path:name,bytes:bytes.length,sha256:sha(bytes)});
  }
  writeFileSync(join(proofRoot,'construction.json'),JSON.stringify({schema:'prismpm/account-genesis-construction/1',
    binding,installed:[['account-genesis-binding.mjs',Buffer.from(source)],['account-genesis.wasm',wasm]]
      .map(([path,bytes])=>({path,bytes:bytes.length,sha256:sha(bytes)})),
    proofs:rows,inputs:build.inputs,scope:'construction-only-not-account-service-or-full-owner-acceptance'})+'\n',{flag:'wx',mode:0o444});
  assert.deepEqual(readFileSync(output),wasm);
  assert.equal(readFileSync('/opt/prismpm/browser/account-genesis-binding.mjs','utf8'),source);
  assert.deepEqual(verifyAccountGenesisConstruction('/opt/prismpm',build.inputs),binding);
}
console.log(JSON.stringify({mode,binding,retirement,scope:'actual account-genesis artifact construction; complete DK33/SDK acceptance remains mandatory'}));
rmSync(build.work,{recursive:true,force:false});
completed=true;
} catch(error) {failed=true;throw error;}
finally {
  if(!retired)try {compiler.close();}catch(error){if(!failed)throw error;}
  if(build&&!completed)process.stderr.write('Retained incomplete account-genesis construction '+build.work+'\n');
}
