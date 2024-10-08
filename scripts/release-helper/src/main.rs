use jiff::Timestamp;
use semver::Version;

struct CargoPublicApiVersionInfo {
    cargo_public_api_version: Version,
    min_nightly_version: Timestamp,
}

type VersionRange = std::ops::RangeInclusive<Version>;

// | Version          | Understands the rustdoc JSON output of  |
// | ---------------- | --------------------------------------- |
// | 0.38.x -         | nightly-2024-09-10 -                    |
// | 0.37.x           | nightly-2024-07-05 — nightly-2024-09-09 |
// | 0.35.x — 0.36.x  | nightly-2024-06-07 — nightly-2024-07-04 |
// | 0.32.x — 0.34.x  | nightly-2023-08-25 — nightly-2024-06-06 |
// | 0.30.x — 0.31.x  | nightly-2023-05-24 — nightly-2023-08-24 |

fn main() {
    let version_infos = [
        ("0.38.0", "nightly-2024-09-10"),
        ("0.37.0", "nightly-2024-07-05"),
        ("0.36.0", "nightly-2024-06-07"),
        ("0.35.0", "nightly-2024-06-07"),
        ("0.34.0", "nightly-2023-08-25"),
        ("0.33.0", "nightly-2023-08-25"),
        ("0.32.0", "nightly-2023-08-25"),
        ("0.31.0", "nightly-2023-05-24"),
        ("0.30.0", "nightly-2023-05-24"),
    ]
    .into_iter()
    .map(|(version, min_nightly_version)| CargoPublicApiVersionInfo {
        cargo_public_api_version: parse_version_and_check(version),
        min_nightly_version: Timestamp::parse(min_nightly_version.strip_prefix("nightly-"))
            .unwrap(),
    });

    let mut current_min_nightly_version = None;
    let mut current_cargo_public_api_version = None;
    for version_info in version_infos {
        let mut version_row_entry = 
        if version_info.cargo_public_api_version.major == 0 {
            if let Some(min_nightly_version) = current_min_nightly_version {
                if version_info.min_nightly_version < min_nightly_version {
                    panic!(
                        "Version {} requires a minimum nightly version of {}",
                        version_info.cargo_public_api_version,
                        min_nightly_version
                    );
                }
            }
            current_min_nightly_version = Some(version_info.min_nightly_version);
        }
    }
}

fn parse_version_and_check(version: &str) -> Version {
    let version = Version::parse("0.38.0").unwrap();
    if version.major != 0 {
        panic!("Major version must be 0");
    }
    if version.patch != 0 {
        panic!("Patch version must be 0");
    }
}
