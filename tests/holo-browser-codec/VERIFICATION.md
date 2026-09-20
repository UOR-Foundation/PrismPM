# HO-13 component verification

The registered `conformance_ho_13` passed in the non-root devcontainer in
791.29 seconds. It replayed current LexLean/kernel verification, regenerated
and compared the entire stdlib package, and passed all three codec/Live cases.
The six generated codec tests passed in both std and no_std. The unchanged
portable codec also passed its original 57 modeled cases.

- LexLean: `4e318850d508e3b22174542b57594f52625d3666d60f2bd919e73fa3b9f07c94`
- Verification: `86c9b0c5503f13c076bb261e8f13cbc7f1f0539fc838bd8ece267d232f285254`
- IR: `88f63bada9d28865703ecf82b2cbff11daf368c86a53e5682b80dcee765b72be`
- Crate: `d29cfcbc50a05c56c0707e2737a505f959b4361b721731fd16de1b8f468534fd`

Normal release packaging independently reproduced the crate. Repository/model
validation passed with 173 IDs, 84 diagnostics and 158 infrastructure tests.
Pinned Live executed its genuine portable positive control and refused the
browser selector before View attachment, including with a 64 MiB `/tmp`.

This establishes the distinct archive profile, not browser application
acceptance, an SDK release or a Foundry deployment. Historical identities do
not replace fresh verification of the integrated source.

## Integrated development baseline

20 September 2026: normal `cargo xtask check-golden --write` passed on
integration `262f591`, producing 348 reviewed files after fresh source/kernel,
export and execution checks. It retains the same semantic identity and IR
above, with all 54 package exports. The generated Runtime adds only the
BrowserWire import; source copies and provenance maps are regenerated.

- Build: `97643781c8ef8bbe46c50f903343e700c69f4a9200e20d006b53c3d69453e29c`
- Verification: `97b8412a718b309f29ff4fe63806810c97a4ae32762ea1606fcab123987d16e5`
- Execution: 597 cases and 54 control-coverage cases passed.

An earlier invocation completed verification but could not replace a
root-owned baseline directory. Correcting ownership and rerunning the normal
writer resolved that filesystem error; no proof or acceptance check changed.
This is the AMD64 development baseline, not native ARM, complete SDK or
deployed Foundry acceptance.

## Integrated private browser baseline

20 September 2026: the normal AMD64 writer passed on `b24cb00`, producing
354 files. The six added source copies bind presentation, custody and journal;
the declaration copy binds isolated durability policy. Their behavioral
acceptance remains with their registered owners, not these source copies.

- Build: `bd667cc528214fed489769cd5c69625c54ea5e70189b0c678630afb4928ac3f5`
- Verification: `5ccbb3ea8c039466ee144f318eacc9db801e587aecc22960fe59c8ff0cdaa53c`
- Runtime semantic identity and IR above are unchanged; all 597 execution
  cases and 54 control-coverage cases passed.

A separate fresh `cargo xtask stdlib-package` check reproduced `prism-stdlib`
0.2.0 from this same verification identity and IR. The generated package,
crate archive and release metadata were unchanged. Integrated `prismpm` and
`repo-conformance` all-target Clippy passed with warnings denied.

Public browser runtime, complete SDK and Foundry deployment remain unaccepted.
