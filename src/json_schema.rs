//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `calendula-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry.
//!
//! Every runnable command appears, a write by its receipt, and a command
//! printing one of two types is described by both, through `OneOf`. A
//! `build`, a `create` and an `update` answer data too, an abandoned edit
//! being an outcome they report.
//!
//! Protocol-specific entries are gated behind the same cargo features as
//! their command modules, so the registry stays coherent under any
//! feature combination, none included.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::JsonSchemaCommand

use pimalaya_cli::{clap::commands::JsonSchemaRegistry, error::ErrorReport};
use schemars::JsonSchema;

#[cfg(any(feature = "caldav", feature = "gcal", feature = "vdir"))]
use crate::shared::receipt::IdReceipt;
use crate::{
    error::ErrorCode,
    shared::{
        build::{IcalBuildOutput, SavedOutput},
        receipt::WriteReceipt,
    },
};

/// Builds the command-to-schema registry consumed by `json-schema <DIR>`.
///
/// Each value describes the type the command hands to the printer, and
/// `calendula-error` the failure object every command prints instead.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("calendula", env!("CARGO_PKG_VERSION"));

    macro_rules! insert {
        ($key:expr, $ty:ty) => {
            registry.insert::<$ty>($key);
        };
    }

    insert!(
        "calendula-account-list",
        crate::account::list::AccountListOutput
    );
    insert!(
        "calendula-account-check",
        crate::account::check::AccountCheckOutput
    );

    insert!(
        "calendula-calendar-list",
        crate::shared::calendar::list::CalendarListOutput
    );
    insert!(
        "calendula-event-agenda",
        crate::shared::event::agenda::EventAgendaOutput
    );
    insert!(
        "calendula-event-list",
        crate::shared::event::list::EventListOutput
    );
    insert!(
        "calendula-event-read",
        crate::shared::event::read::EventReadOutput
    );
    insert!(
        "calendula-event-find",
        crate::shared::event::list::EventListOutput
    );
    insert!(
        "calendula-item-list",
        crate::shared::item::list::ItemListOutput
    );
    insert!(
        "calendula-item-read",
        crate::shared::item::read::ItemReadOutput
    );

    // NOTE: a build prints the item, or under `-o` the file written.
    insert!(
        "calendula-event-build",
        OneOf<crate::shared::event::build::EventBuildOutput, SavedOutput>
    );
    insert!(
        "calendula-event-create",
        crate::shared::event::create::EventCreateOutput
    );
    insert!(
        "calendula-event-update",
        crate::shared::event::update::EventUpdateOutput
    );
    insert!(
        "calendula-item-build",
        OneOf<IcalBuildOutput, SavedOutput>
    );
    insert!(
        "calendula-item-create",
        crate::shared::item::create::ItemCreateOutput
    );
    insert!(
        "calendula-item-update",
        crate::shared::item::update::ItemUpdateOutput
    );

    for key in [
        "calendula-calendar-create",
        "calendula-calendar-update",
        "calendula-calendar-delete",
        "calendula-event-delete",
        "calendula-item-delete",
    ] {
        insert!(key, WriteReceipt);
    }

    #[cfg(feature = "caldav")]
    {
        insert!(
            "calendula-caldav-discover",
            crate::caldav::discover::CaldavDiscoverOutput
        );
        insert!(
            "calendula-caldav-list",
            crate::caldav::list::CaldavCalendarListOutput
        );
        insert!("calendula-caldav-create", IdReceipt);
        insert!("calendula-caldav-delete", IdReceipt);
    }

    #[cfg(feature = "gcal")]
    {
        insert!(
            "calendula-gcal-calendars",
            crate::gcal::calendars::GcalCalendarListOutput
        );
        insert!(
            "calendula-gcal-acl-list",
            crate::gcal::acl::list::GcalAclListOutput
        );
        insert!("calendula-gcal-acl-create", IdReceipt);
        insert!("calendula-gcal-acl-update", IdReceipt);
        insert!("calendula-gcal-acl-delete", IdReceipt);
        insert!("calendula-gcal-move", IdReceipt);
        insert!("calendula-gcal-quick-add", IdReceipt);
        insert!(
            "calendula-gcal-free-busy",
            crate::gcal::free_busy::GcalFreeBusyOutput
        );
        insert!(
            "calendula-gcal-instances",
            crate::gcal::instances::GcalInstancesOutput
        );
        insert!(
            "calendula-gcal-colors",
            crate::gcal::colors::GcalColorsOutput
        );
        insert!(
            "calendula-gcal-settings",
            crate::gcal::settings::GcalSettingsOutput
        );
    }

    #[cfg(feature = "pimdir")]
    insert!(
        "calendula-pimdir-status",
        crate::pimdir::status::PimdirStatusOutput
    );
    #[cfg(feature = "pimdir")]
    insert!(
        "calendula-pimdir-reply",
        crate::pimdir::intent::PimdirIntentOutput
    );
    #[cfg(feature = "pimdir")]
    insert!(
        "calendula-pimdir-cancel",
        crate::pimdir::intent::PimdirIntentOutput
    );
    #[cfg(feature = "pimdir")]
    insert!(
        "calendula-pimdir-search",
        crate::pimdir::search::PimdirSearchOutput
    );
    #[cfg(feature = "pimdir")]
    insert!(
        "calendula-pimdir-count",
        crate::pimdir::count::PimdirCountOutput
    );

    #[cfg(feature = "vdir")]
    {
        insert!(
            "calendula-vdir-list",
            crate::vdir::list::VdirCollectionListOutput
        );
        insert!("calendula-vdir-create", IdReceipt);
        insert!("calendula-vdir-rename", IdReceipt);
        insert!("calendula-vdir-delete", IdReceipt);
    }

    insert!("calendula-error", ErrorReport<ErrorCode>);

    registry
}

/// The schema of a command printing one of two types, told apart by their
/// keys.
#[derive(JsonSchema)]
#[serde(untagged)]
#[allow(dead_code)]
enum OneOf<A, B> {
    A(A),
    B(B),
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::{cli::CalendulaCli, json_schema::registry};

    /// The commands printing nothing a schema could describe.
    const SILENT: &[&str] = &["calendula"];

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<String>::new());
    }

    #[test]
    fn every_command_is_described() {
        let problems = registry().check(&CalendulaCli::command(), SILENT);
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
