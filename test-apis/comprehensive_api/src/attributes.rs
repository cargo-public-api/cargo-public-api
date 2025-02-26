#[no_mangle]
#[link_section = ".custom"]
pub static NO_MANGLE_WITH_CUSTOM_LINK_SECTION: usize = 42;

#[non_exhaustive]
pub enum NonExhaustive {
    MoreToCome,
}

#[repr(C)]
pub struct C {
    pub b: bool,
}

#[repr(Rust)]
pub struct ReprRust {
    pub b: bool,
}

#[doc(hidden)]
pub fn doc_hidden() {}

#[export_name = "something_arbitrary"]
pub fn export_name() {}

// #[must_use] is not shown by cargo doc, so we should not display it either if
// it is present
#[must_use]
pub fn must_use() -> usize {
    0
}
