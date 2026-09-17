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
  const validators = new Map();
  function validateDefinition(document, definition) {
    let validate = validators.get(definition);
    if (!validate) {
      validate = ajv.compile({ definitions, $ref: `#/definitions/${definition}` });
      validators.set(definition, validate);
    }
    if (!validate(document)) throw new Error(JSON.stringify(validate.errors));
  }
  const active = new WeakSet();
  const pending = documents.toReversed().map(document => ({ document, leave: false }));
  let resources = 0;
  while (pending.length) {
    const { document, leave } = pending.pop();
    if (leave) {
      active.delete(document);
      continue;
    }
    if (document === null || typeof document !== "object" || Array.isArray(document)) {
      throw new Error("each manifest document must be an object");
    }
    // Generic List is a client-side collection, not a server OpenAPI GVK.
    // Unpack it without skipping or weakening validation of any contained resource.
    if (document.apiVersion === "v1" && document.kind === "List") {
      if (!Array.isArray(document.items)) throw new Error("Kubernetes List items must be an array");
      if (active.has(document)) throw new Error("cyclic Kubernetes List");
      if (Object.hasOwn(document, "metadata")) {
        validateDefinition(document.metadata, "io.k8s.apimachinery.pkg.apis.meta.v1.ListMeta");
      }
      active.add(document);
      pending.push({ document, leave: true });
      for (const item of document.items.toReversed()) pending.push({ document: item, leave: false });
      continue;
    }
    const definition = gvks.get(`${document.apiVersion}\u0000${document.kind}`);
    if (!definition) throw new Error(`unsupported Kubernetes GVK ${document.apiVersion}/${document.kind}`);
    validateDefinition(document, definition);
    resources += 1;
  }
  if (resources === 0) throw new Error("manifest contains no resources");
  process.stdout.write('{"oracle":"kubernetes/openapi@1.36.4","valid":true}\n');
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(4);
}
