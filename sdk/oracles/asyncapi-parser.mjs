#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import { dirname } from "node:path";
import { Parser } from "@asyncapi/parser";
import yaml from "js-yaml";

const lockedReferences = new Map([
  [
    "https://www.asyncapi.com/resources/casestudies/adeo/CostingRequestPayload.avsc",
    "/opt/prismpm/share/standards/oracles/asyncapi-website-20a31a03/CostingRequestPayload.avsc",
  ],
  [
    "https://www.asyncapi.com/resources/casestudies/adeo/CostingResponsePayload.avsc",
    "/opt/prismpm/share/standards/oracles/asyncapi-website-20a31a03/CostingResponsePayload.avsc",
  ],
]);

if (process.argv.length !== 3) {
  process.stderr.write("usage: asyncapi-parser INPUT\n");
  process.exit(2);
}

try {
  const path = process.argv[2];
  const input = await readFile(path, "utf8");
  const raw = yaml.load(input);
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    throw new Error("AsyncAPI document must be an object");
  }
  if (raw.asyncapi !== "3.1.0") {
    process.stderr.write("document does not select AsyncAPI 3.1.0\n");
    process.exit(4);
  }
  // The official examples contain relative external references. Resolve them
  // from the document's source directory. The ADEO example also names two
  // mutable website URLs; an earlier resolver serves only those exact locators
  // from byte-locked files. The submitted document is never rewritten.
  process.chdir(dirname(path));
  const parser = new Parser({
    __unstable: {
      resolver: {
        cache: false,
        resolvers: [
          {
            schema: "https",
            order: 1,
            canRead(uri) {
              return lockedReferences.has(uri.toString());
            },
            async read(uri) {
              const local = lockedReferences.get(uri.toString());
              return local === undefined ? undefined : readFile(local, "utf8");
            },
          },
        ],
      },
    },
  });
  const { document, diagnostics } = await parser.parse(input);
  const errors = diagnostics.filter((diagnostic) => diagnostic.severity === 0);
  if (!document || errors.length !== 0) {
    process.stderr.write(`${JSON.stringify(errors)}\n`);
    process.exit(4);
  }
  process.stdout.write("{\"oracle\":\"@asyncapi/parser@3.6.3\",\"valid\":true}\n");
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(4);
}
