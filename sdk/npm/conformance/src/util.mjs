/**
 * Small helpers shared by the suite: canonical JSON, file reading and the
 * error every command reports as a usage or data problem.
 */
import { readFileSync } from 'node:fs';

/** A problem in the suite's own data, the command line or a runtime file. Exit 2. */
export class ConformanceError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'ConformanceError';
    this.code = code;
  }
}

export const fail = (code, message) => {
  throw new ConformanceError(code, message);
};

export const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

/** JSON with members sorted by key and no whitespace: one spelling per value. */
export function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  if (isObject(value)) {
    const keys = Object.keys(value).sort();
    return `{${keys.map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(',')}}`;
  }
  if (value === undefined) return 'null';
  return JSON.stringify(value);
}

export function readJson(path, code = 'file.unreadable') {
  let text;
  try {
    text = readFileSync(path, 'utf8');
  } catch (cause) {
    fail(code, `${path} cannot be read: ${cause.code ?? cause.message}`);
  }
  try {
    return JSON.parse(text);
  } catch (cause) {
    fail('file.json', `${path} is not JSON: ${cause.message}`);
  }
}

/** Sort a copy of an object by key, recursively, so reports and files are written in one order. */
export function sorted(value) {
  if (Array.isArray(value)) return value.map(sorted);
  if (isObject(value)) return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sorted(value[key])]));
  return value;
}

export const lf = (text) => text.replace(/\r\n/g, '\n');
