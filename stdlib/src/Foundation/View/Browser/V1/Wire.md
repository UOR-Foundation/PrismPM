# Browser presentation prerequisite

DK-23 defines `prismpm/browser-presentation/1` for a private closed DOM adapter.
It does not accept a public application, effective grants, recovery, a product
release, deployment, accessibility certification or Foundation policy.
`PP2011` remains mandatory. `Wire.cddl` defines the exact wire shape.

`Presentation` contains revision, lifecycle, status label, live mode, focus and
ordered `Node` records. `Content` is a closed tagged union. `Intent` contains
revision, action identifier and ordered `FieldValue` records. All counters and
identifiers are uint32. Phase is Ready/Pending/ReplayRequired/Closed (0–3); live
mode is Off/Polite/Assertive (0–2). Zero status means absent; otherwise it is a
one-based reference into the independently source-bound label catalogue.
Content labels are zero-based catalogue references. The catalogue is the DK-21
strictly ordered unique source label list (at most 256 labels, 4096 UTF-8 bytes
per text). A frame never supplies or approves its own catalogue.

Nodes have implicit identifiers 1 through length. Parent zero denotes the
private adapter root; every other parent is an earlier section, navigation or
form. Parent depth is at most 16. Forms cannot have a form ancestor. Fields and
actions must be direct form children. No node has an externally supplied DOM
identifier, tag, role, attribute, class, style, URL, HTML or script. Dynamic
strings are strict UTF-8 text, preserving BOM and Unicode without normalization.

There are at most 256 nodes, 64 actions, 16 field bindings per action, 256 choice
options across the frame, 4096 body table cells across the frame and 16 columns
per table. Each table row exactly matches its columns. Option identifiers are
strictly increasing and nonzero; selection is zero (none) or an existing option.
Action identifiers are unique and nonzero; bindings are strictly increasing
field node identifiers in the same form. At most one action per form is marked
default. An enabled action binds only enabled fields. Only Ready permits enabled
fields/actions. Required empty values are rejected at dispatch, not display.
Text field defaults fit their declared UTF-8 byte maximum. Focus zero retains
focus; nonzero names an existing node. Closed requires focus zero.

The generated codec stores flat table rows privately as at most 256
`TableChunk` records of at most 16 rows; all non-final chunks contain 16 rows.
Chunking adds no wire arrays or values and changes no row/cell limit. It bounds
generated call-stack depth under the pinned compiler's existing stack policy;
writers flatten the chunks back into the identical canonical row array.

The whole frame is limited to 64 MiB (67108864 bytes), including framing. It
must also fit the source-declared View maximum. Dynamic text may occupy the
remaining frame budget. These are operational View budgets, not Organization,
Workspace or other application data limits. An intent must additionally fit
the primary's source-declared request maximum. No accepted frame is normalized:
decode/re-encode must preserve every byte.

Acceptance combines the exact 64 MiB frame with maximum node/row/cell and
combined structural shapes, placing the large payload at both ends. Separate
small structural and single-text maximum tests do not replace these cases.

Before constructing each primitive/array, both decoders consume from a shared
20000-value budget, propagated across sibling collections. Exhaustion is a
limit failure, not a memory trap. A conservative bound for every valid combined
shape is 17160 values: 8 frame values, 12 per node, 64×16 action bindings,
256×3 choice values, 4096 cells, at most 4096 nonempty rows and 256×16 column
labels. Genuine combined-maximum fixtures execute alongside an aggregate bomb
whose individual list bounds are legal. Payload byte limits remain unchanged.

The private renderer validates the complete frame and referenced catalogue
before any DOM mutation. A revision may advance, or repeat with identical bytes;
a changed equal/lower revision rejects. Keyed controls retain edits
only while node kind, generated default/selection and source-owned draft epoch
remain equal. The model advances the epoch to reset a submitted draft or change
editing context, including when the new default is identical. Epoch is not
principal or organization authority. Focus retention applies only to a surviving
same-kind control. A draft reset may retain focus, but not the old selection;
explicit nonzero focus selects the generated target. Before an actual edit, submitted text is
the exact generated default even where native controls display normalized line
endings; after an edit it is the actual native control value. Native controls and form
submission supply keyboard behavior. Status has the generated live mode and
the root is busy in Pending. Close removes listeners and late dispatch results
cannot reopen a closed view. Intent fields are captured in the exact modeled
order; revision is a stale-event guard, not principal/session authority. The
private dispatcher must independently authenticate and authorize commands.
The existence or enabled state of a control grants no effect or role.

Secret-input tag 10 rejects. DK-23 supplies no masked credential control and
does not establish secret custody. Recovery integration must separately provide
a private dispatcher sink with no model-supplied secret default, echo, or
transcript; ordinary text fields are not credential UX.
