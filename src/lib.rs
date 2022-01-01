mod error;
pub use error::Error;
pub use error::Result;

mod internal;
pub use internal::from_rustdoc_json_str;
