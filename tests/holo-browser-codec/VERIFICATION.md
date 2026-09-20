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
