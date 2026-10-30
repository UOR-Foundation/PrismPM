/**
 * Every error the package throws is a `PrismError`. Each failure kind has its
 * own subclass with a stable `code`, so a caller branches on the class or the
 * code and never parses a message.
 *
 * - `PrismCodeError`: load, dispatch and archive failures. Its `code` names
 *   the check that failed (`archive.magic`, `core.exports`, `closure.provenance`, ...).
 * - `VerificationError`: no verdict could be reached, or the holo failed a
 *   check that is about the archive itself, its documents or its provenance.
 * - `ProofMismatchError`: a `proof.*` check of a verdict failed.
 * - `CapabilityDeniedError`: the declared world exceeds the granted capabilities,
 *   or the runtime denied a guest import or a resource (`capabilities.*`, issue #279).
 * - `WorldMismatchError`: the guest does not have the interface of the WIT world it declares,
 *   or the world is not the one the caller pinned (issue #278).
 * - `DeterminismViolationError`: a replay diverged from its recorded history.
 * - `ReplayError`: a replay could not be carried out or its statement failed.
 * - `SigningError`: `attest` could not produce a signature (#272).
 * - `SignatureInvalidError`, `IdentityMismatchError`, `TransparencyError`,
 *   `SubjectMismatchError`, `AttestationNotFoundError`, `TrustRootError`: the
 *   reasons `verify-attestation` can refuse a bundle (#272).
 *
 * The last five map one-to-one to the check ids of a `prism/verdict/1`
 * reason (docs/proposals/269-assurance-levels.md); `errorForReason` is the map.
 */
export class PrismError extends Error {
  /**
   * @param {string} message
   * @param {{ code?: string, check?: string, diagnostic?: string, details?: unknown, verdict?: object, reasons?: object[], cause?: unknown }} [options]
   */
  constructor(message, options = {}) {
    super(message, options.cause === undefined ? undefined : { cause: options.cause });
    this.name = new.target.name;
    this.code = options.code ?? new.target.code;
    if (options.check !== undefined) this.check = options.check;
    if (options.diagnostic !== undefined) this.diagnostic = options.diagnostic;
    if (options.details !== undefined) this.details = options.details;
    if (options.verdict !== undefined) this.verdict = options.verdict;
    if (options.reasons !== undefined) this.reasons = options.reasons;
  }
}
PrismError.code = 'prism_error';

/**
 * One error type for load, dispatch and archive failures. `code` names the
 * check that failed (`archive.magic`, `core.exports`, `closure.provenance`, ...).
 */
export class PrismCodeError extends PrismError {
  constructor(code, message, details = undefined) {
    super(message, { code, details });
  }
}

/** No verdict was reached, or an archive, document or provenance check failed. `check` names it. */
export class VerificationError extends PrismError {}
VerificationError.code = 'verification_failed';

/** A `proof.*` check failed: the proof statement is absent, malformed, or does not belong to this holo. */
export class ProofMismatchError extends PrismError {}
ProofMismatchError.code = 'proof_mismatch';

/**
 * The world the holo declares exceeds the capabilities granted to it
 * (`world.capabilities`), or the runtime denied what the grant does not allow:
 * a guest import (`capabilities.import`) or a resource
 * (`capabilities.outbound-hosts`, `capabilities.key-value-stores`,
 * `capabilities.variables`). `details` names the import or resource and the set.
 */
export class CapabilityDeniedError extends PrismError {}
CapabilityDeniedError.code = 'capability_denied';

/**
 * The guest does not have the imports and exports of its declared world
 * (`world.contract`), or the declared world differs from the world hash the
 * caller pinned (`world.hash`). `check` says which.
 */
export class WorldMismatchError extends PrismError {}
WorldMismatchError.code = 'world_mismatch';

/** A replay produced output that differs from the recorded history. */
export class DeterminismViolationError extends PrismError {}
DeterminismViolationError.code = 'determinism_violation';

/** A replay could not be carried out, or the holo's replay statement failed. */
export class ReplayError extends PrismError {}
ReplayError.code = 'replay_failed';

/**
 * `attest` could not produce a signature: the key is unusable, no identity token
 * was available, or Fulcio, Rekor or the timestamp authority refused. `check`
 * is `sign.key`, `sign.identity`, `sign.service`, `sign.config`, `sign.predicate`
 * or `sign.subject`.
 */
export class SigningError extends PrismError {}
SigningError.code = 'signing_failed';

/** The DSSE signature, the signing key or the certificate chain of a bundle does not verify. */
export class SignatureInvalidError extends PrismError {}
SignatureInvalidError.code = 'signature_invalid';

/** The signer is valid but is not the identity (or issuer) the policy names. */
export class IdentityMismatchError extends PrismError {}
IdentityMismatchError.code = 'identity_mismatch';

/** A transparency log entry or signed timestamp is missing, untrusted or not about this bundle. `reasons` lists each one. */
export class TransparencyError extends PrismError {}
TransparencyError.code = 'transparency_failed';

/** A statement subject is not a content blob of this holo, or its digests differ from the blob's. */
export class SubjectMismatchError extends PrismError {}
SubjectMismatchError.code = 'subject_mismatch';

/** The holo carries no attestation, or none of the requested predicate type. */
export class AttestationNotFoundError extends PrismError {}
AttestationNotFoundError.code = 'attestation_not_found';

/** The pinned trusted root cannot be read or is not a usable root. */
export class TrustRootError extends PrismError {}
TrustRootError.code = 'trust_root_invalid';

/** Every class with a code of its own, by code. */
export const ERROR_CLASSES = Object.freeze({
  [VerificationError.code]: VerificationError,
  [ProofMismatchError.code]: ProofMismatchError,
  [CapabilityDeniedError.code]: CapabilityDeniedError,
  [WorldMismatchError.code]: WorldMismatchError,
  [DeterminismViolationError.code]: DeterminismViolationError,
  [ReplayError.code]: ReplayError,
  [SigningError.code]: SigningError,
  [SignatureInvalidError.code]: SignatureInvalidError,
  [IdentityMismatchError.code]: IdentityMismatchError,
  [TransparencyError.code]: TransparencyError,
  [SubjectMismatchError.code]: SubjectMismatchError,
  [AttestationNotFoundError.code]: AttestationNotFoundError,
  [TrustRootError.code]: TrustRootError,
});

/**
 * The class a verdict reason maps to, by its check id. `replay.divergence` is
 * the one replay check that is a determinism violation; every other `replay.*`
 * check is a replay error. Anything else, `holo.*`, `provenance.*` and
 * `policy.*`, is a verification error.
 */
export function classForCheck(check) {
  if (check.startsWith('proof.')) return ProofMismatchError;
  if (check === 'world.capabilities' || check.startsWith('capabilities.')) return CapabilityDeniedError;
  if (check === 'world.contract' || check === 'world.hash') return WorldMismatchError;
  if (check === 'replay.divergence') return DeterminismViolationError;
  if (check.startsWith('replay.')) return ReplayError;
  return VerificationError;
}

/** The typed error for one reason of a verdict, carrying the reason's check id and diagnostic code. */
export function errorForReason(reason, verdict = undefined) {
  const Class = classForCheck(reason.check);
  return new Class(reason.message, {
    check: reason.check,
    diagnostic: reason.code,
    verdict,
    reasons: verdict ? verdict.reasons : [reason],
  });
}

/**
 * The error that explains a failed verdict, or null for one that passed. A
 * rule the caller pinned and the holo broke (a digest, the world) comes
 * first: the caller asked for that fact by name, and it holds whatever the
 * level is. A world outside the granted capabilities (`world.capabilities`)
 * comes next: the default policy requires it at every level (#283). Otherwise, when the level is below the policy's minimum, it is
 * the first reason that withholds a level up to the minimum, which is what
 * kept the holo down. The error carries the whole verdict and every reason.
 */
export function errorFromVerdict(verdict, ranks) {
  if (verdict.passed) return null;
  const minimum = ranks[verdict.policy.minimum];
  const pinned = verdict.reasons.find((r) => r.withholds === null && r.check !== 'policy.level');
  // The default policy requires the world within the grant at every level (#269, #283), so it is named before a level shortfall.
  const outside = verdict.reasons.find((r) => r.check === 'world.capabilities');
  const blocking = verdict.reasons.find((r) => r.withholds && ranks[r.withholds] <= minimum);
  const reason = pinned ?? outside ?? blocking ?? verdict.reasons.find((r) => r.check !== 'policy.level') ?? verdict.reasons[0];
  return errorForReason(reason, verdict);
}

/**
 * What a run exits with, one row per outcome: the single source of truth for
 * container exit codes and the `prism-code` process (issue #348). The table is
 * in docs/site/guides/run-a-holo.md, and a test holds the two equal. A row
 * names an error class, or none for success, and `when` narrows it. The first
 * row that matches decides.
 *
 * `check` reports a verdict, not an error, so its own contract stays 0, 1, 2.
 * 1 to 3 keep the meanings the command line already gave them. 10 to 13 are
 * new. 64 to 78 are the BSD sysexits and 125 and above are Docker's and the
 * shell's, so no typed error takes those, apart from `lock.*` as usage (64).
 */
export const EXIT_CODES = Object.freeze(
  [
    { exit: 0, error: null, meaning: 'The guest returned its output.' },
    { exit: 1, error: 'VerificationError', when: 'with a verdict', meaning: 'The holo failed check, for an archive, document, provenance or policy reason.' },
    { exit: 2, error: 'VerificationError', when: 'no verdict', meaning: 'Check could not reach a verdict: an unreadable policy, a closure, an unknown format revision.' },
    { exit: 2, error: 'PrismCodeError', meaning: 'The artifact was refused, or dispatch refused the request: a bad archive, a request over the guest allocation.' },
    { exit: 2, error: 'CapabilityDeniedError', meaning: 'The world exceeds the granted capabilities, or the runtime denied an import or a resource.' },
    { exit: 3, error: 'SigningError', meaning: 'A signature could not be produced.' },
    { exit: 10, error: 'ProofMismatchError', meaning: 'A proof check failed: the statement is absent, malformed, or about another holo.' },
    { exit: 11, error: 'WorldMismatchError', meaning: 'The guest does not have the interface of its declared world, or the world is not the one pinned.' },
    { exit: 12, error: 'DeterminismViolationError', meaning: 'A replay produced output that differs from the recorded history.' },
    { exit: 13, error: 'ReplayError', meaning: 'A replay could not be carried out, or the replay statement failed.' },
    { exit: 1, error: 'SignatureInvalidError', meaning: 'An attestation did not verify: its signature.' },
    { exit: 1, error: 'IdentityMismatchError', meaning: 'An attestation did not verify: its signer.' },
    { exit: 1, error: 'TransparencyError', meaning: 'An attestation did not verify: its log entry or timestamp.' },
    { exit: 1, error: 'SubjectMismatchError', meaning: 'An attestation did not verify: its subject.' },
    { exit: 1, error: 'AttestationNotFoundError', meaning: 'An attestation did not verify: there is none.' },
    { exit: 1, error: 'TrustRootError', meaning: 'An attestation did not verify: the trusted root is unusable.' },
    { exit: 1, error: 'PrismError', meaning: 'Any other typed error.' },
    { exit: 70, error: 'anything else', meaning: 'Dispatch threw something untyped: a guest trap, or a bug in the runtime.' },
  ].map(Object.freeze),
);

const classNamed = (name) => (name === 'PrismError' ? PrismError : name === 'PrismCodeError' ? PrismCodeError : Object.values(ERROR_CLASSES).find((C) => C.name === name));

/**
 * The exit code for how a run ended: `exitCodeFor(null)` is 0, a typed error
 * takes its row of `EXIT_CODES`, and anything else is 70. A `lock.*` code is
 * the one `PrismCodeError` that is the caller's to fix, so it is usage (64),
 * and `attest.unavailable` means no signature could be produced (3).
 */
export function exitCodeFor(error) {
  if (error === null || error === undefined) return 0;
  if (!(error instanceof PrismError)) return 70;
  if (error instanceof PrismCodeError && error.code.startsWith('lock.')) return 64;
  if (error.code === 'attest.unavailable') return 3;
  return EXIT_CODES.find((row) => {
    const Class = row.error === null ? undefined : classNamed(row.error);
    if (!Class || !(error instanceof Class)) return false;
    if (row.when === 'with a verdict') return error.verdict !== undefined;
    if (row.when === 'no verdict') return error.verdict === undefined;
    return true;
  }).exit;
}

export function fail(code, message, details) {
  throw new PrismCodeError(code, message, details);
}
