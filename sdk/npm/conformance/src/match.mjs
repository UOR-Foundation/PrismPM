/**
 * The expectation matcher. A pattern is JSON: a primitive matches by
 * equality, an object matches when every member it names matches (members it
 * does not name are free), and an array matches element by element with the
 * same length. An object whose keys all start with `$` is an operator:
 *
 *   { "$type": "string" }      typeof, with "array" and "null" distinguished
 *   { "$min": 1 }              a number at least this
 *   { "$max": 8 }              a number at most this
 *   { "$regex": "^blake3:" }   a string the regular expression matches
 *   { "$length": pattern }     the length of a string or array, matched by pattern
 *   { "$some": pattern }       an array with at least one element matching pattern
 *
 * Operators combine: { "$type": "number", "$min": 1 }. `mismatches` returns
 * every difference with its path, so a failed scenario names what differed.
 */
import { isObject } from './util.mjs';

export const OPERATORS = Object.freeze(['$type', '$min', '$max', '$regex', '$length', '$some']);
const TYPES = Object.freeze(['string', 'number', 'boolean', 'object', 'array', 'null']);

export function typeOf(value) {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'array';
  return typeof value;
}

const isOperator = (pattern) => isObject(pattern) && Object.keys(pattern).length > 0 && Object.keys(pattern).every((key) => key.startsWith('$'));

/** Throws when a pattern uses an operator this matcher does not define, or one with a wrong argument. */
export function validatePattern(pattern, path = '$') {
  if (Array.isArray(pattern)) {
    pattern.forEach((item, index) => validatePattern(item, `${path}[${index}]`));
    return;
  }
  if (!isObject(pattern)) return;
  const keys = Object.keys(pattern);
  const operators = keys.filter((key) => key.startsWith('$'));
  if (operators.length > 0 && operators.length !== keys.length) {
    throw new Error(`${path}: a pattern mixes operators ${operators.join(', ')} with members`);
  }
  if (operators.length === 0) {
    for (const key of keys) validatePattern(pattern[key], `${path}.${key}`);
    return;
  }
  for (const key of operators) {
    if (!OPERATORS.includes(key)) throw new Error(`${path}: unknown operator ${key}; the operators are ${OPERATORS.join(', ')}`);
    const argument = pattern[key];
    if (key === '$type' && !TYPES.includes(argument)) throw new Error(`${path}: $type takes one of ${TYPES.join(', ')}`);
    if ((key === '$min' || key === '$max') && typeof argument !== 'number') throw new Error(`${path}: ${key} takes a number`);
    if (key === '$regex') {
      if (typeof argument !== 'string') throw new Error(`${path}: $regex takes a string`);
      new RegExp(argument);
    }
    if (key === '$length' || key === '$some') validatePattern(argument, `${path}.${key}`);
  }
}

function describe(value) {
  const text = JSON.stringify(value);
  return text === undefined ? 'undefined' : text.length > 200 ? `${text.slice(0, 197)}...` : text;
}

/**
 * Every place `actual` does not match `pattern`, as { path, expected, actual }.
 * Empty when it matches.
 */
export function mismatches(pattern, actual, path = '$') {
  const out = [];
  if (isOperator(pattern)) {
    for (const [operator, argument] of Object.entries(pattern)) {
      switch (operator) {
        case '$type':
          if (typeOf(actual) !== argument) out.push({ path, expected: `a ${argument}`, actual: describe(actual) });
          break;
        case '$min':
          if (typeof actual !== 'number' || actual < argument) out.push({ path, expected: `a number at least ${argument}`, actual: describe(actual) });
          break;
        case '$max':
          if (typeof actual !== 'number' || actual > argument) out.push({ path, expected: `a number at most ${argument}`, actual: describe(actual) });
          break;
        case '$regex':
          if (typeof actual !== 'string' || !new RegExp(argument).test(actual)) out.push({ path, expected: `a string matching /${argument}/`, actual: describe(actual) });
          break;
        case '$length':
          if (typeof actual !== 'string' && !Array.isArray(actual)) out.push({ path, expected: 'a string or an array', actual: describe(actual) });
          else out.push(...mismatches(argument, actual.length, `${path}.length`));
          break;
        case '$some':
          if (!Array.isArray(actual)) out.push({ path, expected: 'an array', actual: describe(actual) });
          else if (!actual.some((item) => mismatches(argument, item).length === 0)) {
            out.push({ path, expected: `an array with an element matching ${describe(argument)}`, actual: describe(actual) });
          }
          break;
        default:
          throw new Error(`${path}: unknown operator ${operator}`);
      }
    }
    return out;
  }
  if (Array.isArray(pattern)) {
    if (!Array.isArray(actual)) return [{ path, expected: `an array of ${pattern.length}`, actual: describe(actual) }];
    if (actual.length !== pattern.length) return [{ path, expected: `an array of ${pattern.length}`, actual: `an array of ${actual.length}` }];
    pattern.forEach((item, index) => out.push(...mismatches(item, actual[index], `${path}[${index}]`)));
    return out;
  }
  if (isObject(pattern)) {
    if (!isObject(actual)) return [{ path, expected: 'an object', actual: describe(actual) }];
    for (const [key, member] of Object.entries(pattern)) {
      if (!Object.hasOwn(actual, key)) out.push({ path: `${path}.${key}`, expected: describe(member), actual: 'absent' });
      else out.push(...mismatches(member, actual[key], `${path}.${key}`));
    }
    return out;
  }
  if (actual !== pattern) out.push({ path, expected: describe(pattern), actual: describe(actual) });
  return out;
}

export const matches = (pattern, actual) => mismatches(pattern, actual).length === 0;
