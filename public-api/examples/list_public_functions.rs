use rustdoc_types::{Crate, Id, ItemEnum};
use std::{error::Error, fs::File};

fn main() -> Result<(), Box<dyn Error>> {
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("test-apis/example_api-v0.2.0/Cargo.toml")
        .build()?;

    let public_api = public_api::Builder::from_rustdoc_json(&json_path).build()?;

    let file = File::open(json_path)?;
    let krate = serde_json::from_reader::<_, Crate>(file)?;

    for public_item in public_api.items() {
        if !is_function(&krate, public_item.id()) {
            continue;
        }
        println!("{public_item}");
    }

    Ok(())
}

fn is_function(krate: &Crate, id: Id) -> bool {
    matches!(krate.index.get(&id).unwrap().inner, ItemEnum::Function(_))
}
