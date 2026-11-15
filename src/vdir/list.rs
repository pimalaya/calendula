//! # Vdir collection list
//!
//! The `calendula vdir list` command, showing every collection
//! directory under the configured home directory.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_vdir::collection::VdirCollection;
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::vdir::client::VdirClient;

/// List on-disk vdir collections.
///
/// One line per directory directly under the configured home: its id,
/// its path, then the colour and the display name its metadata marker
/// files carry, the description under `--json` alone, as `calendar list`
/// prints it.
///
/// JSON output: `{"collections": [{"id", "displayName", "description",
/// "color", "path"}]}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionListCommand;

impl VdirCollectionListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let collections = client.list_collections()?;
        let rows = collections.into_iter().map(CollectionRow::from).collect();
        printer.out(VdirCollectionListOutput { rows })
    }
}

/// The vdir collection listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VdirCollectionListOutput {
    /// One entry per collection directory.
    #[serde(rename = "collections")]
    pub rows: Vec<CollectionRow>,
}

impl fmt::Display for VdirCollectionListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::new(["ID", "PATH", "COLOR", "NAME"]);

        for collection in &self.rows {
            let name = collection.display_name.as_deref().unwrap_or_default();
            let color = collection.color.as_deref().unwrap_or_default();

            columns.push([
                sanitize::text(&collection.id).into_owned(),
                sanitize::text(&collection.path).into_owned(),
                sanitize::text(color).into_owned(),
                sanitize::text(name).into_owned(),
            ]);
        }

        write!(f, "{columns}")
    }
}

/// One collection directory, as the listing prints it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRow {
    /// The directory name, which is the collection id.
    pub id: String,
    /// The name the `displayname` marker file carries, if any.
    pub display_name: Option<String>,
    /// The text the `description` marker file carries, if any.
    pub description: Option<String>,
    /// The colour the `color` marker file carries, if any.
    pub color: Option<String>,
    /// The directory's absolute path.
    pub path: String,
}

impl From<VdirCollection> for CollectionRow {
    fn from(collection: VdirCollection) -> Self {
        Self {
            id: collection.id().to_owned(),
            display_name: collection.display_name.clone(),
            description: collection.description.clone(),
            color: collection.color.clone(),
            path: collection.path.as_str().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// A row carries the metadata its marker files hold, as cardamum's
    /// does: the description under `--json` alone, the colour in both.
    #[test]
    fn a_row_carries_the_description_and_the_colour() {
        let output = VdirCollectionListOutput {
            rows: vec![
                CollectionRow {
                    id: String::from("work"),
                    display_name: Some(String::from("Work")),
                    description: Some(String::from("Meetings")),
                    color: Some(String::from("#ABCDEF")),
                    path: String::from("/vdir/work"),
                },
                CollectionRow {
                    id: String::from("bare"),
                    display_name: None,
                    description: None,
                    color: None,
                    path: String::from("/vdir/bare"),
                },
            ],
        };

        let expected = "\
ID    PATH        COLOR    NAME
work  /vdir/work  #ABCDEF  Work
bare  /vdir/bare  -        -
";
        assert_eq!(output.to_string(), expected);

        assert_eq!(
            serde_json::to_value(&output).unwrap()["collections"][0],
            json!({
                "id": "work",
                "displayName": "Work",
                "description": "Meetings",
                "color": "#ABCDEF",
                "path": "/vdir/work",
            })
        );
    }
}
