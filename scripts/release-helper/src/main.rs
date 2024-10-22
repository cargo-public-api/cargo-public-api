use jiff::Timestamp;
use semver::Version;
use pretty_assertions;

struct CargoPublicApiVersionInfo {
    cargo_public_api_version: Version,
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
    let version_infos = [
        ("0.39.0", "nightly-2024-10-13"),
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

}

fn parse_version_and_check(version: &str) -> Version {
    let version = Version::parse(version).unwrap();
    if version.major != 0 {
        panic!("Major version must be 0");
    }
    if version.patch != 0 {
        panic!("Patch version must be 0");
    }
}

fn render_compatibility_matrix(version_infos: &[CargoPublicApiVersionInfo]) -> String {

    struct CompatibilityMatrixRow {
        cargo_public_api_version_range: CargoPublicApiVersionRange,
        nightly_version_range: NightlyVersionRange,
    }

    struct CargoPublicApiVersionRange {
        start: Version,
        // This is not an `Option`` because we never have to represent "all
        // unknown future versions". We know exactly what versions of
        // cargo-public-api that exists. If it changes, we update this code.
        end: Version,
    }

    struct NightlyVersionRange {
        start: Timestamp,
        end: Option<Timestamp>,
    }

    // We start from the bottom
    let version_infos_reversed = version_infos.iter().rev();
    
    
    let mut rows = Vec::new();
    let mut output_string = String::new();
    let mut current_version_range = None;
    let mut current_nightly_version_range = None;
    let mut current_min_nightly_version = None;
    let mut current_cargo_public_api_version = None;
    for version_info in version_infos_reversed {
        if current_version_range.is_none() {
            current_version_range = Some(CargoPublicApiVersionRange {
                start: version_info.cargo_public_api_version.clone(),
                end: version_info.cargo_public_api_version.clone(),
            });
        } else {
            rows.push(CompatibilityMatrixRow {
                cargo_public_api_version_range: current_version_range.unwrap(),
                nightly_version_range: current_nightly_version_range.unwrap(),
            });

        }
        if current_nightly_version_range.is_none() {
            current_nightly_version_range = Some(NightlyVersionRange {
                start: version_info.min_nightly_version.clone(),
                end: None,
            });
        }
        if current_cargo_public_api_version.is_none() {
            current_cargo_public_api_version = Some(version_info.cargo_public_api_version);
        }

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


/*


| Version          | Understands the rustdoc JSON output of  |
| ---------------- | --------------------------------------- |
| 0.38.x -         | nightly-2024-09-10 -                    |
| 0.37.x           | nightly-2024-07-05 — nightly-2024-09-09 |
| 0.35.x — 0.36.x  | nightly-2024-06-07 — nightly-2024-07-04 |
| 0.32.x — 0.34.x  | nightly-2023-08-25 — nightly-2024-06-06 |
| 0.30.x — 0.31.x  | nightly-2023-05-24 — nightly-2023-08-24 |



*/


fn render_compatibility_matrix_helper(version_infos: &[(&str, &str)]) -> Vec<CargoPublicApiVersionInfo> {
    version_infos.iter()
    .map(|(version, min_nightly_version)| CargoPublicApiVersionInfo {
        cargo_public_api_version: parse_version_and_check(version),
        min_nightly_version: Timestamp::parse(min_nightly_version.strip_prefix("nightly-"))
            .unwrap(),
    }).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_test(version_infos: [(&str, &str)], expected_output: &str) {
        let version_infos = render_compatibility_matrix_helper(&version_infos);
        let output = render_compatibility_matrix(&version_infos);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_render_compatibility_matrix_one_version() {
        do_test([("0.39.0", "nightly-2024-10-13")
        ],         "| Version          | Understands the rustdoc JSON output of  |\n\
        | ---------------- | --------------------------------------- |\n\
        | 0.38.x -         | nightly-2024-09-10 -                    |\n"
        );
        


     

        
    }
}