// Generic typed presentation renderer. No admission or command policy lives here.
import {bytesCopy} from './identity.mjs';
import {fail} from './view-error.mjs';
const encoder = new TextEncoder(), decoder = new TextDecoder('utf-8', {fatal: true, ignoreBOM: true});
const keys = ['action','action0','action1','action2','action3','action4','asOf','author','body',
  'close','closed','conflict','contributor','event','inputError','members','message','messages',
  'next','none','offset','owner','pending','principal','reader','ready','refresh','rejected',
  'replay','result','role','select','spec','submit','title','total','unavailable','unknown','workspace'].sort();
const hex = bytes => Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
const read16 = (b, at) => b[at] * 256 + b[at + 1];
const read24 = (b, at) => b[at] * 65536 + b[at + 1] * 256 + b[at + 2];
export function captureLabels(value) {
  try {
    const text = decoder.decode(bytesCopy(value, 8192)), labels = JSON.parse(text);
    if (!labels || typeof labels !== 'object' || Array.isArray(labels)
        || Object.keys(labels).sort().join(',') !== keys.join(',')) throw 0;
    if (labels.spec !== 'prismpm/workspace-view-labels/1') throw 0;
    for (const key of keys) if (typeof labels[key] !== 'string' || !labels[key].isWellFormed()
      || encoder.encode(labels[key]).length < 1 || encoder.encode(labels[key]).length > 256) throw 0;
    if (JSON.stringify(Object.fromEntries(keys.map(key => [key, labels[key]]))) !== text) throw 0;
    return Object.freeze(labels);
  } catch { throw fail('invalid-labels'); }
}
export function presentation(bytes) {
  try {
    if (bytes.length < 84 || bytes.length > 66676 || hex(bytes.subarray(0, 5)) !== '0050564e01'
      || bytes[5] > 3 || bytes[6] > 4 || bytes[7] > 1 || bytes[8] > 127 || bytes[9] > 1
      || bytes[10] > 2 || bytes[79] > 16 || bytes[80] > 1 || 84 + read24(bytes, 81) !== bytes.length) throw 0;
    const rows = [], count = bytes[79]; let at = 84;
    for (let index = 0; index < count; index++) {
      if (bytes[7] === 0) {
        if (at + 33 > bytes.length || bytes[at + 32] > 2) throw 0;
        rows.push([hex(bytes.subarray(at, at + 32)), bytes[at + 32]]); at += 33;
      } else {
        if (at + 66 > bytes.length) throw 0;
        const size = read16(bytes, at + 64);
        if (size < 1 || size > 4096 || at + 66 + size > bytes.length) throw 0;
        rows.push([hex(bytes.subarray(at, at + 32)), hex(bytes.subarray(at + 32, at + 64)),
          decoder.decode(bytes.subarray(at + 66, at + 66 + size))]); at += 66 + size;
      }
    }
    if (at !== bytes.length) throw 0;
    return {phase: bytes[5], status: bytes[6], table: bytes[7], controls: bytes[8], focus: bytes[9],
      live: bytes[10], workspace: hex(bytes.subarray(11, 43)), head: hex(bytes.subarray(43, 75)),
      total: read16(bytes, 75), offset: read16(bytes, 77), count, next: bytes[80], rows};
  } catch { throw fail('invalid-generated-output'); }
}

export function makeRenderer(root, labels, dispatch) {
  if (!(root instanceof HTMLElement) || !root.isConnected) throw fail('invalid-root');
  const document = root.ownerDocument, listeners = [], controls = [];
  let closed = false;
  const element = (tag, text) => { const node = document.createElement(tag); if (text !== undefined) node.textContent = text; return node; };
  const listen = (node, type, action) => { node.addEventListener(type, action); listeners.push(() => node.removeEventListener(type, action)); };
  const section = element('section'); section.setAttribute('aria-label', labels.title);
  const title = element('h1', labels.title), status = element('p'), diagnostic = element('p');
  status.setAttribute('role', 'status'); status.dataset.slot = 'status';
  diagnostic.setAttribute('role', 'alert'); diagnostic.dataset.slot = 'diagnostic';
  const report = () => { if (!closed) diagnostic.textContent = labels.inputError; };
  const submit = bytes => { diagnostic.textContent = ''; Promise.resolve(dispatch(bytes)).catch(report); };
  const button = (label, bit, intent) => {
    const node = element('button', label); node.type = 'button'; controls.push([node, bit]);
    listen(node, 'click', () => submit(Uint8Array.of(intent))); return node;
  };
  const inputLabel = (text, node) => { const label = element('label'); node.setAttribute('aria-label', text); label.append(element('span', text), node); return label; };
  const selection = element('form'), workspace = element('input');
  workspace.type = 'text'; workspace.maxLength = 64; workspace.autocomplete = 'off'; workspace.spellcheck = false;
  workspace.required = true; workspace.pattern = '[0-9a-f]{64}'; workspace.dataset.slot = 'workspace-input';
  const select = element('button', labels.select); select.type = 'submit';
  controls.push([workspace, 1], [select, 1]); selection.append(inputLabel(labels.workspace, workspace), select);
  const bytes32 = text => {
    if (!/^[0-9a-f]{64}$/.test(text)) throw fail('invalid-input');
    return Uint8Array.from(text.match(/../g), byte => parseInt(byte, 16));
  };
  listen(selection, 'submit', event => { event.preventDefault(); try { submit(new Uint8Array([0, ...bytes32(workspace.value)])); } catch { report(); } });
  const navigation = element('nav'); navigation.setAttribute('aria-label', labels.result);
  navigation.append(button(labels.members, 2, 1), button(labels.messages, 4, 2), button(labels.next, 8, 3),
    button(labels.refresh, 32, 5), button(labels.close, 64, 6));
  const command = element('form'), action = element('select'), body = element('textarea');
  for (let index = 0; index < 5; index++) { const option = element('option', labels['action' + index]); option.value = String(index); action.append(option); }
  body.maxLength = 4096; body.dataset.slot = 'command-body'; body.spellcheck = false;
  const send = element('button', labels.submit); send.type = 'submit';
  controls.push([action, 16], [body, 16], [send, 16]);
  command.append(inputLabel(labels.action, action), inputLabel(labels.body, body), send);
  listen(command, 'submit', event => {
    event.preventDefault();
    try {
      const selected = Number(action.value); if (!Number.isInteger(selected) || selected < 0 || selected > 4) throw 0;
      if (!body.value.isWellFormed()) throw 0;
      const bytes = selected === 0 ? new Uint8Array() : selected === 4 ? encoder.encode(body.value) : bytes32(body.value);
      if (bytes.length > 4096) throw 0;
      submit(new Uint8Array([4, selected, ...bytes]));
    } catch { report(); }
  });
  const result = element('section'); result.tabIndex = -1; result.setAttribute('aria-label', labels.result); result.dataset.slot = 'result';
  const selected = element('p'), asOf = element('p'), paging = element('p'), table = element('table');
  selected.dataset.slot = 'selection'; asOf.dataset.slot = 'as-of'; paging.dataset.slot = 'paging';
  result.append(status, selected, asOf, paging, table); section.append(title, selection, navigation, command, diagnostic, result);
  root.replaceChildren(section);
  return Object.freeze({
    render(value) {
      section.setAttribute('aria-busy', String(value.phase === 1));
      for (const [node, bit] of controls) node.disabled = (value.controls & bit) === 0;
      status.setAttribute('aria-live', ['off', 'polite', 'assertive'][value.live]);
      status.textContent = value.status ? labels[['', 'rejected', 'conflict', 'unknown', 'unavailable'][value.status]]
        : labels[['ready', 'pending', 'replay', 'closed'][value.phase]];
      selected.textContent = labels.workspace + ': ' + (/^0+$/.test(value.workspace) ? labels.none : value.workspace);
      asOf.textContent = labels.asOf + ': ' + (/^0+$/.test(value.head) ? labels.none : value.head);
      paging.textContent = labels.total + ': ' + value.total + '; ' + labels.offset + ': ' + value.offset;
      const caption = element('caption', value.table === 0 ? labels.members : labels.messages), head = element('thead'), header = element('tr'), rows = element('tbody');
      for (const key of value.table === 0 ? ['principal', 'role'] : ['event', 'author', 'message']) {
        const cell = element('th', labels[key]); cell.scope = 'col'; header.append(cell);
      }
      head.append(header);
      for (const row of value.rows) {
        const node = element('tr');
        for (let index = 0; index < row.length; index++) node.append(element('td', value.table === 0 && index === 1 ? labels[['owner', 'contributor', 'reader'][row[index]]] : row[index]));
        rows.append(node);
      }
      table.replaceChildren(caption, head, rows);
      if (value.focus === 1) result.focus({preventScroll: true});
    },
    close(failed = false) { closed = true; for (const remove of listeners.splice(0)) remove(); diagnostic.textContent = ''; if (failed) root.replaceChildren(); },
  });
}
