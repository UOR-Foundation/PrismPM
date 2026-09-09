#!/usr/bin/env node
import { readFile, stat } from "node:fs/promises";
import Ajv2019 from "ajv/dist/2019.js";
import { parse, printParseErrorCode } from "jsonc-parser";

if (process.argv.length !== 3) {
  process.stderr.write("usage: devcontainer-validator INPUT\n");
  process.exit(2);
}

try {
  const inputStat = await stat(process.argv[2]);
  const input = inputStat.isDirectory()
    ? `${process.argv[2]}/.devcontainer/devcontainer.json`
    : process.argv[2];
  const parseErrors = [];
  const document = parse(await readFile(input, "utf8"), parseErrors, {
    allowTrailingComma: true,
    disallowComments: false,
  });
  if (parseErrors.length !== 0) {
    throw new Error(
      `invalid devcontainer JSONC: ${parseErrors
        .map(({ error, offset }) => `${printParseErrorCode(error)}@${offset}`)
        .join(",")}`,
    );
  }
  const schema = JSON.parse(
    await readFile("/opt/prismpm/share/standards/oracles/devcontainer-c95ffeed-base.schema.json", "utf8"),
  );
  const ajv = new Ajv2019({ allErrors: true, strict: false, validateFormats: false });
  const validate = ajv.compile(schema);
  if (!validate(document)) {
    process.stderr.write(`${JSON.stringify(validate.errors)}\n`);
    process.exit(4);
  }
  process.stdout.write('{"oracle":"devcontainers/spec@c95ffeed1d05","valid":true}\n');
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(4);
}
