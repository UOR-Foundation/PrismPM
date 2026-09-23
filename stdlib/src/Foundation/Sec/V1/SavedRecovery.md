# Saved recovery lifecycle

`SavedRecovery.lex.tex` owns two INTERNAL pure transitions: recover a credential
using admitted saved-code evidence, and replace saved codes using admitted
current-credential authorization. Neither is a public authentication endpoint.
Generated record constructors are ordinary data constructors, not unforgeable
evidence. No raw-input adapter is provided.

## Required admission boundary

The integrating verifier must authenticate the complete request and code
commitment, prove possession of the replacement credential, and enforce its
recovery assurance and throttling policy. Fresh replacement codes require
approved randomness, protected delivery and approved one-way verification
storage. Commitments here are opaque references, never plaintext secrets or
proof of cryptographic correctness. Admission must also reject previously
issued secrets even if their verifier encoding could differ; merely choosing
a new label or salt is not fresh issuance. No email ownership is inferred.

State must be the current authenticated snapshot, not a recovered backup.
Account, scope, credential revision, recovery epoch, current credential and
current authorization reference are bound to the admitted request. The storage
adapter must atomically compare that complete current state and commit the
entire returned state plus notification intent. Concurrent losers must reread
and revalidate; a returned value is not a durability or notification receipt.

## Transitions

Recovery consumes exactly the selected active code, installs the bound new
credential, increments revision and recovery epoch, appends one fresh active
replacement code, and emits a bound notification intent. Other slots are
unchanged. Remaining active codes require fresh admission at the new epoch.

Authenticated replacement retires every previous slot, appends a nonempty fresh
inventory, increments only recovery epoch, and emits a notification intent.
It can issue the first inventory for an already authenticated account; it does
not create an account or organization. Both transitions preserve the current
authorization reference exactly. There are no role, grant or quorum outputs.

All rejections return only a typed error. Consumed slots remain tombstones;
duplicate references or commitments, including consumed ones, are rejected.
No implicit history pruning or counter rollover is permitted.

Engineering bounds: opaque references 1–128 bytes; configured retained-slot
capacity 1–256; revision and recovery epoch 0–4,294,967,295. These are finite
implementation bounds, not standards-prescribed policy. Exhaustion fails
closed without deleting evidence. Unlimited account lifetime, retention
migration, resource-safe public decoding and storage are not provided here.

## Standards and evidence

[NIST SP 800-63B-4](https://pages.nist.gov/800-63-4/sp800-63b.html), §4.2.1.1,
requires saved recovery codes with at least 64 bits from an approved random
generator, approved one-way storage, throttled verification (§3.2.2), and
invalidation followed by replacement after use. Replacement issuance and
recovery require notification (§§4.2.3, 4.6). Those cryptographic, operational
and delivery requirements remain adapter obligations. Assurance-level recovery
rules in §4.2.2 are separate: this primitive does not establish AAL2/AAL3 or
complete NIST conformance.

ST-11 executes the modeled corpus through independently regenerated native
`std` and `no_std` packages, with complete selected-source declaration audit
and planted behavioral defects. This is finite transition evidence, not a
deployed account service, mailbox proof, network or production release.
