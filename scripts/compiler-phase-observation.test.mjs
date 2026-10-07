// Numeric diagnostic infrastructure only, not compiler or model acceptance.
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {phaseCollector} from './owning-node-reporter.mjs';
import {compilerPhase} from '../tests/browser-view/compile.mjs';
const prefix='# prismpm-compiler-phase ';
const row=(phase='kernel-export',elapsed_ms=12,success=true)=>({phase,elapsed_ms,success});
const encoded=value=>prefix+JSON.stringify(value)+'\n';
test('classifications are closed labels, not private paths or arguments',()=>{
  const cases=[['cargo',['--version'],'toolchain-check'],['cargo',['build'],'rust-compilation'],
    ['tar',['-xf','/private/archive'],'archive-extraction'],['lake',['update'],'lake-update'],
    ['lake',['build','prod-export'],'exporter-construction'],['lake',['build','PrismGenerated'],'generated-module-build'],
    ['/private/prod-export',[],'kernel-export'],['/private/driver',['verify'],'lexlean-verification'],
    ['/private/driver',['native'],'native-code-generation'],['/private/driver',['wasm'],'wasm-code-generation'],
    ['/private/runner',['private-payload'],'generated-execution']];
  for(const [program,args,phase]of cases)assert.equal(compilerPhase(program,args),phase);
});
test('arbitrary chunks and interleaved files preserve exact numeric totals',()=>{
  const c=phaseCollector(),a=encoded(row()),b=encoded(row('kernel-export',8,false));
  for(let i=0;i<a.length;i++){c.accept('/private/a',a[i]);if(i<b.length)c.accept('/private/b',b[i]);}
  for(let i=a.length;i<b.length;i++)c.accept('/private/b',b[i]);
  assert.deepEqual(c.summary(),{scope:'compiler-phase-diagnostic-not-acceptance',
    phases:[{phase:'kernel-export',calls:2,failures:1,elapsed_ms:20}]});
  assert(!JSON.stringify(c.summary()).includes('/private'));
});
test('malformed, unbounded and open diagnostics cannot be echoed or counted',()=>{
  const c=phaseCollector();
  for(const bad of [null,{},row('private-path'),row('kernel-export',-1),row('kernel-export',1.5),
    row('kernel-export',600001),row('kernel-export',1,'yes'),{...row(),secret:'credential'},
    {...row(),elapsed_ms:null}])c.accept('a',encoded(bad));
  c.accept('a',prefix+'{malformed}\n'+encoded(row()).trimEnd());
  assert.deepEqual(c.summary().phases,[],'unterminated messages are not observations');
  c.accept('a','\n'+'x'.repeat(1000000)+'\n'+encoded(row('rust-compilation',0)));
  assert.deepEqual(c.summary().phases,[{phase:'kernel-export',calls:1,failures:0,elapsed_ms:12},
    {phase:'rust-compilation',calls:1,failures:0,elapsed_ms:0}]);
});
test('stream inventory is bounded and an oversized record cannot hide the next valid line',()=>{
  const c=phaseCollector();for(let i=0;i<64;i++)c.accept('file-'+i,'');
  c.accept('unregistered',encoded(row()));
  c.accept('file-0',prefix+' '.repeat(300)+JSON.stringify(row())+'\n'+encoded(row()));
  assert.equal(c.summary().phases[0].calls,1);
});
test('actual Node reporter preserves complete passing and failing test results',t=>{
  const directory=mkdtempSync(join(tmpdir(),'prismpm-phase-reporter-'));t.after(()=>rmSync(directory,{recursive:true}));
  const reporter='data:text/javascript;base64,'+readFileSync(new URL('./owning-node-reporter.mjs',import.meta.url)).toString('base64');
  const environment={...process.env};delete environment.NODE_TEST_CONTEXT;
  for(const fails of [false,true]){
    const file=join(directory,fails?'failure.mjs':'success.mjs');
    writeFileSync(file,`import test from 'node:test';import assert from 'node:assert/strict';test('actual fixture',()=>{process.stderr.write(${JSON.stringify(encoded(row()))});assert.equal(${fails},false);});`);
    const p=spawnSync(process.execPath,['--test','--test-reporter='+reporter,file],{env:environment,encoding:'utf8',timeout:10000,maxBuffer:1024*1024});
    assert.ifError(p.error);assert.equal(p.signal,null);assert.equal(p.status,fails?1:0);
    const lines=p.stdout.split('\n'),fileRows=lines.filter(line=>line.startsWith('# prismpm-owning-file '));
    assert.equal(fileRows.length,1);const completion=JSON.parse(fileRows[0].slice('# prismpm-owning-file '.length));
    assert.equal(completion.tests,1);assert.equal(completion.success,!fails);assert.equal(completion.failed,Number(fails));
    const summaries=lines.filter(line=>line.startsWith('# prismpm-compiler-phase-summary '));
    assert.equal(summaries.length,1);assert.deepEqual(JSON.parse(summaries[0].slice('# prismpm-compiler-phase-summary '.length)).phases,
      [{phase:'kernel-export',calls:1,failures:0,elapsed_ms:12}]);
  }
});
test('real SDK command timings and a closed diagnostic pipe preserve outcomes',async()=>{
  const environment={...process.env};delete environment.NODE_TEST_CONTEXT;
  const program=`import assert from 'node:assert/strict';
    import {run,repository} from ${JSON.stringify(new URL('../tests/browser-view/compile.mjs',import.meta.url).href)};
    function execute(){
      assert.equal(run('node',['-e','process.stdout.write("phase-positive")'],repository),'phase-positive');
      assert.throws(()=>run('node',['-e','process.stdout.write("phase-negative");process.exit(7)'],repository),
        error=>error.code==='ERR_ASSERTION'&&error.actual===7&&error.expected===0);
    }
    execute();process.stderr.write('PHASE_PIPE_READY\\n');
    await new Promise(resolve=>process.stdin.once('data',resolve));
    execute();process.stdout.write('actual-outcomes-preserved');`;
  const child=spawn(process.execPath,['--input-type=module','-e',program],{env:environment,stdio:['pipe','pipe','pipe']});
  let stdout='',stderr='',closed=false;
  child.stdout.on('data',bytes=>{stdout+=bytes;assert(stdout.length<=1024*1024);});
  child.stderr.on('data',bytes=>{
    stderr+=bytes;assert(stderr.length<=1024*1024);
    if(!closed&&stderr.includes('PHASE_PIPE_READY\n')){closed=true;child.stderr.destroy();child.stdin.end('continue');}
  });
  const timeout=setTimeout(()=>child.kill('SIGKILL'),30000);
  let outcome;
  try{outcome=await new Promise((resolve,reject)=>{child.once('error',reject);child.once('close',(status,signal)=>resolve({status,signal}));});}
  finally{clearTimeout(timeout);}
  assert.equal(outcome.signal,null);assert.equal(outcome.status,0,stderr);assert(closed);
  assert.equal(stdout,'actual-outcomes-preserved');
  const records=stderr.split('\n').filter(line=>line.startsWith(prefix)).map(line=>JSON.parse(line.slice(prefix.length)));
  assert.deepEqual(records.filter(row=>row.phase==='generated-execution').map(row=>row.success),[true,false]);
  assert(records.every(row=>Number.isSafeInteger(row.elapsed_ms)&&row.elapsed_ms>=0));
  assert(!stderr.includes('phase-positive')&&!stderr.includes('phase-negative'));
});
