// Shared deterministic acceptance inputs, not application presentation logic.
const node = (content, parent = 0) => [parent,
  [5, 6, 7].includes(content[0]) && content.length === 6 ? [...content, 0] : content];
const view = nodes => [1, 1, 0, 0, 0, 0, nodes];

export const maximumCases = Object.freeze(['FrameMaximum',
  'FrameNodesMaximumFirst', 'FrameCellsMaximum1First', 'FrameCellsMaximum16First', 'FrameCombinedMaximumFirst',
  'FrameNodesMaximumLast', 'FrameCellsMaximum1Last', 'FrameCellsMaximum16Last', 'FrameCombinedMaximumLast']);

export function combinedShape(rowMaximum) {
  const nodes = Array.from({length: 14}, (_, i) => node([0, 0], i));
  nodes.push(node([2, 0], 14));
  nodes.push(...Array.from({length: 16}, () => node([5, 0, true, false, 1, ''], 15)));
  nodes.push(node([7, 0, true, false, 0, Array.from({length: 256}, (_, i) => [i + 1, 0])], 15));
  nodes.push(...Array.from({length: 64}, (_, i) => node([8, 0, i + 1, true, i === 0,
    Array.from({length: 16}, (_, j) => j + 16)], 15)));
  while (nodes.length < (rowMaximum ? 254 : 255)) nodes.push(node([4, '']));
  if (rowMaximum) nodes.push(node([9, 0, Array(16).fill(0), []], 14));
  nodes.push(node([9, 0, Array(rowMaximum ? 1 : 16).fill(0),
    Array.from({length: rowMaximum ? 4096 : 256}, () => Array(rowMaximum ? 1 : 16).fill(''))], 14));
  return view(nodes);
}

export function maximumShape(id) {
  if (!maximumCases.includes(id)) throw Error('unknown maximum fixture');
  const last = id.endsWith('Last');
  let frame, nodeIndex, rowIndex, columnIndex, path;
  if (id === 'FrameMaximum' || id.startsWith('FrameNodesMaximum')) {
    const count = id === 'FrameMaximum' ? 1 : 256;
    frame = view(Array.from({length: count}, () => node([4, ''])));
    nodeIndex = last ? count - 1 : 0;
    path = [6, nodeIndex, 1, 1];
  } else {
    if (id.startsWith('FrameCombinedMaximum')) frame = combinedShape(true);
    else {
      const columns = id.startsWith('FrameCellsMaximum16') ? 16 : 1;
      frame = view([node([9, 0, Array(columns).fill(0),
        Array.from({length: 4096 / columns}, () => Array(columns).fill(''))])]);
    }
    nodeIndex = frame[6].length - 1;
    const rows = frame[6][nodeIndex][1][3];
    rowIndex = last ? rows.length - 1 : 0;
    columnIndex = last ? rows[0].length - 1 : 0;
    path = [6, nodeIndex, 1, 3, rowIndex, columnIndex];
  }
  const selector = `[data-presentation-node="${nodeIndex + 1}"]`
    + (rowIndex === undefined ? '' : ` tbody tr:nth-child(${rowIndex + 1}) td:nth-child(${columnIndex + 1})`);
  return {frame, selector, setText(text) {
    const container = path.slice(0, -1).reduce((value, index) => value[index], frame);
    container[path.at(-1)] = text;
  }};
}
