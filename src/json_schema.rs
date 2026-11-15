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
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::json_schema::JsonSchemaCommand

use pimalaya_cli::{
    clap::commands::json_schema::JsonSchemaRegistry, error::ErrorReport, output::SavedOutput,
};
use schemars::JsonSchema;

#[cfg(any(
    feature = "caldav",
    feature = "gcal",
    feature = "msgraph",
    feature = "vdir"
))]
use crate::shared::receipt::IdReceipt;
use crate::{
    error::{ErrorCode, ItemError},
    shared::{build::IcalBuildOutput, receipt::WriteReceipt},
};

/// Builds the command-to-schema registry consumed by `json-schema <DIR>`.
///
/// Each value describes the type the command hands to the printer, and
/// `calendula-error` the failure object every command prints instead, and
/// `calendula-item-error` an entry of the `errors` a listing prints.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("calendula", env!("CARGO_PKG_VERSION"));

    macro_rules! insert {
        ($key:expr, $ty:ty) => {
            registry.insert::<$ty>($key);
        };
    }

    insert!(
        "calendula-account-list",
        pimalaya_cli::account::AccountListOutput
    );
    insert!(
        "calendula-account-check",
        pimalaya_cli::account::AccountCheckOutput<ErrorCode>
    );
    insert!(
        "calendula-account-capability",
        pimalaya_cli::capability::AccountCapabilityOutput
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
        "calendula-event-search",
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
        "calendula-event-add",
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
        "calendula-item-add",
        crate::shared::item::create::ItemCreateOutput
    );
    insert!(
        "calendula-item-update",
        crate::shared::item::update::ItemUpdateOutput
    );

    insert!(
        "calendula-calendar-create",
        crate::shared::calendar::create::CalendarCreateOutput
    );

    for key in [
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
            "calendula-caldav-propfind",
            crate::caldav::propfind::CaldavPropfindOutput
        );
        insert!("calendula-caldav-proppatch", IdReceipt);
        insert!("calendula-caldav-mkcalendar", IdReceipt);
        for key in [
            "calendula-caldav-report-query",
            "calendula-caldav-report-multiget",
        ] {
            insert!(key, crate::caldav::report::entries::CaldavItemEntriesOutput);
        }
        insert!(
            "calendula-caldav-report-sync",
            crate::caldav::report::sync::CaldavReportSyncOutput
        );
        insert!(
            "calendula-caldav-report-raw",
            crate::caldav::report::raw::CaldavReportRawOutput
        );
        insert!("calendula-caldav-get", crate::caldav::get::CaldavGetOutput);
        insert!("calendula-caldav-put", crate::caldav::put::CaldavPutOutput);
        insert!("calendula-caldav-delete", IdReceipt);
    }

    #[cfg(feature = "msgraph")]
    {
        insert!(
            "calendula-msgraph-calendars-list",
            crate::msgraph::calendars::render::MsgraphCalendarsOutput
        );
        for key in [
            "calendula-msgraph-calendars-get",
            "calendula-msgraph-calendars-create",
            "calendula-msgraph-calendars-rename",
        ] {
            insert!(
                key,
                crate::msgraph::calendars::render::MsgraphCalendarOutput
            );
        }
        insert!("calendula-msgraph-calendars-delete", IdReceipt);
        insert!(
            "calendula-msgraph-events-list",
            crate::msgraph::events::render::MsgraphEventsOutput
        );
        for key in [
            "calendula-msgraph-events-get",
            "calendula-msgraph-events-create",
            "calendula-msgraph-events-update",
        ] {
            insert!(key, crate::msgraph::events::render::MsgraphEventOutput);
        }
        insert!("calendula-msgraph-events-delete", IdReceipt);
        insert!(
            "calendula-msgraph-events-delta",
            crate::msgraph::events::delta::MsgraphEventDeltaOutput
        );
        insert!(
            "calendula-msgraph-profile-get",
            crate::msgraph::profile::get::MsgraphProfileGetOutput
        );
        insert!(
            "calendula-msgraph-request",
            crate::shared::raw_json::RawJsonOutput
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
        insert!("calendula-gcal-acl-create", IdReceipt);
        insert!("calendula-gcal-acl-update", IdReceipt);
        insert!("calendula-gcal-acl-delete", IdReceipt);
        // NOTE: the former names keep their schema beside the new ones.
        for key in [
            "calendula-gcal-events-move",
            "calendula-gcal-events-quick-add",
            "calendula-gcal-move",
            "calendula-gcal-quick-add",
        ] {
            insert!(key, IdReceipt);
        }
        insert!(
            "calendula-gcal-free-busy",
            crate::gcal::free_busy::GcalFreeBusyOutput
        );
        for key in [
            "calendula-gcal-events-instances",
            "calendula-gcal-instances",
        ] {
            insert!(key, crate::gcal::events::instances::GcalInstancesOutput);
        }
        insert!(
            "calendula-gcal-colors",
            crate::gcal::colors::GcalColorsOutput
        );
        insert!(
            "calendula-gcal-settings",
            crate::gcal::settings::GcalSettingsOutput
        );
        insert!(
            "calendula-gcal-request",
            crate::shared::raw_json::RawJsonOutput
        );
    }

    #[cfg(feature = "pimdir")]
    insert!(
        "calendula-pimdir-status",
        crate::pimdir::status::PimdirStatusOutput
    );
    #[cfg(feature = "pimdir")]
    {
        insert!(
            "calendula-pimdir-queue-list",
            crate::pimdir::queue::list::PimdirQueueListOutput
        );
        insert!(
            "calendula-pimdir-queue-show",
            crate::pimdir::queue::show::PimdirQueueShowOutput
        );
        insert!(
            "calendula-pimdir-queue-cancel",
            crate::pimdir::queue::cancel::PimdirQueueCancelOutput
        );
    }
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
        insert!(
            "calendula-vdir-item-list",
            crate::vdir::item::list::VdirItemListOutput
        );
        insert!(
            "calendula-vdir-item-get",
            crate::vdir::item::get::VdirItemGetOutput
        );
        insert!(
            "calendula-vdir-item-create",
            crate::vdir::item::create::VdirItemCreateOutput
        );
        for key in [
            "calendula-vdir-item-update",
            "calendula-vdir-item-delete",
            "calendula-vdir-item-copy",
            "calendula-vdir-item-move",
        ] {
            insert!(key, IdReceipt);
        }
    }

    insert!("calendula-error", ErrorReport<ErrorCode>);
    insert!("calendula-item-error", ItemError);

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

    use crate::{cli::Cli, json_schema::registry};

    /// The commands printing nothing a schema could describe.
    const SILENT: &[&str] = &["calendula", "calendula-repl", "calendula-mcp"];

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<String>::new());
    }

    #[test]
    fn every_command_is_described() {
        let problems = registry().check(&Cli::command(), SILENT);
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
