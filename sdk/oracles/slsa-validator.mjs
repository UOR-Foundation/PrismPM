#!/usr/bin/env node
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";

const usage =
  "usage: slsa-validator --provenance FILE --trusted-root FILE --source-uri URI --certificate-identity ID --certificate-oidc-issuer URI SUBJECT\n";

function option(name) {
  const index = process.argv.indexOf(name);
  if (index < 2 || index + 1 >= process.argv.length) throw new Error(`missing ${name}`);
  return process.argv[index + 1];
}

try {
  const provenance = option("--provenance");
  const trustedRoot = option("--trusted-root");
  const sourceUri = option("--source-uri");
  const identity = option("--certificate-identity");
  const issuer = option("--certificate-oidc-issuer");
  const subject = process.argv.at(-1);
  if (!subject || subject.startsWith("--") || process.argv.length !== 13) {
    throw new Error(usage.trim());
  }

  const verification = spawnSync(
    "/usr/local/bin/cosign",
    [
      "verify-blob-attestation",
      "--bundle",
      provenance,
      "--trusted-root",
      trustedRoot,
      "--certificate-identity",
      identity,
      "--certificate-oidc-issuer",
      issuer,
      "--type",
      "slsaprovenance1",
      subject,
    ],
    { encoding: "utf8", maxBuffer: 4_194_304, shell: false },
  );
  if (verification.error) throw verification.error;
  if (verification.status !== 0) {
    process.stderr.write(verification.stderr);
    process.exit(4);
  }

  const bundle = JSON.parse(await readFile(provenance, "utf8"));
  if (
    bundle.mediaType !== "application/vnd.dev.sigstore.bundle+json;version=0.1" &&
    bundle.mediaType !== "application/vnd.dev.sigstore.bundle.v0.3+json"
  ) {
    throw new Error("unsupported Sigstore bundle media type");
  }
  const envelope = bundle.dsseEnvelope ?? bundle.content?.dsseEnvelope;
  if (!envelope || envelope.payloadType !== "application/vnd.in-toto+json") {
    throw new Error("bundle does not contain an in-toto DSSE envelope");
  }
  const statement = JSON.parse(Buffer.from(envelope.payload, "base64").toString("utf8"));
  if (statement.predicateType !== "https://slsa.dev/provenance/v1") {
    throw new Error("provenance is not SLSA v1");
  }
  const subjectDigest = createHash("sha256").update(await readFile(subject)).digest("hex");
  if (
    !Array.isArray(statement.subject) ||
    !statement.subject.some((value) => value?.digest?.sha256 === subjectDigest)
  ) {
    throw new Error("SLSA subject does not bind the measured artifact");
  }
  const builder = statement.predicate?.runDetails?.builder?.id;
  if (builder !== identity) throw new Error("SLSA builder does not match the signer policy");
  const source = statement.predicate?.buildDefinition?.externalParameters?.source?.uri;
  const expectedSource = `git+https://${sourceUri}@`;
  if (typeof source !== "string" || !source.startsWith(expectedSource)) {
    throw new Error("SLSA source does not match the source policy");
  }
  process.stdout.write('{"oracle":"slsa/v1+cosign/3.1.3","valid":true}\n');
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(4);
}
