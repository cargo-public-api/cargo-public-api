use jiff::{civil::Date, ToSpan};

use crate::version_info::CargoPublicApiVersionInfo;

pub fn render(version_infos: &[CargoPublicApiVersionInfo]) -> String {
    /// Same as ['CargoPublicApiVersionInfo'] but easier to manipulate
    /// programatically.
    struct InternalCargoPublicApiVersionInfo {
        cargo_public_api_minor_version: u32,
        min_nightly_rust_version: Date,
    }

    /// Represents `0.39.x — 0.40.x` or just `0.39.x` if start and end is the
    /// same.
    #[derive(Clone)]
    struct CargoPublicApiMinorVersionRange {
        start: u32,
        end: u32,
    }

    #[derive(Clone)]
    struct NightlyVersionRange {
        start: Date,
        end: Option<Date>,
    }

    #[derive(Clone)]
    struct CompatibilityMatrixRow {
        cargo_public_api_version_range: CargoPublicApiMinorVersionRange,
        nightly_rust_version_range: NightlyVersionRange,
    }

    let version_infos: Vec<_> = version_infos
        .iter()
        .map(|version_info| InternalCargoPublicApiVersionInfo {
            // Turn `0.39.x` into `39`
            cargo_public_api_minor_version: version_info
                .cargo_public_api_version
                .split('.')
                .nth(1)
                .unwrap()
                .parse()
                .unwrap(),
            // Turn `nightly-2024-10-13` into `2024-10-13`
            min_nightly_rust_version: version_info
                .min_nightly_rust_version
                .strip_prefix("nightly-")
                .unwrap()
                .parse()
                .unwrap(),
        })
        .collect();

    let mut rows = Vec::new();
    let mut current_row: Option<CompatibilityMatrixRow> = None;
    for version_info in version_infos.iter().rev() {
        if let Some(mut row) = current_row.take() {
            if row.nightly_rust_version_range.start == version_info.min_nightly_rust_version {
                row.cargo_public_api_version_range.end =
                    version_info.cargo_public_api_minor_version;
                current_row = Some(row);
            } else {
                row.nightly_rust_version_range.end = Some(
                    version_info
                        .min_nightly_rust_version
                        .checked_sub(1.days())
                        .unwrap(),
                );
                rows.push(row);
                current_row = None;
            }
        }
        if current_row.is_none() {
            current_row = Some(CompatibilityMatrixRow {
                cargo_public_api_version_range: CargoPublicApiMinorVersionRange {
                    start: version_info.cargo_public_api_minor_version,
                    end: version_info.cargo_public_api_minor_version,
                },
                nightly_rust_version_range: NightlyVersionRange {
                    start: version_info.min_nightly_rust_version,
                    end: None,
                },
            });
        }
    }
    rows.push(current_row.unwrap());

    let mut output = String::new();
    for row in rows.iter().rev() {
        let cargo_public_api_version_str = if row.cargo_public_api_version_range.start
            == row.cargo_public_api_version_range.end
        {
            format!("0.{}.x         ", row.cargo_public_api_version_range.start)
        } else {
            format!(
                "0.{}.x — 0.{}.x",
                row.cargo_public_api_version_range.start, row.cargo_public_api_version_range.end
            )
        };
        let nightly_version_range_str = format!(
            "{} — {}",
            jiff::fmt::strtime::format("nightly-%Y-%m-%d", row.nightly_rust_version_range.start)
                .unwrap(),
            row.nightly_rust_version_range
                .end
                .map(|t| jiff::fmt::strtime::format("nightly-%Y-%m-%d", t).unwrap())
                .unwrap_or_else(|| "                  ".to_string())
        );
        output.push_str(&format!(
            "| {cargo_public_api_version_str}  | {nightly_version_range_str} |\n",
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_render_compatibility_matrix_one_version() {
        assert_render(
            &[CargoPublicApiVersionInfo {
                cargo_public_api_version: "0.39.x",
                min_nightly_rust_version: "nightly-2024-10-13",
            }],
            "| 0.39.x           | nightly-2024-10-13 —                    |\n\
            ",
        );
    }

    #[test]
    fn test_render_compatibility_matrix_two_versions() {
        assert_render(
            &[
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.39.x",
                    min_nightly_rust_version: "nightly-2024-10-13",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.38.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
            ],
            "| 0.39.x           | nightly-2024-10-13 —                    |\n\
             | 0.38.x           | nightly-2024-09-10 — nightly-2024-10-12 |\n\
            ",
        );
    }

    #[test]
    fn test_render_compatibility_matrix_two_versions_same_nightly() {
        assert_render(
            &[
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.39.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.38.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
            ],
            "| 0.38.x — 0.39.x  | nightly-2024-09-10 —                    |\n\
            ",
        );
    }

    #[test]
    fn test_render_compatibility_matrix_three_versions() {
        assert_render(
            &[
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.39.x",
                    min_nightly_rust_version: "nightly-2024-10-13",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.38.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.37.x",
                    min_nightly_rust_version: "nightly-2024-07-05",
                },
            ],
            "\
             | 0.39.x           | nightly-2024-10-13 —                    |\n\
             | 0.38.x           | nightly-2024-09-10 — nightly-2024-10-12 |\n\
             | 0.37.x           | nightly-2024-07-05 — nightly-2024-09-09 |\n\
            ",
        );
    }

    #[test]
    fn test_render_compatibility_matrix_three_versions_last_same_nightly() {
        assert_render(
            &[
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.39.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.38.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.37.x",
                    min_nightly_rust_version: "nightly-2024-07-05",
                },
            ],
            "\
             | 0.38.x — 0.39.x  | nightly-2024-09-10 —                    |\n\
             | 0.37.x           | nightly-2024-07-05 — nightly-2024-09-09 |\n\
            ",
        );
    }

    #[test]
    fn test_render_compatibility_matrix_three_versions_first_same_nightly() {
        assert_render(
            &[
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.39.x",
                    min_nightly_rust_version: "nightly-2024-10-13",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.38.x",
                    min_nightly_rust_version: "nightly-2024-07-05",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.37.x",
                    min_nightly_rust_version: "nightly-2024-07-05",
                },
            ],
            "\
             | 0.39.x           | nightly-2024-10-13 —                    |\n\
             | 0.37.x — 0.38.x  | nightly-2024-07-05 — nightly-2024-10-12 |\n\
            ",
        );
    }

    #[test]
    fn test_render_compatibility_matrix_four_versions_middle_same_nightly() {
        assert_render(
            &[
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.39.x",
                    min_nightly_rust_version: "nightly-2024-10-13",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.38.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.37.x",
                    min_nightly_rust_version: "nightly-2024-09-10",
                },
                CargoPublicApiVersionInfo {
                    cargo_public_api_version: "0.36.x",
                    min_nightly_rust_version: "nightly-2024-06-07",
                },
            ],
            "\
             | 0.39.x           | nightly-2024-10-13 —                    |\n\
             | 0.37.x — 0.38.x  | nightly-2024-09-10 — nightly-2024-10-12 |\n\
             | 0.36.x           | nightly-2024-06-07 — nightly-2024-09-09 |\n\
            ",
        );
    }

    fn assert_render(version_infos: &[CargoPublicApiVersionInfo], expected_output: &str) {
        let output = render(version_infos);
        assert_eq!(output, expected_output);
    }
}
