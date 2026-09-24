# Internal mailbox admission / 1

`MailboxAdmission.lex.tex` owns a pure admission transition. It consumes an
already authenticated, profile-specific assertion and a current authenticated
account, authority policy and pending challenge. Generated constructors are
ordinary data constructors, not proof. There is no raw-input adapter, caller
verification boolean, token parser, signature verifier or mailbox service here.

## Admission and authority

The integrating verifier must establish the assertion's authenticity under the
approved provider profile, exact issuer, audience, subject and signing-key
configuration. It must verify the signed nonce commitment to the complete
modeled intent, prove possession of the candidate credential, and enforce the
operation's assurance and throttling policy. References are opaque bindings;
equality does not authenticate their contents. Trusted current time, canonical
encoding, entropy, signatures, provider discovery and key rotation are separate
verified effects. No provider registration is implied or supplied.

`ProfileSpecificCurrentControl` means only the current-control fact established
by that separately adopted and verified profile. It is not a generic property
of OIDC tokens. Assertion issuance freshness is not a fresh user-authentication
or mailbox-read event; stronger assurance must come from the adopted profile.
[OIDC Core](https://openid.net/specs/openid-connect-core-1_0.html)
defines issuer, subject, audience, nonce and time validation; `email_verified`
does not alone establish present control. [Google's authority conditions](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token)
distinguish hosted accounts from historical verification of external addresses.
[Microsoft's email claim](https://learn.microsoft.com/en-us/entra/identity-platform/id-token-claims-reference)
is mutable and not guaranteed correct. Those provider integrations are not
implemented by this kernel. Historical-address proof is rejected even if both
caller-supplied labels and the account's address agree.

The authority binding fixes installation, authority, configuration, key and
profile references; issuer and audience; and authority/configuration/key
revisions. Both challenge and assertion must match the current policy exactly.
Retired policies and disallowed operations fail. The policy itself must be
authenticated and authorized independently; selecting a profile is not proof.

## State transition

The four operations are enrollment, login, recovery and mailbox replacement.
Accounts already have allocated stable identities and credentials. Enrollment
requires an unbound account. Both enrollment and replacement require prior
current-credential authorization of the complete stored challenge. Authentic
stored state alone is not caller authorization, and mailbox proof alone cannot
authorize either challenge. Login and recovery require the existing exact
installation/issuer/subject/mailbox binding; replacement also requires an existing
binding. Challenge issuance and account allocation are separate transitions,
not implicit side effects.

The assertion binds account, revision, credential epoch, current credential,
authorization reference, operation, mailbox, candidate credential, challenge,
nonce commitment and challenge timing. Future or pre-challenge assertions,
expired assertions/challenges, stale proofs and all substitutions fail.
Mailboxes and identifiers are compared as exact bytes after profile-defined
validation. No local-part case folding, alias equivalence or syntax validation
is inferred here.

Admission increments the account revision and retains the challenge as consumed.
Only recovery changes the credential and increments its epoch. Enrollment and
replacement update the mailbox binding; login returns a bound session intent.
Every operation preserves the authorization reference. There are no organization,
role, grant or quorum outputs, and no recovery root bypass. The returned effect
binds old/new credential and mailbox state for session invalidation and required
notifications; it is not an execution or delivery receipt.

The store must atomically compare the complete current state and authority
configuration and commit state plus effects. A stale replica or two pure calls
against one old snapshot do not establish global single-use. Rollback resistance,
retention/migration, durable admission, notifications and network availability
remain integration requirements. Consumed challenges cannot be reopened through
this API. Rejections contain only a typed error, never a replacement state.

## Bounds and evidence

Engineering bounds, not standards-prescribed limits: opaque references 1–128
UTF-8 bytes; issuer/audience 1–512; subject 1–255; mailbox 1–320. Policies contain
1–4 distinct operations and a maximum proof age of 1–3,600 seconds. Challenges
live at most 3,600 seconds. Revisions, epochs and time values are 0–4,294,967,295.
Counter exhaustion fails closed; admitted increments never wrap.
The decoder must bound allocation before constructing internal values.

ST-13 executes the complete finite corpus through independently regenerated
native `std`/`no_std` packages and audits every selected declaration. It tests
both proof-class and candidate-key defects at actual generated runtime roots,
then reproduces the restored artifact identities. Synthetic admitted records
are kernel test inputs, not real mailbox proofs. This is not arbitrary-email
coverage, NIST authentication/recovery conformance, browser or release acceptance.
