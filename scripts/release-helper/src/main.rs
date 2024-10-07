use jiff::Timestamp;
use semver::Version;

struct CargoPublicApiVersion {
    name: String,

    /// E.g. "2024-09-10"
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
        CargoPublicApiVersion {
            name: "0.38.x".to_string(),
            min_nightly_version: "2024-09-10".parse().unwrap(),
        },
        CargoPublicApiVersion {
            name: "0.37.x".to_string(),
            min_nightly_version: "2024-07-05".parse().unwrap(),
        },
        CargoPublicApiVersion {
            name: "0.36.x".to_string(),
            min_nightly_version: "2024-06-07".parse().unwrap(),
        },
        CargoPublicApiVersion {
            name: "0.35.x".to_string(),
            min_nightly_version: "2024-06-07".parse().unwrap(),
        },
        CargoPublicApiVersion {
            name: "0.34.x".to_string(),
            min_nightly_version: "2023-08-25".parse().unwrap(),
        },
        CargoPublicApiVersion {
            name: "0.33.x".to_string(),
            min_nightly_version: "2023-08-25".parse().unwrap(),
        },
        CargoPublicApiVersion {
            name: "0.32.x".to_string(),
            min_nightly_version: "2023-08-25".parse().unwrap(),
        },
        CargoPublicApiVersion {
            name: "0.31.x".to_string(),
            min_nightly_version: "2023-05-24".parse().unwrap(),
        },
    ];
    println!("Hello, world!");
}
