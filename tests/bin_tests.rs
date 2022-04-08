use assert_cmd::Command;
use public_items::MINIMUM_RUSTDOC_JSON_VERSION;

#[test]
fn print_public_items() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("./tests/rustdoc_json/public_items-v0.4.0.json");
    cmd.assert()
        .stdout("pub enum public_items::Error
pub enum variant public_items::Error::SerdeJsonError(serde_json::Error)
pub fn public_items::Error::fmt(&self, __formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::Error::fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result
pub fn public_items::Error::from(source: serde_json::Error) -> Self
pub fn public_items::Error::source(&self) -> std::option::Option<&std::error::Error>
pub fn public_items::Options::clone(&self) -> Options
pub fn public_items::Options::default() -> Self
pub fn public_items::Options::fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result
pub fn public_items::PublicItem::cmp(&self, other: &PublicItem) -> crate::cmp::Ordering
pub fn public_items::PublicItem::eq(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::PublicItem::ne(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::partial_cmp(&self, other: &PublicItem) -> crate::option::Option<crate::cmp::Ordering>
pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>
pub mod public_items
pub struct field public_items::Options::with_blanket_implementations: bool
pub struct public_items::Options
pub struct public_items::PublicItem
pub type public_items::Result = std::result::Result<T, Error>
")
        .stderr("")
        .success();
}

#[test]
fn print_public_items_with_blanket_implementations() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("--with-blanket-implementations");
    cmd.arg("./tests/rustdoc_json/public_items-v0.4.0.json");
    cmd.assert()
        .stdout(
            "pub const fn public_items::Error::borrow(&self) -> &T
pub const fn public_items::Error::borrow_mut(&mut self) -> &mut T
pub const fn public_items::Error::from(t: T) -> T
pub const fn public_items::Error::into(Self) -> U
pub const fn public_items::Error::try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>
pub const fn public_items::Error::try_into(Self) -> Result<U, <U as TryFrom<T>>::Error>
pub const fn public_items::Options::borrow(&self) -> &T
pub const fn public_items::Options::borrow_mut(&mut self) -> &mut T
pub const fn public_items::Options::from(t: T) -> T
pub const fn public_items::Options::into(Self) -> U
pub const fn public_items::Options::try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>
pub const fn public_items::Options::try_into(Self) -> Result<U, <U as TryFrom<T>>::Error>
pub const fn public_items::PublicItem::borrow(&self) -> &T
pub const fn public_items::PublicItem::borrow_mut(&mut self) -> &mut T
pub const fn public_items::PublicItem::from(t: T) -> T
pub const fn public_items::PublicItem::into(Self) -> U
pub const fn public_items::PublicItem::try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>
pub const fn public_items::PublicItem::try_into(Self) -> Result<U, <U as TryFrom<T>>::Error>
pub enum public_items::Error
pub enum variant public_items::Error::SerdeJsonError(serde_json::Error)
pub fn public_items::Error::fmt(&self, __formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::Error::fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result
pub fn public_items::Error::from(source: serde_json::Error) -> Self
pub fn public_items::Error::source(&self) -> std::option::Option<&std::error::Error>
pub fn public_items::Error::to_string(&self) -> String
pub fn public_items::Error::type_id(&self) -> TypeId
pub fn public_items::Options::clone(&self) -> Options
pub fn public_items::Options::clone_into(&self, target: &mut T)
pub fn public_items::Options::default() -> Self
pub fn public_items::Options::fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result
pub fn public_items::Options::to_owned(&self) -> T
pub fn public_items::Options::type_id(&self) -> TypeId
pub fn public_items::PublicItem::cmp(&self, other: &PublicItem) -> crate::cmp::Ordering
pub fn public_items::PublicItem::eq(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::PublicItem::ne(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::partial_cmp(&self, other: &PublicItem) -> crate::option::Option<crate::cmp::Ordering>
pub fn public_items::PublicItem::to_string(&self) -> String
pub fn public_items::PublicItem::type_id(&self) -> TypeId
pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>
pub mod public_items
pub struct field public_items::Options::with_blanket_implementations: bool
pub struct public_items::Options
pub struct public_items::PublicItem
pub type public_items::Error::Error = <U as TryFrom<T>>::Error
pub type public_items::Error::Error = Infallible
pub type public_items::Options::Error = <U as TryFrom<T>>::Error
pub type public_items::Options::Error = Infallible
pub type public_items::Options::Owned = T
pub type public_items::PublicItem::Error = <U as TryFrom<T>>::Error
pub type public_items::PublicItem::Error = Infallible
pub type public_items::Result = std::result::Result<T, Error>
"
        )
        .stderr("")
        .success();
}

#[test]
fn print_diff_with_changed_and_added() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("./tests/rustdoc_json/public_items-v0.2.0.json");
    cmd.arg("./tests/rustdoc_json/public_items-v0.4.0.json");
    cmd.assert().stdout("Removed:
(nothing)

Changed:
-pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>
+pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>

Added:
+pub fn public_items::Options::clone(&self) -> Options
+pub fn public_items::Options::default() -> Self
+pub fn public_items::Options::fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result
+pub struct field public_items::Options::with_blanket_implementations: bool
+pub struct public_items::Options

").stderr("").success();
}

#[test]
fn print_diff_with_removed_and_added() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("./tests/rustdoc_json/public_items-v0.0.4.json");
    cmd.arg("./tests/rustdoc_json/public_items-v0.0.5.json");
    cmd.assert().stdout("Removed:
-pub fn public_items::from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<HashSet<String>>

Changed:
(nothing)

Added:
+pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<String>>

").stderr("").success();
}

#[test]
fn short_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("-h");
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn long_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn no_args_shows_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn too_many_args_shows_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.args(&["too", "many", "args"]);
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

fn expected_help_text() -> String {
    format!(
        "public_items v{}

Requires at least {}.

NOTE: See https://github.com/Enselic/cargo-public-items for a convenient cargo
wrapper around this program (or to be precise; library) that does everything
automatically.

If you insist of using this low-level utility and thin wrapper, you run it like this:

    public_items <RUSTDOC_JSON_FILE>

where RUSTDOC_JSON_FILE is the path to the output of

    RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

which you can find in

    ./target/doc/${{CRATE}}.json

To diff the public API between two commits, you generate one rustdoc JSON file for each
commit and then pass the path of both files to this utility:

    public_items <RUSTDOC_JSON_FILE_OLD> <RUSTDOC_JSON_FILE_NEW>

To include blanket implementations, pass --with-blanket-implementations.

",
        env!("CARGO_PKG_VERSION"),
        MINIMUM_RUSTDOC_JSON_VERSION,
    )
}
