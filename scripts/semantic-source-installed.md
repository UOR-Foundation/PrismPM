The SDK's semantic-source helper is a private conformance asset, not a public
authoring command. It is installed outside PATH at
`/opt/prismpm/conformance/bin/prismpm-semantic-source` and measured as
`conformance-semantic-source` in the normal SDK inventory. The wrapper verifies
the fixed inventory, actual executable bytes and root-owned immutable custody
before and after dispatch. It retains all four existing private operations.

The presence of the fixed SDK profile marker, fixed inventory, or fixed
conformance source directory forces this installed route, independently of the
caller's source location. The fixed conformance script location also refuses
if those paths disappear. A missing or changed asset refuses without Cargo
fallback or an external helper override, including in captured source trees.
Older SDKs without this asset therefore refuse this new wrapper. Explicit
source-bootstrap helpers and source Cargo execution remain available only
outside those SDK contexts.

The construction smoke exercises dispatch, including an empty runtime audit;
it does not establish nonempty runtime import closure or installed qualification.
The complete original Binary/Library owners, source checks, immutable inventory
verification and native qualification remain required. This correction adds no
public parser or contract. Public source authoring uses the separately reviewed
normal `prismpm source` CLI, once present in a qualified selected SDK.
