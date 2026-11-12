// Candidate checks only. Copy exact harness bytes into fresh root-owned
// volumes; never change ownership of the checkout or weaken installed custody.
import assert from 'node:assert/strict';
import {chmodSync,constants,copyFileSync,lstatSync,mkdirSync,readdirSync,realpathSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {snapshotTree} from './exporter-seed.mjs';

const roots=['scripts','sdk'];
const bounds={files:8192,file:32*1024**2,total:64*1024**2};
const mode=row=>row.kind==='directory'||(row.mode&0o111)?0o555:0o444;

export function captureOverlay(root,{installed=false}={}){
 root=resolve(root);assert.equal(realpathSync(root),root,'overlay root alias');
 const rows=[];
 for(const name of roots){
  const path=join(root,name),custody=new Map();
  const members=snapshotTree(path,{bounds,custody});
  for(const [relative,stat] of custody){
   if(installed){
    assert.equal(stat.uid,0n,'overlay must be root-owned: '+join(path,relative));
    assert.equal(Number(stat.mode&0o777n),stat.isDirectory()||(stat.mode&0o111n)?0o555:0o444,
     'overlay must be immutable: '+join(path,relative));
   }
  }
  rows.push({path:name,kind:'directory',mode:0o555});
  for(const row of members)rows.push({...row,path:name+'/'+row.path,mode:mode(row)});
 }
 return rows;
}

export function stageOverlay(source,destination){
 assert.equal(process.getuid(),0,'overlay construction must run as root');
 source=resolve(source);destination=resolve(destination);
 assert.equal(realpathSync(destination),destination,'overlay destination alias');
 assert(source!==destination&&!destination.startsWith(source+'/')&&!source.startsWith(destination+'/'));
 const before=captureOverlay(source);
 // Only the two dedicated, newly-created volumes may be written. Never
 // recursively chown/chmod host inputs or any installed compiler member.
 for(const name of roots){
  const path=join(destination,name),stat=lstatSync(path);
  assert(stat.isDirectory()&&!stat.isSymbolicLink()&&stat.uid===0&&(stat.mode&0o022)===0,'fresh root-owned overlay volume required');
  assert.deepEqual(readdirSync(path),[],'overlay volume must be empty');
 }
 for(const row of before){
  const target=join(destination,row.path);
  if(row.kind==='directory'){
   if(!roots.includes(row.path))mkdirSync(target,{mode:0o700});
  }else{
   copyFileSync(join(source,row.path),target,constants.COPYFILE_EXCL);
   chmodSync(target,row.mode);
  }
 }
 for(const row of before.toReversed())if(row.kind==='directory')chmodSync(join(destination,row.path),row.mode);
 assert.deepEqual(captureOverlay(source),before,'harness source changed during staging');
 assert.deepEqual(captureOverlay(destination,{installed:true}),before,'staged harness bytes differ');
 return before;
}

if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const [command,...args]=process.argv.slice(2);
 if(command==='capture'&&args.length===1)console.log(JSON.stringify(captureOverlay(...args)));
 else if(command==='stage'&&args.length===2)console.log(JSON.stringify(stageOverlay(...args)));
 else if(command==='verify'&&args.length===1)console.log(JSON.stringify(captureOverlay(args[0],{installed:true})));
 else throw Error('closed qualification overlay command');
}
