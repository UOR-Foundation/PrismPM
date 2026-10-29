/**
 * The release entries of CHANGELOG.md held to STABILITY.md section 2, the
 * pre-1.0 rule: a minor bump may break you, a patch bump never does, and every
 * entry uses the four headers below, in this order.
 *
 * Two checks, both on the file alone:
 *   - headers: an entry newer than LEGACY_THROUGH uses only the four headers,
 *     once each and in order, and always has Breaking Changes (empty says None.);
 *   - bump: below 1.0 an entry with breaking changes is a minor bump over the
 *     entry before it, and an entry without any is a patch bump.
 *
 * Entries up to and including LEGACY_THROUGH were written before the rule and
 * keep the Keep a Changelog headers they were published with. They are never
 * rewritten: `release-check` reads the 0.4.0 entry and the release notes cite it.
 */

export const HEADERS = ['Features Added', 'Breaking Changes', 'Bugs Fixed', 'Other Changes'];
export const LEGACY_THROUGH = '0.4.0';

const ENTRY = /^## \[(Unreleased|\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)\](?: - (\d{4}-\d{2}-\d{2}))?\s*$/;

/** @returns {number[]} major, minor, patch; a pre-release suffix is ignored. */
export function parseVersion(version) {
  return version.split('-')[0].split('.').map(Number);
}

function newerThan(a, b) {
  const [x, y] = [parseVersion(a), parseVersion(b)];
  for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return x[i] > y[i];
  return false;
}

/**
 * @returns {{ version: string, date: string|null, line: number, sections: { name: string, line: number, empty: boolean }[] }[]}
 *   the release entries, in the order written (newest first).
 */
export function parseChangelog(text) {
  const entries = [];
  let entry = null;
  let section = null;
  text.split(/\r?\n/).forEach((raw, index) => {
    const line = index + 1;
    if (/^## /.test(raw)) {
      const match = ENTRY.exec(raw);
      entry = match ? { version: match[1], date: match[2] ?? null, line, sections: [] } : null;
      section = null;
      if (entry) entries.push(entry);
      return;
    }
    if (!entry) return;
    const heading = /^### (.+?)\s*$/.exec(raw);
    if (heading) {
      section = { name: heading[1], line, empty: true };
      entry.sections.push(section);
    } else if (section && raw.trim() && !/^none\.?$/i.test(raw.trim())) {
      section.empty = false;
    }
  });
  return entries;
}

/** @returns {{ line: number, message: string }[]} */
export function releaseEntryProblems(text) {
  const problems = [];
  const entries = parseChangelog(text);
  entries.forEach((entry, index) => {
    const released = entry.version !== 'Unreleased';
    if (released && !entry.date) problems.push({ line: entry.line, message: `${entry.version} has no date: write ## [${entry.version}] - YYYY-MM-DD` });
    if (released && !newerThan(entry.version, LEGACY_THROUGH)) return;

    for (const section of entry.sections) {
      if (!HEADERS.includes(section.name)) problems.push({ line: section.line, message: `${entry.version}: ### ${section.name} is not one of ${HEADERS.map((h) => `### ${h}`).join(', ')}` });
    }
    const known = entry.sections.filter((s) => HEADERS.includes(s.name));
    known.forEach((section, i) => {
      if (known.findIndex((s) => s.name === section.name) !== i) problems.push({ line: section.line, message: `${entry.version}: ### ${section.name} appears twice` });
      else if (i > 0 && HEADERS.indexOf(section.name) < HEADERS.indexOf(known[i - 1].name)) problems.push({ line: section.line, message: `${entry.version}: ### ${section.name} is out of order, the order is ${HEADERS.join(', ')}` });
    });
    const breaking = entry.sections.find((s) => s.name === 'Breaking Changes');
    if (released && !breaking) problems.push({ line: entry.line, message: `${entry.version} has no ### Breaking Changes: write None. when it breaks nothing` });
    if (!released || !breaking) return;

    const previous = entries.slice(index + 1).find((e) => e.version !== 'Unreleased');
    if (!previous) return;
    const [major, minor, patch] = parseVersion(entry.version);
    const [pMajor, pMinor, pPatch] = parseVersion(previous.version);
    if (major !== 0 || pMajor !== 0) return;
    const minorBump = minor === pMinor + 1 && patch === 0;
    const patchBump = minor === pMinor && patch === pPatch + 1;
    if (!breaking.empty && !minorBump) problems.push({ line: breaking.line, message: `${entry.version} lists breaking changes, so before 1.0 it must be a minor bump over ${previous.version}` });
    if (breaking.empty && !patchBump) problems.push({ line: entry.line, message: `${entry.version} breaks nothing, so before 1.0 it must be a patch bump over ${previous.version}` });
  });
  return problems;
}
