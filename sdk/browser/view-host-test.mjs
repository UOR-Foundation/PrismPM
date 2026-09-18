import test from 'node:test';
import {verifyView,verifyDependencies} from '../../tests/browser-view/checks.mjs';
import {inputs,verifyJourneys,verifyMutants,verifyExceptions} from '../../tests/browser-view/browser.mjs';
import {verifyDiagnostics} from '../../tests/browser-view/diagnostics.mjs';
test('DK-16 fresh modeled dependencies, actual private browser journeys and mutants',{timeout:3500000},async t=>{
  const view=await verifyView(t),dependencies=await verifyDependencies(t),build=inputs(view,dependencies);
  await t.test('real private host/DOM/identity/storage journeys and exact native transcripts',t=>verifyJourneys(t,build));
  await t.test('actual browser host mutants fail owning assertions',t=>verifyMutants(t,build));
  await t.test('synchronous and asynchronous exceptional promotion closes safely',t=>verifyExceptions(t,build));
  await t.test('every registered View host diagnostic and exact generated detail',async t=>t.diagnostic(JSON.stringify(await verifyDiagnostics(build))));
});
