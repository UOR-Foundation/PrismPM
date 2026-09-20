# Installed product CLI gate

`bash scripts/product-sdk-check.sh SDK_INDEX@sha256:DIGEST SOURCE_COMMIT NEW_EVIDENCE_DIR`
runs after the real two-platform SDK index and signed child SBOMs exist, before
promotion. Both native installed-SDK jobs require it in addition to full V&V.

The image supplies all model, compiler, verifier and fixture bytes. The existing
platform-lock capture measures both image inventories. Explicit `fetch --locked`
acquires authoritative inputs and actual image advisory scans; a separate offline,
socket-free container constructs releases A/B in two roots through the public
product CLI. Cargo input is the actual generated package and checksum-bound
image-owned registry, not a handwritten package or crates.io qualification.

A fresh offline receiver receives only the OCI graph and checks source-free
replay/export, missing/changed proof rejection and restored export. Inventory,
locks, advisory results, generated SPDX and exact identities remain evidence.
This gate grants neither SDK release acceptance nor deployment authorization.
Unit boundary fixtures do not count as installed-image execution.
