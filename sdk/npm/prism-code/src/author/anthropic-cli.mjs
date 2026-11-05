#!/usr/bin/env node
/**
 * The default author agent: one `prism/author-request/1` on stdin, one
 * `prism/author-response/1` on stdout. It needs `@anthropic-ai/sdk`, which
 * prism-code does not depend on, so no install carries it unasked, and
 * credentials the SDK can find (`ANTHROPIC_API_KEY`, or `ant auth login`).
 */
import { messageRequest, responseFromMessage } from './anthropic.mjs';

async function main() {
  let Anthropic;
  try {
    ({ default: Anthropic } = await import('@anthropic-ai/sdk'));
  } catch (cause) {
    throw new Error(`the default author agent needs @anthropic-ai/sdk: run npm install @anthropic-ai/sdk, or pass --author-command (${cause.code ?? cause.message})`);
  }
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const request = JSON.parse(Buffer.concat(chunks).toString('utf8'));
  const client = new Anthropic();
  const message = await client.beta.messages.stream(messageRequest(request)).finalMessage();
  process.stdout.write(JSON.stringify(responseFromMessage(message)));
}

main().catch((error) => {
  process.stderr.write(`${error.message}\n`);
  process.exitCode = 1;
});
