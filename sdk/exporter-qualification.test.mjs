// Synthetic closed-reader data only, never installed compiler acceptance.
import assert from 'node:assert/strict';
import test from 'node:test';
import {validateQualification} from './exporter-qualification.mjs';
import {lockFixture, qualificationFixture, sourceAuthorityFixture} from '../scripts/library-sdk-fixture.mjs';

test('compiler qualification retains both native constructions and all eight original exports', () => {
  for (const architecture of ['amd64', 'arm64']) {
    const binding=lockFixture().binding(architecture);
    validateQualification(qualificationFixture(binding), binding, sourceAuthorityFixture);
  }
});

test('qualification rejects missing phases, substituted authority and changed executions or exports', () => {
  const binding=lockFixture().binding('amd64');
  const mutations=[
    value=>{delete value.construction;},
    value=>{value.binding.platform='linux/arm64';},
    value=>{value.manifests.pop();},
    value=>{value.manifests[1]+='\n';},
    value=>{value.construction.raw_construction.pop();},
    value=>{value.construction.raw_construction[1]=structuredClone(value.construction.raw_construction[0]);},
    value=>{value.relocation.observations.pop();},
    value=>{value.relocation.observations[0].kernel.exit_code=1;},
    value=>{value.relocation.observations[0].kernel.argv[2]='--version';},
    value=>{value.relocation.observations[0].kernel.executable_sha256='0'.repeat(64);},
    value=>{value.relocation.observations[0].kernel.environment.LEAN_NUM_THREADS='9999';},
    value=>{value.relocation.observations[1].changed_build_files=[{path:'.lake/build/trace',change:'changed'}];},
    value=>{value.relocation.observations[0].exports.pop();},
    value=>{value.relocation.observations[0].exports[1].artifacts['kernel.ir']='other';},
    value=>{delete value.relocation.observations[0].exports[0].artifacts['coverage.json'];},
    value=>{value.relocation.observations[3]=structuredClone(value.relocation.observations[1]);},
  ];
  for(const mutate of mutations){const value=qualificationFixture(binding);mutate(value);assert.throws(()=>validateQualification(value,binding,sourceAuthorityFixture));}
  for(const authority of [{...sourceAuthorityFixture,archive_sha256:'0'.repeat(64)},{...sourceAuthorityFixture,toolchain:'leanprover/lean4:v4.30.0'}])
    assert.throws(()=>validateQualification(qualificationFixture(binding),binding,authority));
});
