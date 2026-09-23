# Candidate browser bootstrap lifecycle

`BrowserBootstrap.lex.tex` is an INTERNAL pure candidate policy kernel. It is
not a transport implementation, public input decoder, selected Kappa profile,
or production acceptance. DK-09's manual direct-channel adapter is unchanged.
The kernel cannot attest that a public endpoint exists or that a browser is
connected. Generated constructors are ordinary data, not cryptographic proof.

## Admission boundary

The caller supplies the current authenticated snapshot, approved policy and
trusted monotonic time. Policy references bind an immutable operator inventory
and disclosure document: endpoint identities and secure protocols, capabilities,
addresses/peer identifiers/timing/volume visible to each party, retention,
costs, quotas, terms and availability. Public operators may provide signaling
and encrypted transport only, not Foundation application execution or durable
Foundation storage. No operator or endpoint is selected by this candidate.

User consent is a separate admitted fact bound to policy, disclosure, local
principal and expiration. Consent to operator metadata does not authorize
direct-peer address disclosure. Neither consent nor a public endpoint claim
authenticates a peer. The integrating verifier must separately validate a
domain-separated peer authorization over the complete profile, policy, peer
pair, epoch, attempt, session, challenge, transcript and both DTLS fingerprints.
It must bind actual browser channel observations and authenticated relay
reservation responses to that same tuple. A supplied reference or `proofRef`
is not verification of a signature, TLS certificate, DTLS handshake or relay.

Raw SDP/ICE/signaling parsing, signature and certificate checks, key-to-peer
reference derivation, entropy and freshness, endpoint discovery and network
effects remain mandatory adapter obligations. Self-signed DTLS encryption
without the admitted peer binding is insufficient. Current time and epoch
must survive rollback/restart safely; this kernel does not provide a clock.

## Lifecycle

Creation establishes an idle session with attempt zero and an initial reserved
session/challenge pair. Begin admits separate consent, consumes one attempt,
retains the fresh pair, and enters connecting. Connection admission binds
signed authorization, observed encrypted/ordered/reliable channel facts and,
for relayed mode only, an authenticated bounded reservation. Direct mode
rejects relay reservations. Only a connected, unexpired session may admit
bounded message bytes; admission is not delivery or durable acknowledgment.

Signaling loss interrupts connecting but does not tear down an already
connected direct peer channel. Loss of the relay interrupts connecting or
connected relayed sessions. Peer loss interrupts either mode. Interrupted
sessions enter backoff and reject messages. Restoring operator availability
does not restore a channel: a new bounded attempt and fresh admission are
required. Retry exhaustion and close are fail-closed. No availability claim
is inferred from a lease or a previously reachable operator.

Every successful transition advances the revision, preserves the complete
policy, epoch, peer pair and pending-operations reference, and retains session
and challenge history. No transition deletes, acknowledges, retries or commits
durable operations. The caller must atomically compare and replace the full
current snapshot; concurrent losers reread and revalidate. Failed transitions
return only a typed error. Transport loss must not delete pending operations.

Engineering bounds, not standards prescriptions: references 1–128 UTF-8 bytes;
attempt budget 1–16; retry delay and lease duration 1–3,600 seconds; message and
reservation budgets 1–1,048,576 bytes; epoch, revision and admitted time within
0–4,294,967,295. No counter wrap or implicit history pruning is allowed.

## Standards and finite evidence

[W3C WebRTC, Recommendation 13 March 2025](https://www.w3.org/TR/2025/REC-webrtc-20250313/),
§4.1 leaves signaling to the application; §§4.2–4.3 define configuration and
connection states, §6 data channels, and §13 address/metadata disclosure and
security considerations. [RFC 8831](https://www.rfc-editor.org/rfc/rfc8831.html)
defines SCTP/DTLS data channels. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445.html)
defines ICE connectivity establishment. [RFC 8827](https://www.rfc-editor.org/rfc/rfc8827.html),
§§6–7, separates signaling/identity trust from transport security.
[RFC 8656](https://www.rfc-editor.org/rfc/rfc8656.html), §§7–8, specifies TURN
allocation and refresh lifetimes; an application lease does not assert that
a TURN allocation exists. This candidate does not invent a signaling wire
standard, implement ICE/TURN, or equate a relay reservation with availability.

ST-14's finite oracle checks this declared composition's transitions through
actual generated native `std` and `no_std` packages, complete selected-source
declaration audits and planted behavioral defects. It is not the authoritative
WebRTC implementation test suite, complete RFC conformance, browser/network
interoperability, or a Kappa replication oracle. The separate
[W3C-linked Web Platform Tests](https://github.com/web-platform-tests/wpt/tree/master/webrtc/)
and protocol/independent-peer tests remain required for an eventual adapter.
There is no Veilid anonymity equivalence, internet-grade availability, deployed
Foundry service or production transport selection in this kernel.
