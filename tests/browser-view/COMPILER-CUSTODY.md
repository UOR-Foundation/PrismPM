# Private compiler executable custody

`compiler-artifact.mjs` captures original and distinct execution-file identities.
It checks owned private directories, complete bytes, inode and link identity before
and after every actual invocation. Frozen branded handles cannot be cloned.
This is not compiler-source provenance or a sandbox against a hostile same-UID
process that can transiently replace and restore files between observations.

27 September: nine devcontainer tests passed, including three executed guard
mutants. The preflight mutant actually writes a sentinel; the genuine guard
refuses before that write. Postflight checks run on successful and failed
commands. Alias, inode, write-permission and forged-handle tests also pass.

Owner-scoped reuse additionally requires fresh pinned compiler construction,
complete static/runtime-source binding and unchanged generated-component owners.
No previous target, disk receipt or caller-selected executable establishes that
provenance. The custody helper alone does not authorize reuse or application
acceptance.
