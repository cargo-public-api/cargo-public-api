use std::io::Write;

use public_api::{diff::PublicApiDiff, PublicApi};

pub struct Json;

#[derive(serde::Serialize)]
struct Api {
    items: Vec<PublicItem>,
}

#[derive(serde::Serialize)]
struct Diff {
    added: Vec<PublicItem>,
    removed: Vec<PublicItem>,
    changed: Vec<ChangedItem>,
}

#[derive(serde::Serialize)]
#[serde(transparent)]
struct PublicItem(String);

#[derive(serde::Serialize)]
struct ChangedItem {
    old: PublicItem,
    new: PublicItem,
}

impl Json {
    pub fn print_items(w: &mut dyn Write, api: PublicApi) -> std::io::Result<()> {
        serde_json::to_writer(
            w,
            &Api {
                items: api
                    .items()
                    .into_iter()
                    .map(|itm| PublicItem(itm.to_string()))
                    .collect(),
            },
        )?;

        Ok(())
    }

    pub fn print_diff(w: &mut dyn Write, diff: &PublicApiDiff) -> std::io::Result<()> {
        serde_json::to_writer(
            w,
            &Diff {
                removed: diff
                    .removed
                    .iter()
                    .map(|itm| PublicItem(itm.to_string()))
                    .collect(),
                added: diff
                    .added
                    .iter()
                    .map(|itm| PublicItem(itm.to_string()))
                    .collect(),
                changed: diff
                    .changed
                    .iter()
                    .map(|ch| ChangedItem {
                        old: PublicItem(ch.old.to_string()),
                        new: PublicItem(ch.new.to_string()),
                    })
                    .collect(),
            },
        )?;
        Ok(())
    }
}
