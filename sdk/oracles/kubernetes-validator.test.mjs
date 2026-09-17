import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const validator = fileURLToPath(new URL("./kubernetes-validator.mjs", import.meta.url));
const service = {
  apiVersion: "v1", kind: "Service", metadata: { name: "fixture" },
  spec: { ports: [{ port: 8080, targetPort: 8080 }], selector: { app: "fixture" } },
};
const pod = {
  apiVersion: "v1", kind: "Pod", metadata: { name: "fixture" },
  spec: { containers: [{ name: "app", image: "example.test/app:fixture" }] },
};
const list = items => ({ apiVersion: "v1", kind: "List", items });

function check(document, accepted) {
  const root = mkdtempSync(join(tmpdir(), "prism-kubernetes-oracle-"));
  try {
    const input = join(root, "input.json");
    writeFileSync(input, typeof document === "string" ? document : JSON.stringify(document));
    const result = spawnSync(process.execPath, [validator, input], {
      encoding: "utf8", timeout: 120_000, maxBuffer: 4 * 1024 * 1024,
    });
    assert.ifError(result.error);
    assert.equal(result.signal, null);
    if (accepted) {
      assert.equal(result.status, 0, result.stderr);
      assert.equal(JSON.parse(result.stdout).valid, true);
    } else {
      assert.equal(result.status, 4, result.stderr);
      assert.equal(result.stdout, "");
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("single resources retain official OpenAPI validation", () => {
  check(service, true);
  check({ ...pod, spec: { containers: "invalid" } }, false);
});

test("mixed and nested collections validate every resource", () => {
  check(list([service, pod]), true);
  check(list([service, list([pod])]), true);
  check(list([service, { ...pod, spec: { containers: "invalid" } }]), false);
  check(list([service, { apiVersion: "v1", kind: "NotAResource" }]), false);
});

test("collections reject missing, malformed, and empty contents", () => {
  for (const document of [
    { apiVersion: "v1", kind: "List" }, list(null), list({}), list([]),
    list([null]), list([17]), list([[]]), { ...list([service]), metadata: 3 },
  ]) check(document, false);
});

test("document streams stay complete and cyclic aliases fail", () => {
  check(JSON.stringify(service) + "\n---\n" + JSON.stringify(pod), true);
  check("&cycle\napiVersion: v1\nkind: List\nitems: [*cycle]\n", false);
});
