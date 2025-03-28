//! See `docs/RELEASE.md` for information on how to use this.

pub struct CargoPublicApiVersionInfo {
    pub cargo_public_api_version: &'static str,
    pub min_nightly_rust_version: &'static str,
}

pub static TABLE: &[CargoPublicApiVersionInfo] = &[
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.47.x",
        min_nightly_rust_version: "nightly-2025-03-24",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.46.x", // was never actually released
        min_nightly_rust_version: "nightly-2025-03-16",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.45.x", // was never actually released
        min_nightly_rust_version: "nightly-2025-03-14",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.44.x",
        min_nightly_rust_version: "nightly-2025-01-25",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.43.x",
        min_nightly_rust_version: "nightly-2025-01-25",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.42.x",
        min_nightly_rust_version: "nightly-2024-10-18",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.41.x",
        min_nightly_rust_version: "nightly-2024-10-18",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.40.x",
        min_nightly_rust_version: "nightly-2024-10-18",
    },
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
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.36.x",
        min_nightly_rust_version: "nightly-2024-06-07",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.35.x",
        min_nightly_rust_version: "nightly-2024-06-07",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.34.x",
        min_nightly_rust_version: "nightly-2023-08-25",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.33.x",
        min_nightly_rust_version: "nightly-2023-08-25",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.32.x",
        min_nightly_rust_version: "nightly-2023-08-25",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.31.x",
        min_nightly_rust_version: "nightly-2023-05-24",
    },
    CargoPublicApiVersionInfo {
        cargo_public_api_version: "0.30.x",
        min_nightly_rust_version: "nightly-2023-05-24",
    },
];
