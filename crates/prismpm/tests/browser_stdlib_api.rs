//! Consumer-visible generated APIs; full behavior belongs to DK-13..DK-16.

#[test]
fn modeled_browser_commands_and_queries_keep_the_registered_public_signature() {
    let command: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspaceCommandBytes;
    let query: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspaceQueryBytes;
    assert_eq!(command(Vec::new()).unwrap(), [1]);
    assert_eq!(query(Vec::new()).unwrap(), [1]);
    let interaction: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspaceInteractionBytes;
    let presentation: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspacePresentationBytes;
    assert_eq!(interaction(Vec::new()).unwrap(), [1]);
    assert_eq!(presentation(Vec::new()).unwrap(), [2]);
}
