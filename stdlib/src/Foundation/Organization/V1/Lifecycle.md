# Internal organization lifecycle / 1

`Lifecycle.lex.tex` owns the reducer. `LifecycleCorpus.lex.tex` and
`lifecycle-corpus.json` fix the finite native oracle. ST-15 uses the actual
`Administration` model; it does not implement another authorization policy.

## Admission

Inputs are a current authenticated registry partition, an authenticated stable
semantic account reference and a complete content-bound request. Organization,
partition and creation-request references are opaque identities allocated by
their owning boundary. The caller authenticates administrative approvals as
required by `Administration.md`; typed values and matching strings do not
prove authentication, uniqueness across partitions, signatures or mailbox
ownership. Host decoding must bound allocation before constructing these values.

Display names are non-unique and never authority. Any admitted account may
create an organization; no Foundation name, email, user or root is reserved.
One account's eligibility and grants remain local to each organization.

## State and transitions

`emptyOrganizationRegistry` creates revision zero with no organizations.
Each entry retains its creator and unique creation-request reference and embeds
the actual `AdministrationSnapshot`. Registry and embedded organization
identities cannot change. The registry revision equals the sum of one creation
plus the administrative revision of every entry. This is a consistency check,
not proof that a supplied snapshot has authentic history.

`createOrganization` binds account to request actor, partition and current
revision; rejects duplicate organization or creation-request identities; and
calls `createAdministration`. It appends one provisional organization with
one explicit creator grant and increments the registry revision. Existing
entries and their order are unchanged. Creation grants no platform-wide power
and does not claim complete operational ownership.

`administerOrganization` binds the same actor/partition/revision boundary and
the selected organization to the nested request. It calls the existing
`transitionAdministration` on that entry's actual snapshot. It propagates
the exact administrative error or replaces only that entry's snapshot and
increments the registry revision. The submitting actor is not an extra
approver; an authenticated courier may submit an already fully approved
request. Approval, scope inheritance, old-policy quorums, distinct eligible
users and post-change coverage are determined exclusively by `Administration`.

Activation requires complete scoped ownership: at least two distinct eligible
administrators everywhere, any stronger configured minima, and all affected
old-policy quorums. Founding grants may retire through that same scoped
multi-user process only when successor coverage remains sufficient. Retained
creator metadata grants no privilege after retirement. Organization deletion,
partition migration, capacity changes and global account disablement are not
operations of this kernel. No organization-retirement semantics are inferred.

## Bounds and results

Each admitted call handles a partition with capacity 1–64 organizations, not a
global limit on the number of organizations a platform may host. Registry,
account, organization, root and request references are 1–128 UTF-8 bytes;
display names are 1–256. Revisions are 0–4,294,967,295 and cannot wrap. Embedded
plans and approvals retain every `Administration` limit and stronger policy.
Stale outer or inner revisions reject replays; retained creation references
cannot be rebound to another organization in the admitted partition.

The closed result errors are `BadRegistry`, `BadRequest`, `WrongRegistry`,
`StaleRevision`, `RevisionExhausted`, `ActorMismatch`, `DuplicateOrganization`,
`DuplicateCreationRequest`, `RegistryFull`, `UnknownOrganization`,
`WrongOrganization` and `AdministrationRejected(AdministrationError)`.
Denial yields no replacement state. Call-specific checks have the exact order
in the model; nested administrative errors retain their existing order.

## Acceptance scope

The finite oracle checks complete resulting states, all wrapper errors,
partition/reference/revision boundaries, duplicate names, identity/request
uniqueness, cross-organization substitution, independent memberships,
provisional activation, scoped handover, stronger minima and quorums, and
unchanged sibling entries. Owning native verification executes every indexed
Boolean root in generated `std` and `no_std` consumers and audits every selected
declaration. Behavioral mutants bypass identity uniqueness and an actual
administrative approval rejection; each must fail at its exact generated
runtime root, with restored sources reproducing the original artifact identity.

This internal kernel is not an authentication endpoint, distributed registry,
durable CAS, rollback-resistant journal, browser application, compliance proof,
release or deployed organization. Those integration obligations remain separate.
