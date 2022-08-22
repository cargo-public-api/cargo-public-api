pub mod v0 {
    pub fn foo() {
    }
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
