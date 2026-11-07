---
title: Test your application
description: Test code that dispatches into a holo with a mock host, a clock that skips time, scripted dispatch and recorded histories.
available_in: "0.6"
---

<Note>This page describes Prism 0.6, which has not shipped. [Release status](/reference/release-status) says what works today.</Note>

This guide shows you how to test the code around a holo without a network, a real clock or a running guest: a mock host, a clock that skips time, a dispatch that follows a script, and histories you record and replay.

`@prism-code/testing` is experimental. Pin the exact version if you use it. It takes `prism-code` as a peer dependency and uses its history reader, writer and recorder, so a history it writes is the file `prism-code replay` reads.

```bash
npm install --save-dev @prism-code/testing prism-code
```

## A mock host

A guest that targets the core-wasm world imports nothing, so the host is what surrounds a dispatch: the time it happens at, the random numbers your code draws, and the resources a capability file grants. `createMockHost` makes all three deterministic.

- `clock` starts at 2026-01-01T00:00:00Z unless you pass `start`, and moves only when you move it.
- `random` is seeded, so the same `seed` gives the same numbers on every platform.
- `fetch`, `kv` and `variable` check the same grant `load` enforces. With no capability file nothing is granted, so a missing grant fails in a test the way it fails in production. A granted URL with no stub in `fetch` is an `UnexpectedCallError`.
- `calls` lists every host call with the clock time it happened at.

```js
import { createMockHost } from '@prism-code/testing';

// Nothing is granted unless the capability file says so, exactly as in load().
const host = createMockHost({
  start: '2026-03-01T09:00:00Z',
  seed: 'payments',
  capabilities: {
    schema: 'prism/capabilities/1',
    sets: { default: { 'allow-outbound-hosts': ['https://rates.example.com'], 'allow-variables': ['region'] } },
  },
  variables: { region: 'eu' },
  fetch: { 'https://rates.example.com/usd': '1.08' },
});

// Sleeping takes no real time: until() moves the clock to each timer the work is waiting on.
const rate = await host.clock.until(async () => {
  await host.clock.sleep(60_000);
  return (await host.fetch('https://rates.example.com/usd')).text();
});
console.log(rate, 'at', host.clock.date().toISOString());
console.log(host.variable('region'), host.random.uuid());

// A call the grant does not allow is the same error the runtime raises.
try {
  host.kv('sessions');
} catch (error) {
  console.log(error.name, error.check);
}
console.log(host.calls.map((call) => `${call.kind} ${call.resource}`));
```

```text
1.08 at 2026-03-01T09:01:00.000Z
eu ca79c15d-a612-45fc-8e9e-434f32d2c52b
CapabilityDeniedError capabilities.key-value-stores
[
  'fetch https://rates.example.com/usd',
  'variable region',
  'kv sessions'
]
```

`until(work)` is the time-skipping call. Each time the work is waiting and nothing else can proceed, it moves the clock to the next timer, so a minute of sleeping takes no real time. It rejects with `TestSetupError` when the work is still waiting and no timer is left, which means something other than the clock is awaited. `advance(ms)`, `next()` and `runAll()` give finer control: `advance` fires the timers that fall due, in order, with the clock at each one's time.

`host.load(source)` opens a holo with the host's capability file and set, from bytes or a path, so what the test grants is what `app.capabilities` reports.

## Scripted dispatch

`scriptedDispatch(script)` stands in for `app.dispatch`. The script lists the calls you expect, in order, each with a response or an error. A request is text or bytes, or a function of the received bytes that returns true. A response is text, bytes, or a function of the received bytes. An error is an `Error` to throw or a stable error code, thrown as a `PrismCodeError` the way a failed dispatch is.

A call that is not the next one listed throws `UnexpectedCallError`, naming the index and both requests, and leaves the script where it was. `assertComplete()` throws `ScriptIncompleteError` while listed calls have not happened, so a test that forgets a call fails.

```js
import { scriptedDispatch } from '@prism-code/testing';

const dispatch = scriptedDispatch([
  { request: 'add 1 2', response: 'ok 3' },
  { request: 'add 2 2', error: 'core.trap' },
]);

console.log(new TextDecoder().decode(await dispatch('add 1 2')));

// A request the script does not list next is an UnexpectedCallError, not a silent wrong answer.
try {
  await dispatch('add 9 9');
} catch (error) {
  console.log(error.name, error.code, error.message);
}

// One listed call has not happened, so the test is not finished.
try {
  dispatch.assertComplete();
} catch (error) {
  console.log(error.name, error.code, error.details);
}
```

```text
ok 3
UnexpectedCallError unexpected_call dispatch 1 was not expected: the script lists request "add 2 2"; received "add 9 9"
ScriptIncompleteError script_incomplete { remaining: 1, next: 'request "add 2 2"' }
```

A `prism/history/1` is also a script: `scriptedDispatch(history)` answers the recorded requests with the recorded responses and refusals.

## Histories

A history is the dispatches of one application in order with their recorded outcomes. The package reads and writes the format with prism-code's own code.

| Call | What it does |
| --- | --- |
| `readHistory(path)` | Reads and validates a history file. A file the schema rejects is a `ReplayError`. |
| `writeHistory(path, history)` | Writes canonical JSON and one line feed, the bytes `prism-code history` writes. |
| `recordHistory(app, run)` | Passes `run` a `dispatch` that records every outcome, refusals included, and resolves to the history. |
| `assertReplays(app, histories)` | Replays against the build and rejects with `DeterminismViolationError` on a divergence. |
| `referenceArchive(name)` | Calculator or Payment Approval with its committed history. |

```js
import { assertReplays, recordHistory, referenceArchive, scriptedDispatch } from '@prism-code/testing';

// Calculator and its committed history ship with prism-code.
const calculator = await referenceArchive('Calculator');
const app = await calculator.load();

// Record what a test does against the app, then replay it against the same build.
const [first] = app.vectors;
const history = await recordHistory(app, async (dispatch) => {
  await dispatch(first.request);
});
console.log(history.schema, history.events.length, 'event');
console.log((await assertReplays(app, history)).divergenceCount, 'divergences');

// The committed history also drives a scripted dispatch, so code that calls dispatch runs without the guest.
const scripted = scriptedDispatch(await calculator.history());
console.log(scripted.remaining(), 'scripted calls');
```

```text
prism/history/1 1 event
0 divergences
22 scripted calls
```

Payment Approval is in the repository's fixtures and not in the npm package, so `referenceArchive('PaymentApproval')` works in a checkout and `referenceArchives()` lists what your install has. Pass `{ directory }` to read fixtures from somewhere else.

Commit a recorded history next to your holo and replay it in CI. A change to the model that alters an answer then fails the test, which is what the replay gate in [Rebuild and update](/guides/rebuild-and-update) does on the command line.

## Next steps

- [Run and gate](/guides/run-and-gate)
- [Histories and replay](/api/history-and-replay)
- [Error types](/api/errors)
