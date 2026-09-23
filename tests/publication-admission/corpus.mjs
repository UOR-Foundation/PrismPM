// Independent positional expectations. These are conditional model inputs,
// never forged cryptographic/provider receipts used for release acceptance.
import assert from 'node:assert/strict';
export const clone = value => structuredClone(value);
export const bytes = (size=32, value=1) => new Uint8Array(size).fill(value);
const fields = specification => specification.split(' ').map(field => field.split(':'));
export const records = {
  Target: fields('url:s publisher:s environment:s adapter:b'),
  Obligation: fields('id:n moment:e assurance:e authority:b scope:b'),
  Declaration: fields('stage:s policy:b target:Target clock:b clockAuthority:b trustAuthority:b decisionAuthority:b refAuthority:b deploymentAuthority:b integrityAuthority:b minimumTrust:e refKind:e obligations:Obligation[]'),
  Subject: fields('producer:s source:b release:b model:b build:b services:b controls:b dependencies:b sdk:b compiler:b runtime:b oracles:b tree:b'),
  Context: fields('declaration:Declaration declarationIdentity:b subject:Subject instance:b publisherRevision:b publisherRef:s digest:b'),
  Clock: fields('domain:b authority:b receipt:b tick:n'),
  Fact: fields('context:b obligation:n assurance:e authority:b scope:b evidence:b deployment:b? from:n until:n outcome:e'),
  Trust: fields('context:b authority:b receipt:b status:e from:n until:n'),
  Decision: fields('context:b authority:b refAuthority:b decision:b refEvidence:b ready:b publisherRevision:b publisherRef:s refKind:e from:n until:n outcome:e'),
  Deployment: fields('context:b decision:b authority:b receipt:b publisherRevision:b deploymentRevision:b deploymentId:s observed:n from:n until:n'),
  Integrity: fields('context:b authority:b receipt:b release:b model:b build:b tree:b url:s deployment:b observed:n from:n until:n'),
  State: fields('context:Context revision:n phase:e clock:Clock readyAt:n authorizedAt:n observedAt:n trust:Trust? readiness:Fact[] ready:b? decision:Decision? deployment:Deployment? integrity:Integrity? live:Fact[]'),
};
export function positional(type, value) {
  if (type.endsWith('[]')) return value.map(item => positional(type.slice(0,-2),item));
  if (type.endsWith('?')) return value === null ? [0] : [1,positional(type.slice(0,-1),value)];
  if (type === 'e') return [value];
  if (['s','b','n'].includes(type)) return value;
  assert.ok(records[type],type);
  return records[type].map(([field,kind]) => positional(kind,value[field]));
}
export function encode(value) {
  const head = (major,size) => {
    assert.ok(Number.isSafeInteger(size)&&size>=0&&size<=0xffffffff);
    if(size<24)return Buffer.from([major*32+size]);
    if(size<256)return Buffer.from([major*32+24,size]);
    const result=Buffer.alloc(size<65536?3:5);result[0]=major*32+(size<65536?25:26);
    if(size<65536)result.writeUInt16BE(size,1);else result.writeUInt32BE(size,1);return result;
  };
  if(typeof value==='number')return head(0,value);
  if(typeof value==='string'){const data=Buffer.from(value);return Buffer.concat([head(3,data.length),data]);}
  if(value instanceof Uint8Array)return Buffer.concat([head(2,value.length),value]);
  assert.ok(Array.isArray(value));return Buffer.concat([head(4,value.length),...value.map(encode)]);
}
const successful=state=>[1,0,positional('State',state)], rejected=error=>[1,1,[error]], malformed=error=>[1,2,error];
export const request = {
  initialize:(context,clock)=>[1,0,positional('Context',context),positional('Clock',clock)],
  transition:(context,state,clock,operation,revision=state.revision)=>[1,1,positional('Context',context),positional('State',state),revision,positional('Clock',clock),operation],
  inspect:(context,state,clock)=>[1,2,positional('Context',context),positional('State',state),positional('Clock',clock)],
  declaration:declaration=>[1,3,positional('Declaration',declaration)],
  context:context=>[1,4,positional('Context',context)],
};
export const declarationPreimage=declaration=>Buffer.concat([Buffer.from('prismpm/publication-declaration/1\0'),encode(positional('Declaration',declaration))]);
export const contextPreimage=context=>Buffer.concat([Buffer.from('prismpm/publication-context/1\0'),encode(positional('Context',context).slice(0,-1))]);
export function fixture(count=4,preCount=null) {
  const obligation=id=>{const moment=preCount===null?(id%2===1?0:1):(id<=preCount?0:1);return {id,moment,assurance:moment===0?1:6,authority:bytes(32,20+id%200),scope:bytes(32,80+id%100)};};
  const declaration={stage:'functional-core',policy:bytes(32,2),target:{url:'https://uor-foundation.github.io/foundry-web/',publisher:'https://github.com/UOR-Foundation/foundry-web',environment:'github-pages',adapter:bytes(32,3)},
    clock:bytes(32,4),clockAuthority:bytes(32,5),trustAuthority:bytes(32,6),decisionAuthority:bytes(32,7),refAuthority:bytes(32,8),deploymentAuthority:bytes(32,9),integrityAuthority:bytes(32,10),minimumTrust:0,refKind:0,obligations:Array.from({length:count},(_,index)=>obligation(index+1))};
  const subject={producer:'https://github.com/UOR-Foundation/uor-foundry',source:bytes(20,11),...Object.fromEntries('release model build services controls dependencies sdk compiler runtime oracles tree'.split(' ').map((field,index)=>[field,bytes(32,index+30)]))};
  const context={declaration,declarationIdentity:bytes(32,12),subject,instance:bytes(32,13),publisherRevision:bytes(20,14),publisherRef:'refs/heads/main',digest:bytes(32,15)};
  const clock=tick=>({domain:declaration.clock,authority:declaration.clockAuthority,receipt:bytes(32,16),tick});
  const ready=bytes(32,17),trust={context:context.digest,authority:declaration.trustAuthority,receipt:bytes(32,18),status:0,from:90,until:1000};
  const decision={context:context.digest,authority:declaration.decisionAuthority,refAuthority:declaration.refAuthority,decision:bytes(32,19),refEvidence:bytes(32,20),ready,publisherRevision:context.publisherRevision,publisherRef:context.publisherRef,refKind:0,from:90,until:1000,outcome:0};
  const deployment={context:context.digest,decision:decision.decision,authority:declaration.deploymentAuthority,receipt:bytes(32,21),publisherRevision:context.publisherRevision,deploymentRevision:bytes(20,22),deploymentId:'123456',observed:103,from:90,until:1000};
  const integrity={context:context.digest,authority:declaration.integrityAuthority,receipt:bytes(32,23),...Object.fromEntries(['release','model','build','tree'].map(field=>[field,subject[field]])),url:declaration.target.url,deployment:deployment.receipt,observed:103,from:90,until:1000};
  const facts=moment=>declaration.obligations.filter(row=>row.moment===moment).map(row=>({context:context.digest,obligation:row.id,assurance:row.assurance,authority:row.authority,scope:row.scope,evidence:bytes(32,row.id%255),deployment:moment===0?null:deployment.receipt,from:90,until:1000,outcome:0}));
  const readiness=facts(0),live=facts(1);
  const initial={context,revision:0,phase:0,clock:clock(100),readyAt:0,authorizedAt:0,observedAt:0,trust:null,readiness:[],ready:null,decision:null,deployment:null,integrity:null,live:[]};
  const prepared={...initial,revision:1,phase:1,clock:clock(101),readyAt:101,trust,readiness,ready};
  const authorized={...prepared,revision:2,phase:2,clock:clock(102),authorizedAt:102,decision};
  const observed={...authorized,revision:3,clock:clock(103),observedAt:103,deployment,integrity};
  const accepted={...observed,revision:4,phase:3,clock:clock(104),live};
  const prepare=[0,positional('Trust',trust),readiness.map(row=>positional('Fact',row)),ready];
  const authorize=[1,positional('Decision',decision)],observe=[2,positional('Deployment',deployment),positional('Integrity',integrity)],accept=[3,live.map(row=>positional('Fact',row))];
  return {context,declaration,subject,clock,trust,decision,deployment,integrity,ready,readiness,live,initial,prepared,authorized,observed,accepted,prepare,authorize,observe,accept};
}
function atTick(f,tick) {
  for(const fact of[f.trust,f.decision,f.deployment,f.integrity,...f.readiness,...f.live]){fact.from=tick;fact.until=tick;}
  f.deployment.observed=tick;f.integrity.observed=tick;
  for(const state of[f.initial,f.prepared,f.authorized,f.observed,f.accepted]) {
    state.clock=f.clock(tick);
    if(state.revision>=1)state.readyAt=tick;
    if(state.revision>=2)state.authorizedAt=tick;
    if(state.revision>=3)state.observedAt=tick;
  }
  f.prepare=[0,positional('Trust',f.trust),f.readiness.map(row=>positional('Fact',row)),f.ready];
  f.authorize=[1,positional('Decision',f.decision)];
  f.observe=[2,positional('Deployment',f.deployment),positional('Integrity',f.integrity)];
  f.accept=[3,f.live.map(row=>positional('Fact',row))];
  return f;
}
export function corpus() {
  const rows=[],f=fixture();
  const add=(id,input,output)=>rows.push({id,request:encode(input),response:encode(output)});
  const raw=(id,input,error)=>rows.push({id,request:Uint8Array.from(input),response:encode(malformed(error))});
  const projected=context=>[1,3,contextPreimage(context)];
  add('DeclarationPreimage',request.declaration(f.declaration),[1,3,declarationPreimage(f.declaration)]);
  add('ContextPreimage',request.context(f.context),projected(f.context));
  {const x=clone(f.context);x.digest=bytes(32,255);add('ContextOwnDigestExcluded',request.context(x),projected(f.context));}
  const leaves=(value,path=[])=>value instanceof Uint8Array||typeof value!=='object'?[[path,value]]:Object.entries(value).flatMap(([key,item])=>leaves(item,[...path,key]));
  const baseline=fixture().context;
  baseline.declaration.obligations.forEach((row,index)=>{row.id=(index+1)*10;row.assurance=5;});
  for(const [path,value]of leaves(baseline)) {
    if(path[0]==='digest')continue;
    const x=clone(baseline),parent=path.slice(0,-1).reduce((value,key)=>value[key],x),field=path.at(-1);
    parent[field]=value instanceof Uint8Array?bytes(value.length,255):typeof value==='string'?value+'-changed':field==='id'?value+1:field==='assurance'?0:1-value;
    // Post-publication assurance remains one of its exact allowed kinds.
    if(field==='assurance'&&parent.moment===1)parent[field]=6;
    const expected=contextPreimage(x);assert.ok(!expected.equals(contextPreimage(baseline)));
    add('ContextPreimageField'+path.join(''),request.context(x),[1,3,expected]);
    if(path[0]==='declaration')add('DeclarationPreimageField'+path.slice(1).join(''),request.declaration(x.declaration),[1,3,declarationPreimage(x.declaration)]);
  }
  {const x=clone(f.declaration);x.stage='';add('InvalidDeclarationPreimage',request.declaration(x),rejected(0));}
  {const x=clone(f.context);x.publisherRef='';add('InvalidContextPreimage',request.context(x),rejected(2));}
  add('Initialize',request.initialize(f.context,f.clock(100)),successful(f.initial));
  for(const [id,state,next,operation]of[['Prepare',f.initial,f.prepared,f.prepare],['Authorize',f.prepared,f.authorized,f.authorize],['Observe',f.authorized,f.observed,f.observe],['Accept',f.observed,f.accepted,f.accept]]) {
    add(id,request.transition(f.context,state,next.clock,operation),successful(next));
    add(id+'Replay',request.transition(f.context,next,next.clock,operation),rejected(8));
    add(id+'StaleRevision',request.transition(f.context,state,next.clock,operation,state.revision+1),rejected(6));
  }
  for(const state of[f.initial,f.prepared,f.authorized,f.observed,f.accepted]) {
    add('Inspect'+state.revision,request.inspect(f.context,state,f.clock(110)),successful(state));
    add('ClockRollback'+state.revision,request.inspect(f.context,state,f.clock(99)),rejected(7));
    if(state.revision)add('ExpiredRetained'+state.revision,request.inspect(f.context,state,f.clock(1001)),rejected(4));
    if(state.revision){const altered=clone(state);altered.clock.tick=89;add('EvidenceAfterRecordedState'+state.revision,request.inspect(f.context,altered,f.clock(110)),rejected(4));}
  }
  const wrong=(id,operation,error)=>add(id,request.transition(f.context,f.initial,f.clock(101),operation),rejected(error));
  wrong('NoReadinessAuthorize',f.authorize,8);wrong('NoAuthorizationObserve',f.observe,8);wrong('NoDeploymentAccept',f.accept,8);
  for(const [field,value]of[['stage',''],['policy',bytes(31)],['clock',bytes(31)],['clockAuthority',bytes(31)],['trustAuthority',bytes(31)],['decisionAuthority',bytes(31)],['refAuthority',bytes(31)],['deploymentAuthority',bytes(31)],['integrityAuthority',bytes(31)],['minimumTrust',2],['minimumTrust',3],['obligations',[]]]) {
    const x=clone(f.context);x.declaration[field]=value;add('InvalidDeclaration'+field+rows.length,request.initialize(x,f.clock(100)),rejected(0));
  }
  for(const field of['url','publisher','environment','adapter']){const x=clone(f.context);x.declaration.target[field]=field==='adapter'?bytes(31):'';add('InvalidTarget'+field,request.initialize(x,f.clock(100)),rejected(0));}
  for(const field of Object.keys(f.subject)){const x=clone(f.context);x.subject[field]=field==='producer'?'':bytes(field==='source'?19:31);add('InvalidSubject'+field,request.initialize(x,f.clock(100)),rejected(1));}
  for(const field of['declarationIdentity','instance','digest','publisherRevision','publisherRef']){const x=clone(f.context);x[field]=field==='publisherRef'?'':bytes(field==='publisherRevision'?19:31);add('InvalidContext'+field,request.initialize(x,f.clock(100)),rejected(2));}
  for(const [id,change]of[
    ['Duplicate',rows=>{rows[1].id=rows[0].id;}],['Order',rows=>rows.reverse()],['Zero',rows=>{rows[0].id=0;}],
    ['OnlyPre',rows=>rows.forEach(row=>{row.moment=0;row.assurance=1;})],['OnlyPost',rows=>rows.forEach(row=>{row.moment=1;row.assurance=6;})],
    ['PrematureLive',rows=>{rows[0].assurance=6;}],['DeferredProof',rows=>{rows[1].assurance=0;}],
  ]){const x=clone(f.context);change(x.declaration.obligations);add('Obligation'+id,request.initialize(x,f.clock(100)),rejected(0));}
  for(const field of Object.keys(f.subject)){const x=clone(f.context);x.subject[field]=field==='producer'?f.subject.producer+'-other':bytes(field==='source'?20:32,255);add('ContextChangedSubject'+field,request.transition(x,f.initial,f.clock(101),f.prepare),rejected(5));}
  for(const field of['declarationIdentity','instance','digest','publisherRevision','publisherRef']){const x=clone(f.context);x[field]=field==='publisherRef'?'refs/heads/other':bytes(field==='publisherRevision'?20:32,255);add('ContextChanged'+field,request.transition(x,f.initial,f.clock(101),f.prepare),rejected(5));}
  for(const field of['stage','policy','clock','clockAuthority','trustAuthority','decisionAuthority','refAuthority','deploymentAuthority','integrityAuthority','minimumTrust','refKind']){const x=clone(f.context);x.declaration[field]=field==='stage'?'other-stage':['minimumTrust','refKind'].includes(field)?1:bytes(32,255);add('ContextChangedDeclaration'+field,request.transition(x,f.initial,f.clock(101),f.prepare),rejected(5));}
  const contextChanged=clone(f.context);contextChanged.declaration.obligations[0].scope=bytes(32,255);add('ContextChangedObligation',request.transition(contextChanged,f.initial,f.clock(101),f.prepare),rejected(5));
  for(const field of['url','publisher','environment','adapter']){const x=clone(f.context);x.declaration.target[field]=field==='adapter'?bytes(32,255):x.declaration.target[field]+'other';add('ContextChangedTarget'+field,request.transition(x,f.initial,f.clock(101),f.prepare),rejected(5));}
  for(const [id,minimum,status]of[['AcceptedMinimum',1,1],['AcceptedStronger',0,1]]){const x=fixture();x.declaration.minimumTrust=minimum;x.trust.status=status;x.prepare[1]=positional('Trust',x.trust);add(id,request.transition(x.context,x.initial,x.clock(101),x.prepare),successful(x.prepared));}
  {const x=fixture();x.declaration.minimumTrust=1;add('CandidateBelowRequiredTrust',request.transition(x.context,x.initial,x.clock(101),x.prepare),rejected(9));}
  for(let assurance=0;assurance<8;assurance++){
    const x=fixture(),post=assurance>5,index=post?1:0;x.declaration.obligations[index].assurance=assurance;
    const selected=post?x.live:x.readiness;selected[0].assurance=assurance;
    if(post){x.accept[1]=x.live.map(row=>positional('Fact',row));add('AssuranceKind'+assurance,request.transition(x.context,x.observed,x.clock(104),x.accept),successful(x.accepted));}
    else{x.prepare[2]=x.readiness.map(row=>positional('Fact',row));add('AssuranceKind'+assurance,request.transition(x.context,x.initial,x.clock(101),x.prepare),successful(x.prepared));}
  }
  for(const [field,value]of[['domain',bytes(32,255)],['authority',bytes(32,255)],['receipt',bytes(31)]]){const c=f.clock(101);c[field]=value;add('BadClock'+field,request.initialize(f.context,c),rejected(3));}
  for(const [field,value]of[['context',bytes(32,255)],['authority',bytes(32,255)],['receipt',bytes(31)],['status',2],['status',3],['from',102],['until',100]]){const t=clone(f.trust);t[field]=value;add('BadTrust'+field+rows.length,request.transition(f.context,f.initial,f.clock(101),[0,positional('Trust',t),f.readiness.map(x=>positional('Fact',x)),f.ready]),rejected(9));}
  for(const [name,input,state,error]of[['Pre',f.readiness,f.initial,10],['Post',f.live,f.observed,14]]) {
    const operation=facts=>name==='Pre'?[0,positional('Trust',f.trust),facts.map(x=>positional('Fact',x)),f.ready]:[3,facts.map(x=>positional('Fact',x))];
    for(const [id,change]of[['Missing',rows=>rows.pop()],['Extra',rows=>rows.push(clone(rows[0]))],['Order',rows=>rows.reverse()],['Duplicate',rows=>{rows[1]=clone(rows[0]);}]]){const facts=clone(input);change(facts);add('Coverage'+name+id,request.transition(f.context,state,f.clock(name==='Pre'?101:104),operation(facts)),rejected(error));}
    for(const [field,value]of[['context',bytes(32,255)],['obligation',999],['assurance',5],['authority',bytes(32,255)],['scope',bytes(32,255)],['evidence',bytes(31)],['deployment',name==='Pre'?bytes(32):null],['from',105],['until',100],['outcome',1],['outcome',2]]) {
      const facts=clone(input);facts[0][field]=value;add('Fact'+name+field+rows.length,request.transition(f.context,state,f.clock(name==='Pre'?101:104),operation(facts)),rejected(error));
    }
  }
  for(const moment of[0,1])for(const index of[63,64,65,2047]){
    const x=fixture(4096),facts=clone(moment===0?x.readiness:x.live);facts[index].scope=bytes(32,255);
    const operation=moment===0?[0,positional('Trust',x.trust),facts.map(row=>positional('Fact',row)),x.ready]:[3,facts.map(row=>positional('Fact',row))];
    add('FactPartition'+moment+'Index'+index,request.transition(x.context,moment===0?x.initial:x.observed,x.clock(moment===0?101:104),operation),rejected(moment===0?10:14));
  }
  for(const [field,value]of[['context',bytes(32,255)],['authority',bytes(32,255)],['refAuthority',bytes(32,255)],['decision',bytes(31)],['refEvidence',bytes(31)],['ready',bytes(32,255)],['publisherRevision',bytes(20,255)],['publisherRef','refs/heads/other'],['refKind',1],['from',103],['until',101],['outcome',1],['outcome',2]]){const d=clone(f.decision);d[field]=value;add('Decision'+field+rows.length,request.transition(f.context,f.prepared,f.clock(102),[1,positional('Decision',d)]),rejected(11));}
  for(const [field,value]of[['context',bytes(32,255)],['decision',bytes(32,255)],['authority',bytes(32,255)],['receipt',bytes(31)],['publisherRevision',bytes(20,255)],['deploymentRevision',bytes(19)],['deploymentId',''],['observed',104],['from',104],['until',102]]){const d=clone(f.deployment);d[field]=value;add('Deployment'+field,request.transition(f.context,f.authorized,f.clock(103),[2,positional('Deployment',d),positional('Integrity',f.integrity)]),rejected(12));}
  {const d=clone(f.deployment);d.observed=101;add('DeploymentBeforeAuthorization',request.transition(f.context,f.authorized,f.clock(103),[2,positional('Deployment',d),positional('Integrity',f.integrity)]),rejected(12));}
  {const d=clone(f.deployment);d.observed=1;d.from=0;const i=clone(f.integrity);i.observed=2;i.from=0;add('DeploymentPredatesDecisionValidity',request.transition(f.context,f.authorized,f.clock(103),[2,positional('Deployment',d),positional('Integrity',i)]),rejected(12));}
  for(const [field,value]of[['readyAt',105],['authorizedAt',100],['observedAt',102]]){const state=clone(f.accepted);state[field]=value;add('ForgedTimeline'+field,request.inspect(f.context,state,f.clock(110)),rejected(4));}
  for(const [field,value]of[['context',bytes(32,255)],['authority',bytes(32,255)],['receipt',bytes(31)],['release',bytes(32,255)],['model',bytes(32,255)],['build',bytes(32,255)],['tree',bytes(32,255)],['url','https://uor-foundation.github.io/other/'],['deployment',bytes(32,255)],['observed',102],['observed',104],['from',104],['until',102]]){const i=clone(f.integrity);i[field]=value;add('Integrity'+field+rows.length,request.transition(f.context,f.authorized,f.clock(103),[2,positional('Deployment',f.deployment),positional('Integrity',i)]),rejected(13));}
  for(const field of['trust','ready','decision','deployment','integrity']){const state=clone(f.accepted);state[field]=null;add('ForgedAccepted'+field,request.inspect(f.context,state,f.clock(104)),rejected(4));}
  for(const tick of[0,0xffffffff]) {
    const x=atTick(fixture(),tick);
    for(const [name,state,next,operation]of[['Prepare',x.initial,x.prepared,x.prepare],['Authorize',x.prepared,x.authorized,x.authorize],['Observe',x.authorized,x.observed,x.observe],['Accept',x.observed,x.accepted,x.accept]])
      add('ExactValidity'+tick+name,request.transition(x.context,state,x.clock(tick),operation),successful(next));
    add('ExactValidity'+tick+'Inspect',request.inspect(x.context,x.accepted,x.clock(tick)),successful(x.accepted));
  }
  for(const count of[63,64,65,127,128,129,4095]) {
    const x=fixture(count);
    for(const [name,state,next,operation]of[['Prepare',x.initial,x.prepared,x.prepare],['Authorize',x.prepared,x.authorized,x.authorize],['Observe',x.authorized,x.observed,x.observe],['Accept',x.observed,x.accepted,x.accept]])
      add('FlatBoundary'+count+name,request.transition(x.context,state,next.clock,operation),successful(next));
    add('FlatBoundary'+count+'Declaration',request.declaration(x.declaration),[1,3,declarationPreimage(x.declaration)]);
    add('FlatBoundary'+count+'Context',request.context(x.context),[1,3,contextPreimage(x.context)]);
  }
  const valid=encode(request.initialize(f.context,f.clock(100)));
  const invalidText=Buffer.from(valid);invalidText[invalidText.indexOf(Buffer.from('functional-core'))]=0xff;raw('InvalidUtf8',invalidText,7);
  raw('Trailing',[...valid,0],8);raw('EmptyWire',[],2);raw('Indefinite',[0x9f,1,0,0xff],4);raw('WrongRoot',[0],3);raw('NonminimalRoot',[0x98,4,...valid.slice(1)],5);
  for(const size of[1,2,3,4,valid.length-1])raw('Truncated'+size,valid.slice(0,size),2);
  add('WrongVersion',[2,0,positional('Context',f.context),positional('Clock',f.clock(100))],malformed(3));
  add('UnknownOpcode',[1,5],malformed(3));
  for(const [type,path]of[['phase',['phase']],['outcome',['decision','outcome']]]){const state=clone(f.accepted);if(type==='phase')state.phase=4;else state.decision.outcome=3;add('Unknown'+type,request.inspect(f.context,state,f.clock(104)),malformed(3));}
  assert.equal(new Set(rows.map(row=>row.id)).size,rows.length);return rows;
}
export function maximumCorpus() {
  const f=fixture(4096),rows=[];
  const add=(id,input,output)=>rows.push({id,request:encode(input),response:encode(output)});
  add('MaximumObligationReadiness',request.transition(f.context,f.initial,f.clock(101),f.prepare),successful(f.prepared));
  add('MaximumObligationAcceptance',request.transition(f.context,f.observed,f.clock(104),f.accept),successful(f.accepted));
  add('MaximumObligationInspection',request.inspect(f.context,f.accepted,f.clock(104)),successful(f.accepted));
  add('MaximumDeclarationPreimage',request.declaration(f.declaration),[1,3,declarationPreimage(f.declaration)]);
  add('MaximumContextPreimage',request.context(f.context),[1,3,contextPreimage(f.context)]);
  const combined=fixture(4096);
  function widen(value){if(!value||typeof value!=='object'||value instanceof Uint8Array)return;for(const key of Object.keys(value)){if(typeof value[key]==='string')value[key]='x'.repeat(2048);else if(!Array.isArray(value[key]))widen(value[key]);}}
  widen(combined.context);combined.decision.publisherRef=combined.context.publisherRef;combined.deployment.deploymentId='d'.repeat(2048);combined.integrity.url=combined.declaration.target.url;
  const all=[combined.trust,combined.decision,combined.deployment,combined.integrity,...combined.readiness,...combined.live];
  for(const fact of all)fact.until=0xffffffff;
  combined.observed.clock=combined.clock(0xffffffff);combined.observed.observedAt=0xffffffff;combined.accepted.observedAt=0xffffffff;combined.accepted.clock=combined.clock(0xffffffff);
  const finalOperation=[3,combined.live.map(row=>positional('Fact',row))];
  add('CombinedStructuralMaximum',request.transition(combined.context,combined.observed,combined.clock(0xffffffff),finalOperation),successful(combined.accepted));
  add('CombinedDeclarationPreimage',request.declaration(combined.declaration),[1,3,declarationPreimage(combined.declaration)]);
  add('CombinedContextPreimage',request.context(combined.context),[1,3,contextPreimage(combined.context)]);
  for(const preCount of[1,4095]) {
    const x=fixture(4096,preCount);widen(x.context);x.decision.publisherRef=x.context.publisherRef;x.deployment.deploymentId='d'.repeat(2048);x.integrity.url=x.declaration.target.url;atTick(x,0xffffffff);
    add('CombinedPartition'+preCount+'Readiness',request.transition(x.context,x.initial,x.clock(0xffffffff),x.prepare),successful(x.prepared));
    add('CombinedPartition'+preCount+'Acceptance',request.transition(x.context,x.observed,x.clock(0xffffffff),x.accept),successful(x.accepted));
    add('CombinedPartition'+preCount+'Inspection',request.inspect(x.context,x.accepted,x.clock(0xffffffff)),successful(x.accepted));
  }
  const malformedState=clone(combined.accepted);
  malformedState.readiness=Array.from({length:4096},()=>clone(combined.readiness[0]));
  malformedState.live=Array.from({length:4096},()=>clone(combined.live[0]));
  const incoming=Array.from({length:4096},()=>positional('Fact',combined.live[0]));
  add('MaximumMalformedFactClosure',request.transition(combined.context,malformedState,combined.clock(0xffffffff),[3,incoming]),rejected(4));
  for(const preCount of[1,4095]){
    const x=fixture(4096,preCount);widen(x.context);x.decision.publisherRef=x.context.publisherRef;x.deployment.deploymentId='d'.repeat(2048);x.integrity.url=x.declaration.target.url;atTick(x,0xffffffff);
    const state=clone(x.accepted),selected=preCount===4095?'readiness':'live';state[selected].push(clone(state[selected].at(-1)));
    if(preCount===4095)state.live=Array.from({length:4096},()=>clone(x.live[0]));
    const fullIncoming=Array.from({length:4096},()=>positional('Fact',x.live[0]));
    add('MaximumLateFactClosure'+preCount,request.transition(x.context,state,x.clock(0xffffffff),[3,fullIncoming]),rejected(4));
  }
  const extra=fixture(4097);add('OverObligations',request.initialize(extra.context,extra.clock(100)),malformed(6));
  const text=fixture();text.context.declaration.stage='x'.repeat(2048);add('MaximumText',request.initialize(text.context,text.clock(100)),successful(text.initial));
  text.context.declaration.stage+='x';add('OverText',request.initialize(text.context,text.clock(100)),malformed(6));
  text.context.declaration.stage='\u{1f419}'.repeat(512);add('MaximumUtf8Text',request.initialize(text.context,text.clock(100)),successful(text.initial));
  text.context.declaration.stage+='x';add('OverUtf8Text',request.initialize(text.context,text.clock(100)),malformed(6));
  const highest=fixture();highest.declaration.obligations[3].id=0xffffffff;add('MaximumObligationIdentity',request.initialize(highest.context,highest.clock(0xffffffff)),successful({...highest.initial,clock:highest.clock(0xffffffff)}));
  const framed=Buffer.alloc(67108864);encode(request.initialize(highest.context,highest.clock(100))).copy(framed);
  rows.push({id:'MaximumFrameTrailingRejected',request:framed,response:encode(malformed(8))});
  return rows;
}
export function overFrame(){return {id:'OverFrameRejected',request:Buffer.alloc(67108865),response:encode(malformed(6))};}
