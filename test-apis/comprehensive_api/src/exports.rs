pub mod v0 {
    pub fn foo() {}
}

pub mod v1 {
    // Make v1 compatible with v0 by using a wildcard import like this
    pub use super::v0::*;

    pub fn foo2() {
        foo();
    }
}

pub mod recursion_1 {
    pub use super::recursion_2;
}

pub mod recursion_2 {
    pub use super::recursion_1;
}

pub mod recursion_glob_1 {
    pub use super::recursion_glob_2::*;
}

pub mod recursion_glob_2 {
    pub use super::recursion_glob_1::*;
}

/// Regression test for <https://github.com/cargo-public-api/cargo-public-api/issues/145>
pub mod issue_145 {
    pub mod external {
        pub struct External;
    }

    use external as privately_renamed;
    use external::External;
    use external::External as PrivatelyRenamed;

    pub fn external_arg_type(_transform: External) {}
    pub fn privately_renamed_arg_type(_transform: PrivatelyRenamed) {}
    pub fn external_external_arg_type(_transform: external::External) {}
    pub fn privately_renamed_external_arg_type(_transform: privately_renamed::External) {}

    pub mod external_2 {
        pub struct External;
    }

    pub use external_2 as publicly_renamed;
    use external_2::External as PrivatelyRenamed2;

    pub fn privately_renamed_2_arg_type(_transform: PrivatelyRenamed2) {}
    pub fn publicly_renamed_external(_transform: publicly_renamed::External) {}

    pub mod external_3 {
        pub struct External;
    }

    use external_3 as privately_renamed_3;
    pub use privately_renamed_3::External as PubliclyRenamedFromPrivateMod;

    pub fn publicly_renamed_from_private_mod_arg_type(_transform: PubliclyRenamedFromPrivateMod) {}
}

/// Regression test for <https://github.com/cargo-public-api/cargo-public-api/issues/410>
pub mod issue_410 {
    pub mod container {
        pub mod super_glob {
            pub use super::*;
        }
    }
    pub use container::super_glob;
}
