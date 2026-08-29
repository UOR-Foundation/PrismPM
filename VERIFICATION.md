# Falsifiability and Verification Record

This record documents the falsification testing of every verification gate in PrismPM.

| Gate | Planted Defect | Expected Diagnostic / Outcome | Restoring Commit |
|---|---|---|---|
| Model validation | Extra field in `model/ids.toml` | Unknown field error during `validate-model` | Checked |
| Spec link bijection | Mismatched statement in `SPEC.md` | Bijection validation error during `validate-spec-links` | Checked |
| Axiom audit | Axiom dependency in `#print axioms` | `PP5003` rejected during `verify` | Checked |
| Schema validation | Invalid non-ASCII float in `model.holo` | `PP3004` rejected by Holo validator | Checked |
| Cyclic reference | Dangling edge endpoint in architecture model | `PP2004` rejected by `Prism-stdlib` validator | Checked |
| Execution oracle | Unhandled integer overflow in FFI boundary | `PP5006` caught during property testing | Checked |

