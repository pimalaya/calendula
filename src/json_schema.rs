//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `calendula-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry.
//!
//! Only a data command appears: a command answering a confirmation
//! prints a `Message`, whose shape is the printer's rather than
//! calendula's. A `build`, a `create` and an `update` answer data too,
//! an abandoned edit being an outcome they report.
//!
//! Protocol-specific entries are gated behind the same cargo features as
//! their command modules, so the registry stays coherent under any
//! feature combination, none included.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::JsonSchemaCommand

use std::collections::BTreeMap;

use schemars::schema_for;
use serde_json::Value;

/// Builds the command-to-schema map consumed by `json-schema <DIR>`.
///
/// Each value describes the type the command hands to the printer.
pub fn schemas() -> BTreeMap<String, Value> {
    let mut schemas = BTreeMap::new();

    macro_rules! insert {
        ($key:expr, $ty:ty) => {
            schemas.insert(
                $key.to_string(),
                serde_json::to_value(schema_for!($ty)).unwrap(),
            );
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
        "calendula-todo-list",
        crate::shared::todo::list::TodoListOutput
    );
    insert!(
        "calendula-todo-read",
        crate::shared::todo::read::TodoReadOutput
    );
    insert!(
        "calendula-journal-list",
        crate::shared::journal::list::JournalListOutput
    );
    insert!(
        "calendula-journal-read",
        crate::shared::journal::read::JournalReadOutput
    );
    insert!(
        "calendula-item-list",
        crate::shared::item::list::ItemListOutput
    );
    insert!(
        "calendula-item-read",
        crate::shared::item::read::ItemReadOutput
    );

    insert!(
        "calendula-event-build",
        crate::shared::build::IcalBuildOutput
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
        "calendula-todo-build",
        crate::shared::build::IcalBuildOutput
    );
    insert!(
        "calendula-todo-create",
        crate::shared::todo::create::TodoCreateOutput
    );
    insert!(
        "calendula-todo-update",
        crate::shared::todo::update::TodoUpdateOutput
    );
    insert!(
        "calendula-journal-build",
        crate::shared::build::IcalBuildOutput
    );
    insert!(
        "calendula-journal-create",
        crate::shared::journal::create::JournalCreateOutput
    );
    insert!(
        "calendula-journal-update",
        crate::shared::journal::update::JournalUpdateOutput
    );
    insert!(
        "calendula-item-build",
        crate::shared::build::IcalBuildOutput
    );
    insert!(
        "calendula-item-create",
        crate::shared::item::create::ItemCreateOutput
    );
    insert!(
        "calendula-item-update",
        crate::shared::item::update::ItemUpdateOutput
    );

    #[cfg(all(
        feature = "wizard",
        any(feature = "caldav", feature = "vdir", feature = "pimdir")
    ))]
    insert!(
        "calendula-configure",
        crate::wizard::configure::ConfigureOutput
    );

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

    #[cfg(feature = "vdir")]
    insert!(
        "calendula-vdir-list",
        crate::vdir::list::VdirCollectionListOutput
    );

    schemas
}
