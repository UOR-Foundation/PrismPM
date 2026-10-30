// The Prism studio (prism-code dev, issue #288). One page, two tabs, no
// dependency: it reads /state once and then follows /events over Server-Sent
// Events. Everything it shows is the dev process's state; it computes nothing
// about the holo itself.
'use strict';

const $ = (id) => document.getElementById(id);
const RANK = { P0: 0, P1: 1, P2: 2, P3: 3 };

function text(node, value) {
  node.textContent = value === null || value === undefined ? '-' : String(value);
}

function seconds(ms) {
  if (ms === null || ms === undefined) return '-';
  return ms < 1000 ? `${Math.round(ms)} ms` : `${(ms / 1000).toFixed(2)} s`;
}

function row(cells) {
  const tr = document.createElement('tr');
  for (const cell of cells) {
    const td = document.createElement('td');
    if (cell && typeof cell === 'object') {
      td.textContent = cell.text;
      if (cell.className) td.className = cell.className;
    } else td.textContent = cell === null || cell === undefined ? '-' : String(cell);
    tr.appendChild(td);
  }
  return tr;
}

function fill(tbody, rows) {
  tbody.replaceChildren(...rows);
}

/** Bytes as a person reads them: the text when they are UTF-8, otherwise 0x and hex. */
function displayBytes(hex) {
  if (typeof hex !== 'string') return '-';
  const bytes = new Uint8Array(hex.match(/../g)?.map((h) => parseInt(h, 16)) ?? []);
  try {
    return JSON.stringify(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  } catch {
    return `0x${hex}`;
  }
}

function displayOutcome(outcome) {
  if (!outcome) return '-';
  return 'response' in outcome ? displayBytes(outcome.response) : `error ${outcome.error}`;
}

function renderCheck(state) {
  const card = $('verdict-card');
  const v = state.verdict;
  const levels = state.levels ?? {};
  if (state.run === 0) {
    card.dataset.outcome = 'none';
    text($('outcome'), state.status === 'idle' ? 'waiting for the first run' : `${state.status}...`);
  } else if (state.build && !state.build.ok) {
    card.dataset.outcome = 'problem';
    text($('outcome'), 'BUILD FAILED');
  } else if (state.noVerdict) {
    card.dataset.outcome = 'problem';
    text($('outcome'), 'NO VERDICT');
  } else if (v) {
    card.dataset.outcome = v.passed ? 'passed' : 'failed';
    text($('outcome'), v.passed ? 'PASSED' : 'FAILED');
  } else {
    card.dataset.outcome = 'none';
    text($('outcome'), `${state.status}...`);
  }
  text($('level'), v ? `${v.level} ${levels[v.level] ?? ''}`.trim() : null);
  text($('policy'), v ? `${v.policy.name} requires ${v.policy.minimum} or above` : state.project.policy ? state.project.policy : 'default');
  text($('holo'), v ? v.holo : state.build?.holo ?? null);
  text($('elapsed'), state.timing ? seconds(state.timing.elapsedMs) : state.build && !state.build.ok ? `build failed after ${seconds(state.build.ms)}` : null);
  text(
    $('timing-detail'),
    state.timing
      ? `Run ${state.run} at ${new Date(state.timing.editAt).toLocaleTimeString()}: build ${seconds(state.timing.buildMs)}, check ${seconds(state.timing.checkMs)}${state.replay?.available && state.replay.ms !== null ? `, replay ${seconds(state.replay.ms)}` : ''}. ${state.trigger === 'start' ? 'Started by prism-code dev.' : state.changed.length ? `Changed: ${state.changed.join(', ')}.` : `Triggered by ${state.trigger}.`}`
      : '',
  );

  const buildFailed = Boolean(state.build && !state.build.ok);
  $('build-section').hidden = !buildFailed;
  if (buildFailed) {
    text($('build-problem'), `${state.build.problem}. Command: ${state.project.build}`);
    text($('build-output'), state.build.output?.trim() || '(nothing printed)');
  }

  $('no-verdict-section').hidden = !state.noVerdict;
  if (state.noVerdict) text($('no-verdict'), `${state.noVerdict.check}: ${state.noVerdict.message}`);

  const reasons = state.reasons ?? [];
  $('reasons-section').hidden = !v || reasons.length === 0;
  fill(
    $('reasons'),
    reasons.map((r) => row([{ text: r.check, className: 'mono' }, r.code ?? '-', r.title, r.withholds ?? 'none (policy rule)', r.message])),
  );

  const atP2 = Boolean(v && RANK[v.level] >= RANK.P2);
  $('limits-section').classList.toggle('limits-dim', !atP2);
  text($('limits-note'), atP2 ? 'This check trusts the signer for what it cannot know:' : `Printed with every verdict from P2 up${v ? `; this one is ${v.level}` : ''}. It trusts the signer for these:`);
  $('limits').replaceChildren(
    ...(state.limits ?? []).map((limit) => {
      const li = document.createElement('li');
      li.textContent = limit;
      return li;
    }),
  );

  const runs = state.runs ?? [];
  $('runs-section').hidden = runs.length === 0;
  fill(
    $('runs'),
    runs.map((r) =>
      row([
        r.run,
        new Date(r.at).toLocaleTimeString(),
        r.trigger === 'start' ? 'start' : r.changed ? `${r.changed} changed` : r.trigger,
        !r.built ? { text: 'build failed', className: 'bad' } : r.noVerdict ? { text: `no verdict (${r.noVerdict})`, className: 'bad' } : { text: `${r.passed ? 'PASSED' : 'FAILED'} ${r.level}`, className: r.passed ? 'ok' : 'bad' },
        { text: seconds(r.elapsedMs), className: 'num' },
        { text: seconds(r.buildMs), className: 'num' },
        { text: seconds(r.checkMs), className: 'num' },
      ]),
    ),
  );
}

function renderReplay(state) {
  const card = $('replay-card');
  const replay = state.replay;
  if (!replay || !replay.available) {
    card.dataset.outcome = 'none';
    text($('replay-outcome'), replay?.reason ?? 'replay not available in this build');
    text($('replay-note'), `The Replay tab lists the histories in ${replay?.histories ?? 'histories'}/ and their replay result once this build of prism-code has replay.`);
    $('histories-section').hidden = true;
    $('divergences-section').hidden = true;
    return;
  }
  if (replay.error) {
    card.dataset.outcome = 'problem';
    text($('replay-outcome'), 'COULD NOT REPLAY');
    text($('replay-note'), `${replay.error.check}: ${replay.error.message}`);
  } else if (!replay.result) {
    card.dataset.outcome = 'none';
    text($('replay-outcome'), state.run === 0 ? 'waiting for the first run' : 'no history');
    text($('replay-note'), replay.note ?? '');
  } else {
    const count = replay.result.divergenceCount;
    card.dataset.outcome = count === 0 ? 'passed' : 'failed';
    text($('replay-outcome'), count === 0 ? 'PASS  no divergence' : `FAIL  ${count} divergence${count === 1 ? '' : 's'}`);
    text($('replay-note'), `${replay.result.events} events in ${replay.result.histories.length} histor${replay.result.histories.length === 1 ? 'y' : 'ies'} against ${replay.result.application ?? 'unnamed'}, replayed by ${replay.result.runtime.name} ${replay.result.runtime.version} in ${seconds(replay.ms)}.`);
  }
  const histories = replay.result?.histories ?? [];
  $('histories-section').hidden = histories.length === 0;
  fill(
    $('histories'),
    histories.map((h, i) => row([{ text: replay.files[i] ?? `#${i}`, className: 'mono' }, h.application ?? '-', { text: h.events, className: 'num' }, { text: h.divergences.length, className: h.divergences.length ? 'bad num' : 'ok num' }])),
  );
  const divergences = histories.flatMap((h, i) => h.divergences.map((d) => ({ file: replay.files[i] ?? `#${i}`, ...d })));
  $('divergences-section').hidden = divergences.length === 0;
  fill(
    $('divergences'),
    divergences.map((d) => row([{ text: d.file, className: 'mono' }, d.index, { text: displayBytes(d.request), className: 'mono' }, { text: displayOutcome(d.expected), className: 'mono' }, { text: displayOutcome(d.actual), className: 'mono bad' }])),
  );
}

function render(state) {
  text($('project-name'), state.project.name);
  text($('project-version'), state.project.manifest ? `v${state.project.manifest.version}` : '');
  const status = $('status');
  status.dataset.status = state.status;
  text(status, state.status);
  text($('run'), state.run ? `run ${state.run}` : '');
  $('rebuild').disabled = state.status !== 'idle';
  text($('watching'), `Watching ${[...state.project.files, ...state.project.watch].join(', ')} in ${state.project.directory}`);
  text($('updated'), state.updatedAt ? `Updated ${new Date(state.updatedAt).toLocaleTimeString()}` : '');
  document.title = state.verdict ? `${state.verdict.passed ? 'PASSED' : 'FAILED'} ${state.verdict.level} · ${state.project.name} · Prism studio` : `${state.project.name} · Prism studio`;
  renderCheck(state);
  renderReplay(state);
}

function selectTab(name) {
  for (const button of document.querySelectorAll('.tabs [role="tab"]')) {
    const selected = button.dataset.tab === name;
    button.setAttribute('aria-selected', String(selected));
    document.getElementById(button.getAttribute('aria-controls')).hidden = !selected;
  }
  try {
    localStorage.setItem('prism-studio-tab', name);
  } catch {
    // A private window keeps no tab.
  }
}

function setDisconnected() {
  const status = $('status');
  status.dataset.status = 'disconnected';
  text(status, 'disconnected');
}

function follow() {
  const source = new EventSource('events');
  source.addEventListener('state', (event) => render(JSON.parse(event.data)));
  source.addEventListener('error', setDisconnected);
}

document.querySelectorAll('.tabs [role="tab"]').forEach((button) => button.addEventListener('click', () => selectTab(button.dataset.tab)));
$('rebuild').addEventListener('click', () => {
  fetch('rebuild', { method: 'POST' }).catch(setDisconnected);
});

let initial = 'check';
try {
  initial = localStorage.getItem('prism-studio-tab') === 'replay' ? 'replay' : 'check';
} catch {
  // Default tab.
}
selectTab(initial);

fetch('state')
  .then((r) => r.json())
  .then(render)
  .catch(setDisconnected)
  .finally(follow);
