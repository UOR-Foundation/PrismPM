import {encodeWire} from '../../sdk/browser/presentation-wire.mjs';

export const frame = (options = [[42, 'Shared name'], [7, 'Shared name']], selected = 42) =>
  [1, 1, 0, 0, 0, 0, [[0, [2, 0]],
    [1, [11, 1, true, true, selected, options, 0]],
    [1, [8, 2, 1, true, true, [2]]]]];

export function corpus() {
  const positive = [
    ['SourceOrderDuplicateNames', frame()],
    ['UnicodeMarkupText', frame([[4294967295, '\uFEFF<svg onload="bad()">😀</svg>'], [1, 'é']], 1)],
    ['ExactNameBytes', frame([[1, '😀'.repeat(1024)]], 1)],
    ['NoSelection', frame([], 0)],
  ];
  for (let phase = 1; phase <= 3; phase++) {
    const value = frame(); value[2] = phase;
    value[6][1][1][2] = false; value[6][2][1][3] = false;
    positive.push(['DisabledPhase' + phase, value]);
  }
  const negative = [
    ['DuplicateIdentifier', x => { x[6][1][1][5][1][0] = 42; }],
    ['ZeroIdentifier', x => { x[6][1][1][5][1][0] = 0; }],
    ['MissingSelection', x => { x[6][1][1][4] = 2; }],
    ['EmptyName', x => { x[6][1][1][5][1][1] = ''; }],
    ['OverNameBytes', x => { x[6][1][1][5][1][1] = '😀'.repeat(1024) + 'x'; }],
    ['CatalogueIndexAsText', x => { x[6][1][1][5][1][1] = 1; }],
    ['OptionExtraField', x => { x[6][1][1][5][1].push('extra'); }],
    ['MissingEpoch', x => { x[6][1][1].pop(); }],
    ['OutsideForm', x => { x[6][1][0] = 0; }],
    ['EnabledPending', x => { x[2] = 1; }],
    ['DisabledBoundField', x => { x[6][1][1][2] = false; }],
  ].map(([id, mutate]) => { const value = frame(); mutate(value); return [id, value]; });
  return [...positive.map(([id, value]) => ({id, value, accepted: true, request: encodeWire(value)})),
    ...negative.map(([id, value]) => ({id, value, accepted: false, request: encodeWire(value)}))];
}

export function mixedOptions(count) {
  const value = frame(Array.from({length: count - 1}, (_, i) => [i + 1, 'Name ' + i]), 1);
  value[6].push([1, [7, 1, true, false, 4294967295, [[4294967295, 2]], 0]]);
  return value;
}
