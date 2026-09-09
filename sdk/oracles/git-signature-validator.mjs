#!/usr/bin/env node
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";

function option(name) {
  const index = process.argv.indexOf(name);
  if (index < 2 || index + 1 >= process.argv.length) throw new Error(`missing ${name}`);
  return process.argv[index + 1];
}

function run(executable, args, options = {}) {
  const result = spawnSync(executable, args, {
    encoding: "utf8",
    maxBuffer: 1_048_576,
    shell: false,
    ...options,
  });
  if (result.error) throw result.error;
  return result;
}

try {
  const format = option("--format");
  const key = option("--trusted-root");
  const fingerprint = option("--fingerprint");
  const signature = option("--signature");
  const subject = process.argv.at(-1);
  if (!subject || subject.startsWith("--") || process.argv.length !== 11) {
    throw new Error("malformed git signature verifier arguments");
  }
  const scratch = await mkdtemp("/scratch/git-signature-");
  if (format === "openpgp") {
    const imported = run("/usr/bin/gpg", [
      "--batch", "--homedir", scratch, "--no-auto-key-retrieve", "--import-options",
      "import-minimal", "--import", key,
    ]);
    if (imported.status !== 0) throw new Error(imported.stderr);
    const verified = run("/usr/bin/gpg", [
      "--batch", "--homedir", scratch, "--no-auto-key-retrieve", "--status-fd", "1",
      "--verify", signature, subject,
    ]);
    const valid = verified.stdout
      .split("\n")
      .some((line) => line.startsWith("[GNUPG:] VALIDSIG ") && line.split(" ")[2] === fingerprint);
    if (verified.status !== 0 || !valid) throw new Error("OpenPGP signature or fingerprint mismatch");
  } else if (format === "ssh") {
    const listed = run("/usr/bin/ssh-keygen", ["-lf", key]);
    if (listed.status !== 0 || !listed.stdout.split(/\s+/).includes(fingerprint)) {
      throw new Error("SSH public-key fingerprint mismatch");
    }
    const allowed = `${scratch}/allowed_signers`;
    await writeFile(allowed, `authority ${(await readFile(key, "utf8")).trim()}\n`, { mode: 0o400 });
    const verified = run(
      "/usr/bin/ssh-keygen",
      ["-Y", "verify", "-f", allowed, "-I", "authority", "-n", "git", "-s", signature],
      { input: await readFile(subject) },
    );
    if (verified.status !== 0) throw new Error("SSH signature mismatch");
  } else {
    throw new Error("unsupported git signature format");
  }
  process.stdout.write('{"oracle":"git-signature-validator/1","valid":true}\n');
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(4);
}
