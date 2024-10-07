use jiff::Timestamp;
use semver::Version;

struct CargoPublicApiVersionInfo {
    version: Version,
    min_nightly_version: Timestamp,
}

// | Version          | Understands the rustdoc JSON output of  |
// | ---------------- | --------------------------------------- |
// | 0.38.x -         | nightly-2024-09-10 -                    |
// | 0.37.x           | nightly-2024-07-05 — nightly-2024-09-09 |
// | 0.35.x — 0.36.x  | nightly-2024-06-07 — nightly-2024-07-04 |
// | 0.32.x — 0.34.x  | nightly-2023-08-25 — nightly-2024-06-06 |
// | 0.30.x — 0.31.x  | nightly-2023-05-24 — nightly-2023-08-24 |

fn main() {
    let versions = vec![
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.38.x"),
            min_nightly_version: "2024-09-10".parse().unwrap(),
        },
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.37.0"),
            min_nightly_version: "2024-07-05".parse().unwrap(),
        },
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.36.0"),
            min_nightly_version: "2024-06-07".parse().unwrap(),
        },
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.35.0"),
            min_nightly_version: "2024-06-07".parse().unwrap(),
        },
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.34.0"),
            min_nightly_version: "2023-08-25".parse().unwrap(),
        },
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.33.0"),
            min_nightly_version: "2023-08-25".parse().unwrap(),
        },
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.32.0"),
            min_nightly_version: "2023-08-25".parse().unwrap(),
        },
        CargoPublicApiVersionInfo {
            version: parse_version_and_check("0.31.0"),
            min_nightly_version: "2023-05-24".parse().unwrap(),
        },
    ];
    println!("Hello, world!");
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
