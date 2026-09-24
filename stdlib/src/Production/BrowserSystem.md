# Browser-resident system

`Production.BrowserSystem.SystemModel` contains one selected generated application.
Its closed profile requires a browser session, DOM, WebAssembly, no persistent
storage, and no external application services. Unsupported requirements fail.

`validateManifest` checks profile constants, ownership presence, singleton graph
references, distinct identities, mandatory control names, and exact model/manifest
bindings. Source proof does not authenticate ownership or authorize publication.

Release verification separately binds the selected application proof, its full
acceptance evidence, the generated artifact closure, and the locked SPDX oracle.
This containing-system profile does not claim organizational service completeness.
