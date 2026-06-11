use anyhow::Result;
use clap::Parser;
use io_vdir::{collection::Collection, path::VdirPath};
use pimalaya_cli::printer::{Message, Printer};

use crate::vdir::client::VdirClient;

/// Create a vdir collection directory.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionCreateCommand {
    /// Collection identifier (directory name).
    #[arg(value_name = "ID")]
    pub id: String,

    /// Optional display name written to the `displayname` metadata
    /// file.
    #[arg(short, long, value_name = "NAME")]
    pub display_name: Option<String>,

    /// Optional description written to the `description` metadata
    /// file.
    #[arg(short = 'D', long, value_name = "TEXT")]
    pub description: Option<String>,

    /// Optional hex color (`#RRGGBB`) written to the `color` metadata
    /// file.
    #[arg(long, value_name = "HEX")]
    pub color: Option<String>,
}

impl VdirCollectionCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = VdirPath::new(client.root().as_str()).join(&self.id);
        let collection = Collection {
            path,
            display_name: self.display_name,
            description: self.description,
            color: self.color,
        };

        client.create_collection(collection)?;

        let msg = format!("Collection `{}` successfully created", self.id);
        printer.out(Message::new(msg))
    }
}
