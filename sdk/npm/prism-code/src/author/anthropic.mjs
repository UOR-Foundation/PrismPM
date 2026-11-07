/**
 * The default agent's two pure halves: the request it sends and what it does
 * with the answer. `anthropic-cli.mjs` holds the one call between them, so
 * these are tested with no network and no SDK installed.
 */
import { RESPONSE_SCHEMA } from './agent.mjs';

const SYSTEM = `You write LexLean models for Prism applications. You receive a prism/author-request/1 document: a specification made of examples, intent sentences and types. Answer with a prism/author-response/1 document.

The rules in context.rules bind you. A compiler and a proof kernel check everything you write, and your claims carry no weight of their own, so report honestly: a sentence you cannot prove is status "unproved" with an explanation a developer can act on, and that outcome is correct, not a failure of yours. When previous is present, your last attempt was rejected; previous.diagnostics says why.

The scaffold in context.scaffold is the project every model starts from. Any file you write replaces the scaffold's file at that path. Files you omit stay as the scaffold has them.`;

const FILE = {
  type: 'object',
  additionalProperties: false,
  required: ['path', 'text'],
  properties: { path: { type: 'string' }, text: { type: 'string' } },
};
const SENTENCE = {
  type: 'object',
  additionalProperties: false,
  required: ['theorem', 'status', 'statement', 'explanation'],
  properties: {
    theorem: { type: 'string' },
    status: { type: 'string', enum: ['proved', 'unproved'] },
    statement: { type: 'string' },
    explanation: { type: 'string' },
  },
};
const TYPE = {
  type: 'object',
  additionalProperties: false,
  required: ['source', 'record'],
  properties: { source: { type: 'string' }, record: { type: 'string' } },
};

export const RESPONSE_FORMAT = Object.freeze({
  type: 'json_schema',
  schema: {
    type: 'object',
    additionalProperties: false,
    required: ['files', 'sentences', 'types'],
    properties: {
      files: { type: 'array', items: FILE },
      sentences: { type: 'array', items: SENTENCE },
      types: { type: 'array', items: TYPE },
    },
  },
});

/** The Messages API request for one attempt. The model is the one the lock pins. */
export function messageRequest(request) {
  return {
    model: request.agent.model,
    max_tokens: 64000,
    betas: ['server-side-fallback-2026-07-01'],
    fallbacks: 'default',
    output_config: { effort: 'xhigh', format: RESPONSE_FORMAT },
    system: SYSTEM,
    messages: [{ role: 'user', content: JSON.stringify(request) }],
  };
}

/** The final message as a `prism/author-response/1` document. A stop that is not a finished answer throws. */
export function responseFromMessage(message) {
  if (message.stop_reason === 'refusal') throw new Error(`the model declined the request (${message.stop_details?.category ?? 'no category'}): ${message.stop_details?.explanation ?? 'no explanation'}`);
  if (message.stop_reason !== 'end_turn') throw new Error(`the model stopped early: ${message.stop_reason}`);
  const text = message.content.filter((block) => block.type === 'text').map((block) => block.text).join('');
  const answer = JSON.parse(text);
  const usage = message.usage ?? {};
  const total = (usage.input_tokens ?? 0) + (usage.cache_creation_input_tokens ?? 0) + (usage.cache_read_input_tokens ?? 0) + (usage.output_tokens ?? 0);
  return { schema: RESPONSE_SCHEMA, ...answer, usage: { total_tokens: total } };
}
