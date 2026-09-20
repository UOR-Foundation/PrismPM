// Private typed DOM adapter. Display and dispatch never grant effects or roles.
import {bytesCopy} from './identity.mjs';
import {decodePresentation, encodeIntent, validateIntent, PresentationError,
  PRESENTATION_MAXIMUM} from './presentation-wire.mjs';

const roots = new WeakMap(), encoder = new TextEncoder();
const fail = code => { throw new PresentationError(code); };
const errorText = 'Unable to submit this request.';
const exact = (value, keys) => {
  if (!value || Object.getPrototypeOf(value) !== Object.prototype) fail('options');
  const fields = Object.getOwnPropertyDescriptors(value);
  if (Reflect.ownKeys(fields).length !== keys.length
    || keys.some(key => !fields[key] || !('value' in fields[key]))) fail('options');
  return Object.fromEntries(keys.map(key => [key, fields[key].value]));
};
function catalogue(value) {
  if (!Array.isArray(value) || Object.getPrototypeOf(value) !== Array.prototype) fail('labels');
  const fields = Object.getOwnPropertyDescriptors(value), length = fields.length?.value;
  if (!Number.isInteger(length) || length < 1 || length > 256
    || Reflect.ownKeys(fields).length !== length + 1) fail('labels');
  let previous = '';
  return Object.freeze(Array.from({length}, (_, index) => {
    if (!fields[index] || !('value' in fields[index])) fail('labels');
    const {id, text} = exact(fields[index].value, ['id', 'text']);
    if (typeof id !== 'string' || !/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(id)
      || id <= previous || typeof text !== 'string' || text.length > 4096 || !text.isWellFormed()
      || !text.trim() || /[\u0000-\u001f\u007f-\u009f]/u.test(text)
      || encoder.encode(text).length > 4096) fail('labels');
    previous = id;
    return text;
  }));
}
const same = (a, b) => a.length === b.length && a.every((byte, index) => byte === b[index]);

export function openPresentation(options) {
  if (arguments.length !== 1) fail('options');
  let root, labels, dispatch, maximum, requestMaximum;
  try {
    ({root, labels, dispatch, maximum, requestMaximum} = exact(options,
      ['root', 'labels', 'dispatch', 'maximum', 'requestMaximum']));
    labels = catalogue(labels);
    for (const bound of [maximum, requestMaximum]) {
      if (!Number.isInteger(bound) || bound < 1 || bound > PRESENTATION_MAXIMUM) fail('limit');
    }
    if (typeof dispatch !== 'function') fail('options');
    if (typeof HTMLElement === 'undefined' || !(root instanceof HTMLElement)
      || !root.isConnected || root.closest('form') || roots.has(root)) fail('root');
  } catch (error) {
    if (error instanceof PresentationError) throw error;
    fail('options');
  }
  const document = root.ownerDocument;
  let closed = false, frame, captured, nodes = new Map(), forms = new Map();
  let actionNodes = new Map(), active = null, diagnostic;
  const ownership = {};
  roots.set(root, ownership);
  const element = (tag, text) => {
    const node = document.createElement(tag);
    if (text !== undefined) node.textContent = text;
    return node;
  };
  const report = () => { if (!closed && diagnostic) diagnostic.textContent = errorText; };
  const stop = clear => {
    closed = true; active = null;
    root.removeEventListener('submit', onSubmit);
    root.removeEventListener('keydown', onKey);
    root.removeEventListener('input', onEdit);
    root.removeEventListener('change', onEdit);
    if (roots.get(root) === ownership) {
      if (clear) { roots.delete(root); root.replaceChildren(); root.removeAttribute('aria-busy'); }
    }
  };
  function onEdit(event) {
    for (const record of nodes.values()) {
      if (record.control === event.target && [5, 6, 7].includes(record.tag)) record.edited = true;
    }
  }
  function send(actionNode) {
    if (closed || !frame || active !== null) return;
    try {
      const selected = frame, action = selected[6][actionNode - 1]?.[1];
      if (!action || action[0] !== 8) fail('binding');
      const fields = action[5].map(id => {
        const record = nodes.get(id), control = record.control;
        let value = control.value;
        if (record.tag === 7) value = value === '' ? 0 : Number(value);
        else if (!record.edited && value === record.nativeDefault) value = record.defaultValue;
        return [id, value];
      });
      const intent = [1, selected[1], action[2], fields];
      validateIntent(selected, intent);
      const bytes = encodeIntent(intent, requestMaximum), token = {};
      // Capture and serialize before entering the private dispatcher, including
      // a synchronous reentrant call. This is not authorization or a phase.
      active = token;
      diagnostic.textContent = '';
      let returned;
      try { returned = dispatch(bytes); } catch { active = null; report(); return; }
      Promise.resolve(returned).then(value => {
        if (closed || active !== token) return;
        if (frame !== selected) fail('stale');
        render(value);
      }).catch(report).finally(() => { if (active === token) active = null; });
    } catch { report(); }
  }
  function onSubmit(event) {
    event.preventDefault();
    if (closed || !frame) return;
    const formId = forms.get(event.target);
    if (!formId) return;
    const submitter = event.submitter;
    const id = submitter ? actionNodes.get(submitter)
      : [...nodes].find(([, record]) => record.tag === 8 && record.parent === formId
        && frame[6][record.id - 1][1][4])?.[0];
    if (!id || nodes.get(id)?.parent !== formId) { report(); return; }
    send(id);
  }
  function onKey(event) {
    if (closed || !frame || event.key !== 'Enter' || event.isComposing
      || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const record = [...nodes.values()].find(value => value.tag === 5 && value.control === event.target);
    if (!record) return;
    // Native implicit submission picks the first button, not the modeled
    // default. Keep native form submission while selecting the declared one.
    event.preventDefault();
    const selected = [...nodes.values()].find(value => value.tag === 8
      && value.parent === record.parent && frame[6][value.id - 1][1][4]);
    if (selected && !selected.control.disabled) nodes.get(record.parent).element.requestSubmit(selected.control);
  }
  function render(value) {
    if (arguments.length !== 1) fail('options');
    if (closed) fail('closed');
    let bytes;
    try { bytes = bytesCopy(value, maximum); } catch { fail('bytes'); }
    if (captured && same(bytes, captured)) return;
    const next = decodePresentation(bytes, maximum);
    if (frame && next[1] <= frame[1]) fail('stale');
    const label = index => {
      if (index >= labels.length) fail('labels');
      return labels[index];
    };
    // Resolve every label before touching even a retained control.
    if (next[3]) label(next[3] - 1);
    for (const [, content] of next[6]) {
      if (content[0] === 3) label(content[2]);
      else if (content[0] !== 4) label(content[1]);
      if (content[0] === 7) for (const option of content[5]) label(option[1]);
      if (content[0] === 9) for (const column of content[2]) label(column);
    }
    const focused = [...nodes.values()].find(record => record.control === document.activeElement
      || record.element === document.activeElement);
    let selection;
    if (focused && [5, 6].includes(focused.tag)) {
      selection = [focused.control.selectionStart, focused.control.selectionEnd, focused.control.selectionDirection];
    }
    const fragment = document.createDocumentFragment(), nextNodes = new Map(), nextForms = new Map();
    const nextActions = new Map();
    try {
      const status = element('p', next[3] ? label(next[3] - 1) : '');
      status.setAttribute('role', 'status'); status.setAttribute('aria-live', ['off', 'polite', 'assertive'][next[4]]);
      status.dataset.presentationStatus = ''; fragment.append(status);
      for (let index = 0; index < next[6].length; index++) {
        const id = index + 1, [parent, content] = next[6][index], tag = content[0];
        const previous = nodes.get(id), retained = previous?.tag === tag ? previous : undefined;
        let record;
        if ([5, 6, 7].includes(tag)) {
          record = retained ?? {element: element('label'), control: element(tag === 5 ? 'input' : tag === 6 ? 'textarea' : 'select')};
          const control = record.control, defaultValue = tag === 7 ? content[4] : content[5];
          const preserve = retained && record.defaultValue === defaultValue && record.draftEpoch === content[6];
          record.preservedDraft = Boolean(preserve);
          const current = control.value;
          if (tag === 5) control.type = 'text';
          control.autocomplete = 'off'; control.disabled = !content[2]; control.required = content[3];
          if (tag === 7) {
            const options = [element('option', '')]; options[0].value = '';
            for (const [optionId, title] of content[5]) {
              const option = element('option', label(title)); option.value = String(optionId); options.push(option);
            }
            control.replaceChildren(...options);
            control.value = preserve && (current === '' || content[5].some(option => String(option[0]) === current))
              ? current : defaultValue ? String(defaultValue) : '';
          } else {
            control.maxLength = content[4]; control.spellcheck = false;
            if (!preserve) control.value = defaultValue;
          }
          if (!preserve) { record.edited = false; record.nativeDefault = control.value; }
          record.defaultValue = defaultValue; record.draftEpoch = content[6];
          record.element.replaceChildren(element('span', label(content[1])), control);
        } else if (tag === 8) {
          record = retained ?? {element: element('span'), control: element('button')};
          record.control.type = 'submit'; record.control.disabled = !content[3];
          record.control.textContent = label(content[1]); record.element.replaceChildren(record.control);
          nextActions.set(record.control, id);
        } else {
          const tags = ['section', 'nav', 'form', 'h' + content[1], 'p'];
          record = {element: element(tag === 9 ? 'table' : tags[tag])};
          if (tag <= 2) record.element.setAttribute('aria-label', label(content[1]));
          if (tag === 2) { record.element.noValidate = true; nextForms.set(record.element, id); }
          if (tag === 3) record.element.textContent = label(content[2]);
          if (tag === 4) record.element.textContent = content[1];
          if (tag === 9) {
            const head = element('thead'), row = element('tr'), body = element('tbody');
            for (const column of content[2]) { const cell = element('th', label(column)); cell.scope = 'col'; row.append(cell); }
            head.append(row);
            for (const values of content[3]) { const row = element('tr'); for (const value of values) row.append(element('td', value)); body.append(row); }
            record.element.append(element('caption', label(content[1])), head, body);
          }
        }
        Object.assign(record, {id, tag, parent}); record.element.tabIndex = -1;
        record.element.dataset.presentationNode = String(id); nextNodes.set(id, record);
        (parent ? nextNodes.get(parent).element : fragment).append(record.element);
      }
      diagnostic = element('p'); diagnostic.setAttribute('role', 'alert');
      diagnostic.dataset.presentationDiagnostic = ''; fragment.append(diagnostic);
      root.replaceChildren(fragment); root.setAttribute('aria-busy', String(next[2] === 1));
      frame = next; captured = bytes; nodes = nextNodes; forms = nextForms; actionNodes = nextActions;
      const surviving = focused && nodes.get(focused.id);
      const chosen = next[5] ? nodes.get(next[5]) : surviving?.tag === focused?.tag ? surviving : undefined;
      if (chosen) {
        const target = chosen.control && !chosen.control.disabled ? chosen.control : chosen.element;
        target.focus({preventScroll: true});
        if (!next[5] && chosen === focused && chosen.preservedDraft && selection) chosen.control.setSelectionRange(...selection);
      }
      if (next[2] === 3) stop(false);
    } catch (error) { stop(true); throw error instanceof PresentationError ? error : new PresentationError('dom'); }
  }
  root.addEventListener('submit', onSubmit);
  root.addEventListener('keydown', onKey);
  root.addEventListener('input', onEdit);
  root.addEventListener('change', onEdit);
  return Object.freeze({render, close() { if (arguments.length) fail('options'); stop(true); }});
}
