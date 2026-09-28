# DK-33 verification

Private account-declaration prerequisite. No account service, authenticated
credential succession, mailbox fact, accepted SDK or application deployment.

The registered and installed owners require six construction checks and eight
complete-owner checks. The latter must freshly verify source/kernel/native
std/no_std/two-Wasm parity for all 390 vectors, three browser engines with exact
12-call native-replayed transcripts, four actual host mutations per engine and
ten separately compiled source mutations. Missing engines, setup errors,
unrelated exceptions and invented mutation labels cannot satisfy the owner.

Current checks: six construction tests pass; all 30 SDK-boundary tests pass;
normal model generation and the 186-ID specification bijection pass. The fresh
full owner is **red** (211.891 seconds): source/kernel, both native modes, both
generated Wasm builds, all 390 vectors, and Chromium/Firefox journeys plus host
mutants passed; WebKit admitted the invalid public point `04` plus 64 zero bytes.
The ten compiled source mutations were not reached. A source-owned, independently
verified full P-256 point predicate must guard admission; a zero-point blacklist
or browser-import assumption cannot fix this. No accepted SDK or account service
is inferred. Checks run in pinned devcontainers.

The source frame ceiling is 512 bytes. The verification module's input allocator
allows 1024 bytes so malformed frame 513 reaches the actual source refusal;
allocation 1024 must succeed and 1025 must trap. Output allocation remains 512.
Largest valid request/response are 174/409 bytes. This test transport allowance
does not enlarge the admitted source domain.
