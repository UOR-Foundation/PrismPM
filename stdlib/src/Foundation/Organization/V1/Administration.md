# Internal scoped administration / 1

`Administration.lex.tex` owns the typed semantics. `Corpus.lex.tex` owns the
finite native acceptance vectors; `corpus.json` fixes their complete root set.
ST-12 compiles both with LexLean and lean4-prod and executes the generated
package with and without `std`. No handwritten Lean or native reducer exists.

## Admission boundary

These are internal pure transitions, not a public authorization endpoint.
The caller must supply a current, authenticated organization snapshot and
independently authenticated approvals over the complete request. The request
and each approval bind organization, revision, request reference, proposal
reference and the exact complete proposed plan. Comparing those fields does
not authenticate them. The proposal reference is the authenticated boundary's
content-reference witness, not a caller-chosen substitute for the plan bytes.

Canonical semantic user references identify users independently of keys and
aliases. Authentication must resolve credentials to these references before
admission. Neither a boolean, an email address, another key nor a self-declared
role supplies authority. A user's enabled flag is organization-local eligibility;
it cannot disable that person's platform identity or membership in another
organization. Global single-copy admission, signatures, credential
recovery, durable CAS, rollback resistance and network agreement remain separate
integration obligations. This prerequisite claims none of them.

## State and transitions

`createAdministration` creates one provisional organization, revision zero,
one root scope and one explicit creator grant. Organization and creator
references must already be authenticated/allocated by their owning boundary.
Names are bounded display data; identical names confer no shared identity or
privilege. There are no reserved names, mailboxes, organizations or global users.

`transitionAdministration` validates current state, request bounds and binding,
then computes affected scopes from both ancestry graphs. Old policy and old
eligible administrators authorize each affected scope. A new scope or moved
parent additionally needs the old policy of its nearest existing destination
ancestor. Changes to an ancestor require the affected descendant policies too;
an administrator of the root is not automatically an administrator elsewhere.
An inherited grant counts its active semantic user once, including when the
same user also has a direct grant. Display-only and otherwise scope-inert plan
changes require root-scope approval. Proposed lower quorums never authorize
their own adoption. Duplicate, irrelevant and substituted approvals fail.
Membership/eligibility changes for users with no old or proposed grant require
root approval independently, including when bundled with a scoped grant edit.
Row order does not confer authority; an otherwise inert reordering still uses
the root-scope approval rule.

The reducer owns revision increment. Organization and root references cannot
change; active state cannot return to provisional. Existing scopes cannot be
omitted: deletion requires a separately modeled object-lifecycle protocol.
User/grant changes cannot leave dangling references. Scope graphs have exactly
one root, resolved parents and no cycles. Grants inherit only through scopes
that explicitly enable inheritance; policy minima and quorums are local.

Every provisional scope retains at least one active effective administrator
and an attainable quorum. Activation and active changes require at least two
active administrators per scope, every stronger configured minimum and an
attainable quorum. Destructive provisional changes require those full minima
in every affected scope; unrelated provisional scopes retain their provisional
minimum. Destructive means removing a user, disabling
an enabled user, revoking a grant, or editing an existing scope. Provisional
setup cannot claim accepted redundancy. Founding grants retire by the same
scoped approvals and post-change coverage as any others; different successors
may cover different parts. Denial returns only a typed error, never a changed
state. Recovery cannot invoke this reducer to revive grants without approval.

## Finite limits and errors

Admitted plans contain at most 64 users, 64 scopes and 256 grants. A request
contains at most 64 approvals. References contain 1–128 UTF-8 bytes; display
labels contain 1–256. Policy minima are 2–64 and quorums 1–64. Revisions range
from zero through 4,294,967,295; the maximum cannot advance. Parent traversal
is bounded by the 64-scope limit. The host must bound decoding/allocation
before constructing these internal typed values; this is not a wire decoder.

`AdministrationError` is closed: `BadState`, `ResourceLimit`,
`WrongOrganization`, `StaleRevision`, `RevisionExhausted`, `BadRequest`,
`RootChanged`, `StageRollback`, `ScopeRemoval`, `BadProposal`, `NoChange`,
`BadApproval`, `DuplicateApproval`, `OutOfScopeApproval`,
`InsufficientApproval`, `CoverageLoss`. These internal results are not new
public CLI diagnostics. Checks occur in this listed order.

The finite corpus includes combined admitted resource maxima, each rejection,
provisional/active transitions, inherited and isolated scopes, both ancestry
graphs, distinct-user quorums, stronger minima, concurrent stale requests and
partial/distributed founder retirement. ST-12 separately weakens the real retained
minimum and removes the ungranted-membership approval guard. Each defect must
fail at its exact modeled corpus root in generated execution, not compilation;
restoring the source must reproduce the original accepted artifact identities.
It does not establish human uniqueness, availability, browser execution, release readiness,
OSCAL coverage or a completed organizational application.
