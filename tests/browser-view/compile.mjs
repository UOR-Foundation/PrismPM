import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {copyFileSync,existsSync,mkdirSync,mkdtempSync,readFileSync,renameSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
export const draft=dirname(fileURLToPath(import.meta.url));
export const repository=resolve(draft,'../..');
export const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
function verifyPins(){
  const artifacts=readFileSync(join(repository,'model/dependencies.toml'),'utf8').split('[[dependency.artifact]]').slice(1).map(section=>{
    const text=section.split('[[dependency]]')[0];return {path:/^path = "([^"]+)"$/m.exec(text)?.[1],hash:/^sha256 = "([0-9a-f]{64})"$/m.exec(text)?.[1],tree:/^tree_root = "([^"]+)"$/m.exec(text)?.[1]};
  });
  for(const name of ['vendor/lean4-prod/lean.tar','vendor/lean4-prod/rust/MANIFEST.sha256','vendor/lexlean/MANIFEST.sha256']){
    const matches=artifacts.filter(row=>row.path===name);assert.equal(matches.length,1);const[{hash,tree}]=matches;
    const bytes=readFileSync(join(repository,name));assert.equal(sha(bytes),hash,name);if(!tree)continue;
    const seen=new Set();for(const line of bytes.toString('utf8').trimEnd().split('\n')){const row=/^([0-9a-f]{64})  ([A-Za-z0-9_./-]+)$/.exec(line);assert.ok(row);const[,digest,path]=row;assert.ok(!path.startsWith('/')&&!path.split('/').some(part=>!part||part==='.'||part==='..'));assert.ok(!seen.has(path));seen.add(path);assert.equal(sha(readFileSync(join(repository,tree,path))),digest,path);}
  }
}
function toolchainPins() {
  const rustText = readFileSync(join(repository, 'rust-toolchain.toml'), 'utf8');
  const rust = /^channel = "([0-9]+\.[0-9]+\.[0-9]+)"$/m.exec(rustText)?.[1];
  const lean = readFileSync(join(repository, 'lean-toolchain'), 'utf8').trim();
  assert.ok(rust && /^leanprover\/lean4:v[0-9]+\.[0-9]+\.[0-9]+$/.test(lean),
    'closed pinned compiler toolchains');
  const triple = {x64:'x86_64-unknown-linux-gnu',arm64:'aarch64-unknown-linux-gnu'}[process.arch];
  assert.equal(process.platform, 'linux', 'compiler harness requires the pinned Linux SDK');
  assert.ok(triple, 'supported SDK compiler architecture');
  return {rust, lean, triple};
}
function compilerEnvironment(extra) {
  const {rust, lean, triple} = toolchainPins();
  const forbidden = new Set([
    'RUSTC_BOOTSTRAP',
    'RUSTC', 'RUSTDOC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
    'RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS',
    'CARGO_BUILD_RUSTC', 'CARGO_BUILD_RUSTDOC', 'CARGO_BUILD_RUSTFLAGS',
    'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER',
    'LEAN_PATH', 'LEAN_SRC_PATH', 'LEAN_SYSROOT', 'LEAN_CC', 'LEAN_AR', 'LEAN_CXX',
    'LEAN_OPTS', 'LEAN_FLAGS', 'LAKE_HOME', 'LAKE_CONFIG',
    'NODE_OPTIONS', 'NODE_PATH', 'LD_PRELOAD', 'LD_LIBRARY_PATH', 'BASH_ENV', 'ENV',
  ]);
  for (const key of Object.keys(process.env)) {
    assert.ok(!forbidden.has(key) && !/^CARGO_PROFILE_/.test(key)
      && !/^CARGO_TARGET_.*_(?:RUSTFLAGS|LINKER|RUNNER)$/.test(key),
      'inherited compiler override refused: '+key);
  }
  if (process.env.RUSTUP_TOOLCHAIN !== undefined) assert.ok(
    [rust, rust+'-'+triple].includes(process.env.RUSTUP_TOOLCHAIN),
    'inherited compiler override refused: RUSTUP_TOOLCHAIN');
  if (process.env.ELAN_TOOLCHAIN !== undefined) assert.ok(process.env.ELAN_TOOLCHAIN === lean,
    'inherited compiler override refused: ELAN_TOOLCHAIN');
  for (const [key,value] of Object.entries(extra)) {
    assert.ok(['CARGO_TARGET_DIR','LEAN_PATH'].includes(key), 'unowned compiler override refused: '+key);
    assert.ok(typeof value === 'string' && value.startsWith('/') && resolve(value) === value
      && !/[:\r\n\0]/.test(value), 'confined compiler path required: '+key);
  }
  return {...process.env, PATH:'/usr/local/elan/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin',
    CARGO_NET_OFFLINE:'true', RUSTUP_TOOLCHAIN:rust+'-'+triple, ELAN_TOOLCHAIN:lean, ...extra};
}
function terminateOwnedGroup(pid) {
  if (!Number.isSafeInteger(pid) || pid <= 1) return;
  const signal = name => {
    try { process.kill(-pid, name); return true; }
    catch (error) { if (error.code === 'ESRCH') return false; throw error; }
  };
  if (!signal('SIGTERM')) return;
  // A successful/failed parent can leave grandchildren behind after timeout
  // itself exits. Escalate only this invocation's detached process group.
  const deadline = performance.now()+250;
  const pause = new Int32Array(new SharedArrayBuffer(4));
  while (performance.now() < deadline) {
    if (!signal(0)) return;
    Atomics.wait(pause, 0, 0, 10);
  }
  signal('SIGKILL');
}
function execute(program,args,cwd,env={}) {
  const childEnvironment = compilerEnvironment(env);
  const tools = {cargo:'/usr/local/cargo/bin/cargo',rustc:'/usr/local/cargo/bin/rustc',
    lean:'/usr/local/elan/bin/lean',lake:'/usr/local/elan/bin/lake',node:process.execPath,tar:'/usr/bin/tar'};
  const executable = tools[program] ?? program;
  assert.ok(typeof executable === 'string' && executable.startsWith('/'),
    'compiler command must be SDK-owned or an exact generated executable');
  if (Object.hasOwn(env,'LEAN_PATH')) assert.ok(executable.endsWith('/prod-export'),
    'LEAN_PATH belongs only to the exact generated exporter');
  const result = spawnSync('/usr/bin/timeout',
    ['--signal=TERM','--kill-after=5s','360s',executable,...args],
    {cwd,detached:true,encoding:'utf8',timeout:370000,killSignal:'SIGKILL',
      maxBuffer:32*1024*1024,env:childEnvironment});
  terminateOwnedGroup(result.pid);
  assert.ifError(result.error);
  assert.equal(result.status,0,program+' '+args.join(' ')+'\n'+result.stdout+'\n'+result.stderr);
  return result.stdout;
}
export function verifyCompilerTools() {
  const {rust, lean, triple} = toolchainPins();
  const verbose = command => execute(command,['--version','--verbose'],repository);
  for (const command of ['rustc','cargo']) {
    const version = verbose(command);
    assert.equal(/^release: (.+)$/m.exec(version)?.[1],rust, 'actual '+command+' release');
    assert.equal(/^host: (.+)$/m.exec(version)?.[1],triple, 'actual '+command+' architecture');
  }
  const version = lean.slice('leanprover/lean4:v'.length);
  const authorities = readFileSync(join(repository,'model/authorities.toml'),'utf8')
    .split('[[authority]]').slice(1).filter(section =>
      /^canonical_identifier = "leanprover\/lean4"$/m.test(section)
      && new RegExp('^edition = "'+version.replaceAll('.','\\.')+'"$', 'm').test(section));
  assert.equal(authorities.length,1,'one exact authoritative Lean release');
  const revision = /^revision = "([0-9a-f]{40})"$/m.exec(authorities[0])?.[1];
  assert.ok(revision,'immutable Lean source revision');
  assert.equal(execute('lean',['--version'],repository).trim(),
    'Lean (version '+version+', '+triple+', commit '+revision+', Release)');
  assert.match(execute('lake',['--version'],repository).trim(),
    new RegExp('^Lake version [0-9]+\\.[0-9]+\\.[0-9]+-src\\+'+revision.slice(0,7)
      +' \\(Lean version '+version.replaceAll('.','\\.')+'\\)$'));
}

let compilerToolsVerified = false;
export function run(program,args,cwd,env={}) {
  // Refuse overrides before even the tool-version subprocesses are executed.
  compilerEnvironment(env);
  if (!compilerToolsVerified) { verifyCompilerTools(); compilerToolsVerified = true; }
  return execute(program,args,cwd,env);
}
export function ensureProdExport(repo = repository) {
  verifyPins();
  const dir = resolve(repo, 'target/lean4-prod-export');
  const bin = join(dir, '.lake/build/bin/prod-export');
  if (existsSync(bin)) return { dir, bin };
  const tmp = resolve(repo, `target/lean4-prod-export-tmp-${process.pid}`);
  rmSync(tmp, { recursive: true, force: true });
  mkdirSync(tmp, { recursive: true });
  run('tar', ['-xf', join(repo, 'vendor/lean4-prod/lean.tar'), '-C', tmp], repo);
  run('lake', ['build', 'prod-export'], tmp);
  assert.ok(existsSync(join(tmp, '.lake/build/bin/prod-export')), 'built prod-export binary');
  try {
    mkdirSync(dirname(dir), { recursive: true });
    if (!existsSync(bin)) {
      rmSync(dir, { recursive: true, force: true });
      renameSync(tmp, dir);
    } else {
      rmSync(tmp, { recursive: true, force: true });
    }
  } catch (err) {
    if (!existsSync(bin)) throw err;
    rmSync(tmp, { recursive: true, force: true });
  }
  return { dir, bin };
}
export function prepare(mutation=null){
  assert.ok([null,"session","rows"].includes(mutation),"closed negative-only model mutation");
  verifyPins();
  const work=mkdtempSync(join(tmpdir(),'prismpm-view-'));
  let completed=false;
  try {
    const modules=['Foundation.Browser.V1.Workspace','Foundation.View.V1.Interaction','Foundation.View.Workspace.V1.Corpus','Foundation.View.Workspace.V1.Interaction','Foundation.View.Workspace.V1.Labels'];
    const sourcePath=name=>join(name.endsWith('.Labels')?draft:join(repository,'stdlib'),'src',...name.split('.'))+'.lex.tex';
    const sources=new Map(modules.map(name=>[name,readFileSync(sourcePath(name))]));
    const originalSources=new Map(sources);
    if(mutation){
      const name='Foundation.View.Workspace.V1.Interaction',text=sources.get(name).toString('utf8'),match=/\\semanticdata\{(.*)\}/.exec(text);assert.ok(match);
      const ast=JSON.parse(match[1]);
      if(mutation==='session'){const d=ast.declarations.find(d=>d.name==='interactionComplete'),node=d.body.branches[0].body.branches[0].body.scrutinee.value;assert.equal(node.left.function.name,'workspaceBytesEqual');node.left={kind:'bool',value:true};}
      else{const d=ast.declarations.find(d=>d.name==='interactionPresentation'),node=d.body.arguments[1].branches[0].body;assert.equal(node.arguments[1].function.name,'byteWindow');node.arguments[1]={kind:'bytes',hex:''};}
      const canonical=value=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])])):value;
      sources.set(name,Buffer.from(text.replace(match[0],'\\semanticdata{'+JSON.stringify(canonical(ast))+'}')));
    }
    const project=join(work,'project');
    for(const [name,bytes]of sources){const path=join(project,'src',...name.split('.'))+'.lex.tex';mkdirSync(dirname(path),{recursive:true});writeFileSync(path,bytes,{flag:'wx'});}
    for(const file of ['lexlean.toml','lakefile.toml','lean-toolchain'])copyFileSync(join(draft,file),join(project,file));
    copyFileSync(join(repository,'rust-toolchain.toml'),join(work,'rust-toolchain.toml'));
    const driverTarget=resolve(repository,'target/browser-test-drivers');
    run('cargo',['build','--locked','--offline','--manifest-path',join(draft,'driver/Cargo.toml')],repository,{CARGO_TARGET_DIR:driverTarget});
    const driver=join(driverTarget,'debug/browser-workspace-view-driver');
    run('lake',['update'],project);
    const verified=JSON.parse(run(driver,['verify',join(project,'lexlean.toml')],repository));
    assert.deepEqual(verified.modules,modules);
    const lean=join(work,'lean');mkdirSync(lean);
    for(const name of modules){const relative=join('PrismPM',...name.split('.'))+'.lean',path=join(lean,relative);mkdirSync(dirname(path),{recursive:true});copyFileSync(join(verified.root,'modules',relative),path);}
    copyFileSync(join(repository,'lean-toolchain'),join(lean,'lean-toolchain'));
    writeFileSync(join(lean,'lakefile.toml'),'name = "workspace_view_probe"\nversion = "0.1.0"\n[[lean_lib]]\nname = "PrismGenerated"\nroots = ['+modules.map(n=>'"PrismPM.'+n+'"').join(',')+']\n',{flag:'wx'});
    run('lake',['build','PrismGenerated'],lean);
    const {dir: exporter, bin: prodExport} = ensureProdExport();
    const exported=join(work,'export');
    run(prodExport,['--module','PrismPM.Foundation.View.Workspace.V1.Labels','--root','PrismPM.Foundation.View.Workspace.V1.Interaction.workspaceInteractionBytes','--root','PrismPM.Foundation.View.Workspace.V1.Interaction.workspacePresentationBytes','--root','PrismPM.Foundation.View.Workspace.V1.Labels.workspaceViewLabelsBytes','--ir-module','BrowserWorkspaceView','--out',exported],exporter,{LEAN_PATH:join(lean,'.lake/build/lib/lean')});
    const generated=join(work,'generated'),generation=JSON.parse(run(driver,['generate',join(exported,'kernel.ir'),generated,repository],repository));
    const runner=join(work,'runner');mkdirSync(join(runner,'src'),{recursive:true});copyFileSync(join(draft,'runner.rs'),join(runner,'src/main.rs'));
    writeFileSync(join(runner,'Cargo.lock'),'version = 4\n[[package]]\nname = "browser-workspace-view-core-probe"\nversion = "0.1.0"\n[[package]]\nname = "browser-workspace-view-runner"\nversion = "0.1.0"\ndependencies = ["browser-workspace-view-core-probe"]\n',{flag:'wx'});
    const nativeTarget=join(work,'native-target');
    function compileNative(standard){
      writeFileSync(join(runner,'Cargo.toml'),'[package]\nname = "browser-workspace-view-runner"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n[workspace]\n[dependencies]\nbrowser-workspace-view-core-probe = {path = "../generated", default-features = '+standard+'}\n');
      run('cargo',['build','--locked','--offline','--release','--manifest-path',join(runner,'Cargo.toml')],runner,{CARGO_TARGET_DIR:nativeTarget});return join(nativeTarget,'release/browser-workspace-view-runner');
    }
    const wasm=[];
    for(const label of ['a','b']){
      const guest=join(work,'guest-'+label);assert.deepEqual(JSON.parse(run(driver,['generate-wasm',join(exported,'kernel.ir'),guest,repository],repository)),generation);
      run('cargo',['build','--locked','--offline','--release'],guest,{CARGO_TARGET_DIR:join(guest,'target')});
      wasm.push(readFileSync(join(guest,'target/wasm32-unknown-unknown/release/browser_workspace_view_wasm_probe.wasm')));
    }
    assert.deepEqual(wasm[0],wasm[1],'two freshly generated and compiled guests');
    verifyPins();assert.equal(generation.ir_sha256,sha(readFileSync(join(exported,'kernel.ir'))));
    for(const [name,bytes]of originalSources)assert.deepEqual(bytes,readFileSync(sourcePath(name)),'source remained frozen '+name);
    completed=true;return {work,sources,verified,generation,compileNative,runner,wasmBytes:wasm[0]};
  } finally {if(!completed)process.stderr.write('Retained incomplete View diagnostic build '+work+'\n');}
}
