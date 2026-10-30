/** What the command line needs beyond the public entry point. */
import { readJson, sorted } from './util.mjs';
import { validateReport } from './runner.mjs';

export * from './index.mjs';
export { sorted };

/** A report read from a file, validated. */
export function readJsonReport(path) {
  return validateReport(readJson(path, 'report.unreadable'), path);
}
