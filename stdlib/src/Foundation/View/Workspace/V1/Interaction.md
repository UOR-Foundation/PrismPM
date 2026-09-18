# Workspace View byte contract

Lengths and counters are unsigned big-endian; trailing bytes are rejected.
Internal state is `50564901 | session32 | workspace32 | serial16 | phase8 |
table8 | kind8 | pendingLength16 | pending | pageLength24 | page | status8`.
Phases Ready/Pending/ReplayRequired/Closed are 0/1/2/3; effect kinds
Query/Command/Refresh are 0/1/2. The private session is nonzero and the counter
never wraps. A zero workspace means unselected.

Initialization is `00 | session32`. Other requests begin
`operation8 | stateLength24 | state`: operation1 appends public intent,
operation2 appends private completion, operation3 has no suffix.
Intent tags0..6 mean Select(workspace32), Members, Messages, Next,
Command(action8/body), Refresh and Close. Next uses the private admitted cursor.

Completion is `session32 | serial16 | kind8 | outcome8 | payloadLength24 |
payload`. Outcomes0..5 mean success, rejection, conflict, unknown durable outcome,
unavailable and rejection requiring replay. Only successful Query has a payload:
the exact admitted Query page. All successful commands and unavailable commands
require replay. An actual private Command refresh barrier cannot be inferred
from an error name. Exact correlation is consumed once; closing cannot undo a
started commit and never accepts a late result.

Transition success is `00 | stateLength24 | state | effectLength16 | effect`.
Effect is empty or `session32 | serial16 | kind8 | workspace32 | table8 |
inputLength16 | input`. Query input is an empty/135-byte cursor, Command input
is action/body, Refresh input is empty. Errors are exact registered bytes1..14
and carry no replacement state/effect.

Presentation success is `00 | 50564e01 | phase8 | status8 | table8 | controls8 |
focus8 | live8 | workspace32 | head32 | total16 | offset16 | count8 | next8 |
rowsLength24 | rows`. Rows preserve Query framing and order, including all
16×4096-byte messages. Control bits Select1/Members2/Messages4/Next8/Command16/
Refresh32/Close64 express lifecycle, never authority. Focus is retain0/result1;
live mode off0/polite1/assertive2. Text is never markup.

Caps: state66,882; completion66,842; input133,728; effect4,167; output71,055;
presentation66,676 bytes. Every guest is fresh with a 128-page ceiling.
These checked budgets and finite vectors are not universal memory/termination
proofs. Private effect dispatch and read admission are separate owning gates.
