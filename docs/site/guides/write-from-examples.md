---
title: Write from examples
description: The author route, where an agent wrote the model, was removed. Every app now comes from one spec.
available_in: "0.6"
---
`build --unstable-author`, where an agent wrote `model/` from examples, intent and types, was removed. An agent writing the model directly is the one step whose output nothing ties back to what you asked for, so Prism no longer offers it.

Every app now comes from one `prism/spec/1` spec, a JSON document:

1. An agent or a person writes the spec. This is the only step that is not deterministic.
2. Prism checks it against closed value sets, and refuses text it would have to change.
3. Prism renders it to LexLean, the same bytes for the same spec.
4. Lean proves every property on your machine. Nothing is sent anywhere unless you ask for the hosted verifier.
5. Prism builds the holo.

Start it from an agent with the `prism` MCP server, `npx prism-code@latest mcp`, or see [Use with agents](/docs/guides/agents).

## Next steps

- [Use with agents](/docs/guides/agents)
- [Audit a holo](/docs/guides/audit)
