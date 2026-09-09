#!/usr/bin/env node
// Thin offline invocation of the exact Ajv implementation recommended by the
// upstream SPDX 3 model's validation instructions and its published schema.
import { readFile } from 'node:fs/promises';
import Ajv2020 from 'ajv/dist/2020.js';

const [schemaPath, inputPath] = process.argv.slice(2);
if (!schemaPath || !inputPath) {
  process.stderr.write('usage: spdx-validator SCHEMA INPUT\n');
  process.exit(2);
}
try {
  const schema = JSON.parse(await readFile(schemaPath, 'utf8'));
  const inputBytes = inputPath === '-'
    ? Buffer.from(process.env.PRISMPM_SPDX_DOCUMENT_BASE64 ?? '', 'base64')
    : await readFile(inputPath);
  if (inputBytes.length === 0) throw new Error('SPDX input is empty');
  const input = JSON.parse(inputBytes.toString('utf8'));
  const validator = new Ajv2020({ allErrors: true, strict: false }).compile(schema);
  if (!validator(input)) {
    const errors = validator.errors ?? [];
    const actionable = errors.filter(error =>
      error.keyword !== 'if'
      && error.keyword !== 'anyOf'
      && !(error.keyword === 'const' && String(error.params?.allowedValue ?? '').startsWith('Not a ')));
    process.stderr.write(`${JSON.stringify({ errorCount: errors.length, errors: actionable.slice(0, 32) })}\n`);
    process.exit(1);
  }
  process.stdout.write('{"oracle":"ajv/8.20.0+spdx/3.0.1","valid":true}\n');
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(2);
}
