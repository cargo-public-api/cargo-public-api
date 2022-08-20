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
