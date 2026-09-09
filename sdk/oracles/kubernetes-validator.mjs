#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import AjvDraft4 from "ajv-draft-04";
import yaml from "js-yaml";

function normalizeKubernetesOpenApi(node) {
  if (Array.isArray(node)) {
    for (const value of node) normalizeKubernetesOpenApi(value);
    return;
  }
  if (node === null || typeof node !== "object") return;
  for (const value of Object.values(node)) normalizeKubernetesOpenApi(value);
  if (node["x-kubernetes-int-or-string"] === true || node.format === "int-or-string") {
    delete node.type;
    delete node.format;
    node.anyOf = [{ type: "integer" }, { type: "string" }];
  }
  if (node.nullable === true && typeof node.type === "string") {
    node.type = [node.type, "null"];
  }
  if (node["x-kubernetes-preserve-unknown-fields"] === true) {
    node.additionalProperties = true;
  }
}

if (process.argv.length !== 3) {
  process.stderr.write("usage: kubernetes-validator INPUT\n");
  process.exit(2);
}

try {
  const swagger = JSON.parse(
    await readFile("/opt/prismpm/share/standards/oracles/kubernetes-1.36.4-swagger.json", "utf8"),
  );
  const documents = [];
  yaml.loadAll(await readFile(process.argv[2], "utf8"), (document) => {
    if (document !== undefined && document !== null) documents.push(document);
  });
  if (documents.length === 0) throw new Error("manifest contains no resources");
  const definitions = swagger.definitions ?? {};
  normalizeKubernetesOpenApi(definitions);
  const gvks = new Map();
  for (const [name, definition] of Object.entries(definitions)) {
    for (const gvk of definition["x-kubernetes-group-version-kind"] ?? []) {
      const apiVersion = gvk.group ? `${gvk.group}/${gvk.version}` : gvk.version;
      gvks.set(`${apiVersion}\u0000${gvk.kind}`, name);
    }
  }
  const ajv = new AjvDraft4({ allErrors: true, strict: false, validateFormats: false });
  for (const document of documents) {
    if (typeof document !== "object" || Array.isArray(document)) {
      throw new Error("each manifest document must be an object");
    }
    const definition = gvks.get(`${document.apiVersion}\u0000${document.kind}`);
    if (!definition) throw new Error(`unsupported Kubernetes GVK ${document.apiVersion}/${document.kind}`);
    const validate = ajv.compile({ definitions, $ref: `#/definitions/${definition}` });
    if (!validate(document)) {
      process.stderr.write(`${JSON.stringify(validate.errors)}\n`);
      process.exit(4);
    }
  }
  process.stdout.write('{"oracle":"kubernetes/openapi@1.36.4","valid":true}\n');
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(4);
}
