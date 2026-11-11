#!/usr/bin/env node
// Offline schema oracle for recorded MCP 2026-07-28 transcripts.
//
// Every JSON-RPC message in a transcript is validated, with the exact Ajv
// implementation already locked for the other JSON-Schema oracles, against the
// schema published at the pinned specification commit. Requests and
// notifications are checked against the definition their method names, and a
// response is checked against the definition of the request it answers.
//
// usage: mcp-schema-validator SCHEMA TRANSCRIPT
//
// Exit 0 when every message is valid, 1 when the transcript is not a valid
// 2026-07-28 exchange (the diagnostic names the message index and the schema
// location), 2 when the oracle itself cannot run.
import { readFile } from 'node:fs/promises';
import Ajv2020 from 'ajv/dist/2020.js';

const SPEC = '2026-07-28';
const TRANSCRIPT = 'prismpm/mcp-transcript/1';
const MAX_MESSAGES = 100000;
const MAX_ERRORS = 32;
const DIRECTIONS = new Set(['client-to-server', 'server-to-client']);

class Invalid extends Error {}

const fail = (message) => {
  throw new Invalid(message);
};

const [schemaPath, inputPath, ...extra] = process.argv.slice(2);
if (!schemaPath || !inputPath || extra.length > 0) {
  process.stderr.write('usage: mcp-schema-validator SCHEMA TRANSCRIPT\n');
  process.exit(2);
}

let validator;
let defs;
try {
  const schema = JSON.parse(await readFile(schemaPath, 'utf8'));
  defs = schema.$defs;
  if (!defs || typeof defs !== 'object' || !defs.JSONRPCMessage) {
    throw new Error('the MCP schema has no JSONRPCMessage definition');
  }
  const ajv = new Ajv2020({ allErrors: true, strict: false, validateFormats: false });
  ajv.addSchema(schema, 'mcp');
  validator = (name) => {
    const compiled = ajv.getSchema(`mcp#/$defs/${name}`);
    if (!compiled) throw new Error(`the MCP schema has no ${name} definition`);
    return compiled;
  };
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(2);
}

// Request and notification definitions keyed by the method constant they pin.
function methodTable(unionName) {
  const table = new Map();
  const union = defs[unionName];
  // A union lists its members; a family with one member is that definition itself.
  const members = union?.anyOf ?? (union ? [{ $ref: `#/$defs/${unionName}` }] : []);
  for (const member of members) {
    const name = member.$ref?.split('/').pop();
    const method = defs[name]?.properties?.method?.const;
    if (name && typeof method === 'string') table.set(method, name);
  }
  return table;
}
const clientRequests = methodTable('ClientRequest');
const clientNotifications = methodTable('ClientNotification');
const serverNotifications = methodTable('ServerNotification');

// Typed JSON-RPC errors are selected by the numeric code each definition pins.
const typedErrors = new Map();
for (const [name, definition] of Object.entries(defs)) {
  const code = definition?.properties?.code?.const;
  if (name.endsWith('Error') && Number.isInteger(code)) typedErrors.set(code, name);
}

function check(index, name, value, problems) {
  const validate = validator(name);
  if (validate(value)) return;
  for (const error of validate.errors ?? []) {
    if (problems.length >= MAX_ERRORS) return;
    problems.push({
      message: index,
      definition: name,
      location: error.instancePath || '/',
      keyword: error.keyword,
      detail: error.message,
    });
  }
}

try {
  const input = await readFile(inputPath);
  if (input.length === 0) fail('the transcript is empty');
  let transcript;
  try {
    transcript = JSON.parse(input.toString('utf8'));
  } catch (error) {
    fail(`the transcript is not JSON: ${error.message}`);
  }
  if (transcript?.version !== TRANSCRIPT) fail(`the transcript version must be ${TRANSCRIPT}`);
  if (transcript.spec !== SPEC) fail(`the transcript must declare specification ${SPEC}`);
  const messages = transcript.messages;
  if (!Array.isArray(messages) || messages.length === 0 || messages.length > MAX_MESSAGES) {
    fail(`the transcript must hold between 1 and ${MAX_MESSAGES} messages`);
  }

  const problems = [];
  // Open requests by direction, so a response is read against the request it answers.
  const open = { 'client-to-server': new Map(), 'server-to-client': new Map() };

  messages.forEach((entry, index) => {
    // Specific findings name the definition the message was held to; the
    // generic JSON-RPC envelope findings are reported only when none exist.
    const envelope = [];
    const specific = [];
    const note = (text) => specific.push({ message: index, definition: null, detail: text });
    inspect(entry, index, envelope, specific, note);
    for (const problem of specific.length > 0 ? specific : envelope) {
      if (problems.length < MAX_ERRORS) problems.push(problem);
    }
  });

  function inspect(entry, index, envelope, specific, note) {
    const direction = entry?.direction;
    const message = entry?.message;
    if (!DIRECTIONS.has(direction) || message === null || typeof message !== 'object' || Array.isArray(message)) {
      note('an entry needs a direction and a JSON-RPC object');
      return;
    }
    check(index, 'JSONRPCMessage', message, envelope);
    const answering = direction === 'client-to-server' ? 'server-to-client' : 'client-to-server';
    if (typeof message.method === 'string' && 'id' in message) {
      // A request.
      if (direction === 'server-to-client') {
        note(`${SPEC} defines no server-to-client JSON-RPC request; server input is carried in an input-required result`);
        return;
      }
      const definition = clientRequests.get(message.method);
      if (!definition) {
        note(`${message.method} is not a client request defined by ${SPEC}`);
        return;
      }
      check(index, definition, message, specific);
      open[direction].set(JSON.stringify(message.id), message.method);
    } else if (typeof message.method === 'string') {
      // A notification.
      const table = direction === 'client-to-server' ? clientNotifications : serverNotifications;
      const definition = table.get(message.method);
      if (!definition) {
        note(`${message.method} is not a ${direction} notification defined by ${SPEC}`);
        return;
      }
      check(index, definition, message, specific);
    } else if ('result' in message || 'error' in message) {
      // A response.
      const key = JSON.stringify(message.id);
      const method = open[answering].get(key);
      if (method === undefined && !('error' in message && message.id === undefined)) {
        note(`a response answers no open ${answering} request (id ${key})`);
        return;
      }
      if (method !== undefined) open[answering].delete(key);
      if ('result' in message) {
        const request = clientRequests.get(method);
        const named = request?.replace(/Request$/, 'ResultResponse');
        check(index, named && defs[named] ? named : 'JSONRPCResultResponse', message, specific);
      } else {
        check(index, 'JSONRPCErrorResponse', message, specific);
        const typed = typedErrors.get(message.error?.code);
        if (typed) check(index, typed, message.error, specific);
      }
    } else {
      note('the object is neither a request, a notification nor a response');
    }
  }

  if (problems.length > 0) {
    process.stderr.write(`${JSON.stringify({ oracle: `mcp-schema/${SPEC}`, valid: false, errors: problems })}\n`);
    process.exit(1);
  }
  process.stdout.write(`${JSON.stringify({ oracle: `ajv/8.20.0+mcp-schema/${SPEC}`, valid: true, messages: messages.length })}\n`);
} catch (error) {
  if (error instanceof Invalid) {
    process.stderr.write(`${JSON.stringify({ oracle: `mcp-schema/${SPEC}`, valid: false, errors: [{ detail: error.message }] })}\n`);
    process.exit(1);
  }
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(2);
}
