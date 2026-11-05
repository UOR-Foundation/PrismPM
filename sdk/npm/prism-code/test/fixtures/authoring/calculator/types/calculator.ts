// The request and the response are tab-separated text on the wire; examples/ shows the exact bytes.
export type Operation = "add" | "subtract" | "multiply" | "divide";
export type Request = { version: 1; operation: Operation; left: bigint; right: bigint };
export type CalculatorError = "division-by-zero" | "overflow";
export type ProtocolError = "malformed-request" | "unsupported-version" | "unknown-operation";
export type Response = { ok: bigint } | { error: CalculatorError | ProtocolError };
