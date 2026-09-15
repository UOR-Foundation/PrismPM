//! Compile and execute the complete original 0.1.x public application API.
//! This is a consumer test, not a replacement implementation or generated file.
//! The 31 function signatures and two public types are frozen from
//! 94697b1ff196ef362f4a6137ba72145e2ac68faf:stdlib/generated/package/src/lib.rs.
//! Acceptance is offline and does not depend on retrieving that Git revision.

use std::path::Path;
use std::process::Command;

const CONSUMER: &str = r#"
use prism_stdlib::*;

fn assert_public_traits<T: Copy + Eq + core::fmt::Debug>() {}

fn main() {
    // Function-pointer assignments preserve ownership, arity and error types.
    let _: fn(Vec<u8>, Vec<u8>) -> Vec<u8> = appendBytes;
    let _: fn(Vec<u8>, u64) -> Option<u8> = byteAt;
    let _: fn(Vec<u8>) -> u64 = byteLength;
    let _: fn(Vec<u8>, Vec<u8>) -> core::cmp::Ordering = compareBytes;
    let _: fn(Vec<u8>, u64, u64) -> Option<Vec<u8>> = sliceBytes;
    let _: fn(i64) -> String = formatInt64;
    let _: fn(String) -> Option<i64> = parseInt64;
    let _: fn() -> bool = portableTrue;
    let _: fn(StandardsProfile) -> u64 = applicationSecurityEdition;
    let _: fn(StandardsProfile) -> u64 = architectureEdition;
    let _: fn(StandardsProfile) -> u64 = controlEdition;
    let _: fn(StandardsProfile) -> u64 = qualityEdition;
    let _: fn(StandardsProfile) -> u64 = riskEdition;
    let _: fn(u64, &[u64]) -> bool = allBelow;
    let _: fn(u64, &[u64]) -> Result<bool, ComputeError> = allConsecutive;
    let _: fn(u64, u64, &mut [u64]) -> Result<usize, ComputeError> = canonicalIndexes;
    let _: fn(&[u64]) -> Result<bool, ComputeError> = validateComponentIndexes;
    let _: fn(u64, &[u64]) -> bool = validateControlLinks;
    let _: fn(u64, &[u64]) -> bool = validateEdgeEndpoints;
    let _: fn(StandardsProfile) -> bool = validateExactStandardsProfile;
    let _: fn(u64, &[u64]) -> bool = validateFlattenedBounds;
    let _: fn(u64, &[u64]) -> bool = validateQualityLinks;
    let _: fn(u64, &[u64]) -> bool = validateRiskLinks;
    let _: fn(u64, &[u64]) -> bool = validateViewpointLinks;
    let _: fn(i64, i64) -> Option<i64> = checkedAddInt64;
    let _: fn(i64, i64) -> Option<i64> = checkedDivideInt64;
    let _: fn(i64, i64) -> Option<i64> = checkedMultiplyInt64;
    let _: fn(i64) -> Option<i64> = checkedNegateInt64;
    let _: fn(i64, i64) -> Option<i64> = checkedSubtractInt64;
    let _: fn(Vec<u8>) -> Option<String> = decode;
    let _: fn(String) -> Vec<u8> = encode;
    assert_public_traits::<StandardsProfile>();
    assert_public_traits::<ComputeError>();
    let _ = [ComputeError::AddOverflow, ComputeError::MulOverflow,
        ComputeError::ShiftExponentTooLarge, ComputeError::ShiftOverflow,
        ComputeError::PowExponentTooLarge, ComputeError::PowOverflow,
        ComputeError::OutputTooSmall];

    assert_eq!(appendBytes(vec![1], vec![2, 3]), vec![1, 2, 3]);
    assert_eq!(appendBytes(vec![], vec![]), Vec::<u8>::new());
    assert_eq!(appendBytes(vec![], vec![0]), vec![0]);
    assert_eq!(appendBytes(vec![0], vec![]), vec![0]);
    assert_eq!(byteAt(vec![1, 2], 1), Some(2));
    assert_eq!(byteAt(vec![1, 2], 2), None);
    assert_eq!(byteAt(vec![], 0), None);
    assert_eq!(byteAt(vec![0], 0), Some(0));
    assert_eq!(byteLength(vec![1, 2]), 2);
    assert_eq!(byteLength(vec![]), 0);
    assert_eq!(compareBytes(vec![1], vec![2]), core::cmp::Ordering::Less);
    assert_eq!(compareBytes(vec![0], vec![0]), core::cmp::Ordering::Equal);
    assert_eq!(compareBytes(vec![2], vec![1]), core::cmp::Ordering::Greater);
    assert_eq!(compareBytes(vec![], vec![]), core::cmp::Ordering::Equal);
    assert_eq!(sliceBytes(vec![1, 2, 3], 1, 2), Some(vec![2, 3]));
    assert_eq!(sliceBytes(vec![], 0, 0), Some(vec![]));
    assert_eq!(sliceBytes(vec![0], 1, 0), Some(vec![]));
    assert_eq!(sliceBytes(vec![0], 1, 1), None);
    assert_eq!(sliceBytes(vec![0], 2, 0), None);
    assert_eq!(sliceBytes(vec![1], u64::MAX, 1), None);
    for value in [i64::MIN, -1, 0, 1, i64::MAX] {
        assert_eq!(parseInt64(formatInt64(value)), Some(value));
    }
    assert_eq!(parseInt64("9223372036854775808".to_owned()), None);
    assert_eq!(parseInt64("not an integer".to_owned()), None);
    assert!(portableTrue());
    assert_eq!(checkedAddInt64(1, 2), Some(3));
    assert_eq!(checkedAddInt64(i64::MAX, 1), None);
    assert_eq!(checkedSubtractInt64(1, 2), Some(-1));
    assert_eq!(checkedSubtractInt64(i64::MIN, 1), None);
    assert_eq!(checkedMultiplyInt64(2, 3), Some(6));
    assert_eq!(checkedMultiplyInt64(i64::MAX, 2), None);
    assert_eq!(checkedNegateInt64(1), Some(-1));
    assert_eq!(checkedNegateInt64(i64::MIN), None);
    assert_eq!(checkedDivideInt64(7, 2), Some(3));
    assert_eq!(checkedDivideInt64(-7, 2), Some(-3));
    assert_eq!(checkedDivideInt64(7, -2), Some(-3));
    assert_eq!(checkedDivideInt64(-7, -2), Some(3));
    assert_eq!(checkedDivideInt64(1, 0), None);
    assert_eq!(checkedDivideInt64(i64::MIN, -1), None);
    assert_eq!(decode(encode("Prism λ".to_owned())), Some("Prism λ".to_owned()));
    assert_eq!(decode(encode(String::new())), Some(String::new()));
    assert_eq!(decode(encode("\0".to_owned())), Some("\0".to_owned()));
    assert_eq!(decode(vec![0xff]), None);

    let profile = StandardsProfile { architectureEdition: 2022,
        applicationSecurityEdition: 2011, controlEdition: 2017,
        riskEdition: 2022, qualityEdition: 2023 };
    assert_eq!(architectureEdition(profile), 2022);
    assert_eq!(applicationSecurityEdition(profile), 2011);
    assert_eq!(controlEdition(profile), 2017);
    assert_eq!(riskEdition(profile), 2022);
    assert_eq!(qualityEdition(profile), 2023);
    assert!(validateExactStandardsProfile(profile));
    assert!(!validateExactStandardsProfile(StandardsProfile { qualityEdition: 0, ..profile }));
    assert!(allBelow(2, &[0, 1]));
    assert!(!allBelow(1, &[0, 1]));
    assert_eq!(allConsecutive(0, &[0, 1]), Ok(true));
    assert_eq!(validateComponentIndexes(&[0, 1]), Ok(true));
    assert_eq!(validateComponentIndexes(&[1, 0]), Ok(false));
    let mut output = [0; 2];
    assert_eq!(canonicalIndexes(0, 2, &mut output), Ok(2));
    assert_eq!(output, [0, 1]);
    for check in [validateControlLinks, validateEdgeEndpoints, validateFlattenedBounds,
        validateQualityLinks, validateRiskLinks, validateViewpointLinks] {
        assert!(check(2, &[0, 1]));
        assert!(!check(1, &[0, 1]));
    }
}
"#;

fn checked(command: &mut Command) {
    let output = command
        .output()
        .expect("run pinned Rust compiler or consumer");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn stdlib_preserves_original_application_api_and_behavior() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let work = tempfile::tempdir().unwrap();
    let consumer = work.path().join("consumer.rs");
    std::fs::write(&consumer, CONSUMER).unwrap();
    for standard_library in [false, true] {
        let library = work
            .path()
            .join(format!("libprism_stdlib_{standard_library}.rlib"));
        let mut compile = Command::new("rustc");
        compile.args([
            "--edition=2021",
            "--crate-name=prism_stdlib",
            "--crate-type=rlib",
        ]);
        if standard_library {
            compile.args(["--cfg", "feature=\"std\""]);
        }
        checked(
            compile
                .arg(root.join("stdlib/generated/package/src/lib.rs"))
                .arg("-o")
                .arg(&library),
        );
        let binary = work.path().join(format!("consumer_{standard_library}"));
        checked(
            Command::new("rustc")
                .arg("--edition=2021")
                .arg(&consumer)
                .arg("--extern")
                .arg(format!("prism_stdlib={}", library.display()))
                .arg("-o")
                .arg(&binary),
        );
        checked(&mut Command::new(binary));
    }
}
