// Independent readiness qualification, not full SDK or product acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {lstatSync,readdirSync,readFileSync,realpathSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';

assert.equal(process.argv.length,2);
const root='/opt/prismpm/share/conformance-root';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const owners=['scripts/browser-environment-preflight.sh','scripts/browser-environment-preflight.mjs',
  'sdk/browser/browser-test-server.mjs','sdk/browser-engine/authority.json','sdk/browser-engine/install.py'];
for(const path of owners) assert.equal(sha(readFileSync(join(root,path))),sha(readFileSync(join('/review',path))),path);
const inventoryBytes=readFileSync('/opt/prismpm/share/inventory.json');
const inventory=JSON.parse(inventoryBytes);
function commands() {
  const result=JSON.parse(execFileSync('prismpm',['--json','completion','bash'],{timeout:120000,maxBuffer:1048576}));
  assert.equal(result.schema,'prismpm/completion-result/1');
  assert.equal(sha(readFileSync('/opt/prismpm/share/inventory.json')),sha(inventoryBytes));
  // CLI admission measures every actual PATH command against the installed inventory.
  return sha(Buffer.from(JSON.stringify(result)));
}
function browserTrees() {
  const trees=[];
  for(const name of readdirSync('/ms-playwright').sort()) {
    const directory=join('/ms-playwright',name);
    const initial=lstatSync(directory);
    if(!initial.isDirectory()) continue;
    const records=[];const digest=createHash('sha256');
    function walk(path,prefix='') {
      for(const entry of readdirSync(path).sort((a,b)=>Buffer.from(a).compare(Buffer.from(b)))) {
        const selected=join(path,entry),relative=prefix?`${prefix}/${entry}`:entry;
        const stat=lstatSync(selected);
        if(stat.isDirectory()) walk(selected,relative);
        else {
          assert(stat.isFile()||stat.isSymbolicLink(),'ordinary browser file or confined alias');
          if(stat.isSymbolicLink()) assert(realpathSync(selected).startsWith(directory+'/'),'browser alias confinement');
          const bytes=readFileSync(selected),pathBytes=Buffer.from(relative);
          const pathLength=Buffer.alloc(8),byteLength=Buffer.alloc(8);
          pathLength.writeBigUInt64BE(BigInt(pathBytes.length));byteLength.writeBigUInt64BE(BigInt(bytes.length));
          digest.update(pathLength).update(pathBytes).update(byteLength).update(bytes);
          records.push({path:relative,mode:stat.mode&0o777,length:bytes.length,sha256:sha(bytes)});
        }
      }
    }
    walk(directory);assert(records.length>0);
    trees.push({name,digest:'sha256:'+digest.digest('hex'),records});
  }
  const chrome=trees.filter(row=>row.name==='chromium_headless_shell-cft-157.0.8099.0');
  assert.equal(chrome.length,1);
  const expected=inventory.artifacts.filter(row=>row.id==='playwright-headless-shell');
  assert.equal(expected.length,1);assert.equal(chrome[0].digest,expected[0].digest);
  assert(trees.some(row=>row.name==='firefox-1538'));assert(trees.some(row=>row.name==='webkit-2336'));
  return trees;
}
const beforeCommands=commands(),before=browserTrees();
const owner=await import(pathToFileURL(join(root,'scripts/browser-environment-preflight.mjs')));
// Preserve the original owner's 90s TERM / 5s KILL deadline and no-override CLI.
const result=JSON.parse(execFileSync('bash',[join(root,'scripts/browser-environment-preflight.sh')],
  {timeout:100000,maxBuffer:1048576,encoding:'utf8'}));
owner.validateBrowserEnvironment(result);
const after=browserTrees(),afterCommands=commands();
assert.deepEqual(after,before);assert.equal(afterCommands,beforeCommands);
console.log(JSON.stringify({schema:'prismpm/independent-sdk-browser-readiness/1',scope:'browser-readiness-only',
  uid:process.getuid(),architecture:process.arch,inventory_sha256:sha(inventoryBytes),
  commands:inventory.commands.length,artifacts:inventory.artifacts.length,command_result_sha256:beforeCommands,
  owners:owners.map(path=>({path,sha256:sha(readFileSync(join(root,path)))})),result,trees:before,
  unclaimed:['full-sdk-qualification','full-vv','product-readiness','release']}));
