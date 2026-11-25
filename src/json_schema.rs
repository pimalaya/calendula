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
    shared::receipt::WriteReceipt,
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

    #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
    macro_rules! refusable {
        ($key:expr, $ty:ty) => {
            registry.insert::<crate::shared::refusal::Refusable<$ty>>($key);
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
    #[cfg(feature = "pimdir")]
    insert!(
        "calendula-event-search",
        crate::shared::event::search::EventSearchOutput
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
        "calendula-calendar-create",
        crate::shared::calendar::create::CalendarCreateOutput
    );

    for key in [
        "calendula-calendar-update",
        "calendula-calendar-delete",
        "calendula-event-delete",
    ] {
        insert!(key, WriteReceipt);
    }

    #[cfg(feature = "caldav")]
    {
        refusable!(
            "calendula-caldav-discover",
            crate::caldav::discover::CaldavDiscoverOutput
        );
        refusable!(
            "calendula-caldav-propfind",
            crate::caldav::propfind::CaldavPropfindOutput
        );
        refusable!("calendula-caldav-proppatch", IdReceipt);
        refusable!("calendula-caldav-mkcalendar", IdReceipt);
        refusable!("calendula-caldav-rmcol", IdReceipt);
        for key in [
            "calendula-caldav-report-query",
            "calendula-caldav-report-multiget",
        ] {
            refusable!(key, crate::caldav::report::entries::CaldavItemEntriesOutput);
        }
        refusable!(
            "calendula-caldav-report-sync",
            crate::caldav::report::sync::CaldavReportSyncOutput
        );
        refusable!(
            "calendula-caldav-report-raw",
            crate::caldav::report::raw::CaldavReportRawOutput
        );
        refusable!("calendula-caldav-get", crate::caldav::get::CaldavGetOutput);
        refusable!("calendula-caldav-put", crate::caldav::put::CaldavPutOutput);
        refusable!("calendula-caldav-delete", IdReceipt);
    }

    #[cfg(feature = "msgraph")]
    {
        refusable!(
            "calendula-msgraph-calendars-list",
            crate::msgraph::calendars::render::MsgraphCalendarsOutput
        );
        for key in [
            "calendula-msgraph-calendars-get",
            "calendula-msgraph-calendars-create",
            "calendula-msgraph-calendars-rename",
        ] {
            refusable!(
                key,
                crate::msgraph::calendars::render::MsgraphCalendarOutput
            );
        }
        refusable!("calendula-msgraph-calendars-delete", IdReceipt);
        refusable!(
            "calendula-msgraph-events-list",
            crate::msgraph::events::render::MsgraphEventsOutput
        );
        for key in [
            "calendula-msgraph-events-get",
            "calendula-msgraph-events-create",
            "calendula-msgraph-events-update",
        ] {
            refusable!(key, crate::msgraph::events::render::MsgraphEventOutput);
        }
        refusable!("calendula-msgraph-events-delete", IdReceipt);
        refusable!(
            "calendula-msgraph-events-delta",
            crate::msgraph::events::delta::MsgraphEventDeltaOutput
        );
        refusable!(
            "calendula-msgraph-profile-get",
            crate::shared::raw_json::RawRead
        );
        refusable!(
            "calendula-msgraph-request",
            crate::shared::raw_json::RawJsonOutput
        );
    }

    #[cfg(feature = "gcal")]
    {
        refusable!(
            "calendula-gcal-calendars",
            crate::gcal::calendars::GcalCalendarListOutput
        );
        refusable!(
            "calendula-gcal-acl-list",
            crate::gcal::acl::list::GcalAclListOutput
        );
        refusable!("calendula-gcal-acl-create", IdReceipt);
        refusable!("calendula-gcal-acl-update", IdReceipt);
        refusable!("calendula-gcal-acl-delete", IdReceipt);
        // NOTE: the former names keep their schema beside the new ones.
        for key in [
            "calendula-gcal-events-move",
            "calendula-gcal-events-quick-add",
            "calendula-gcal-move",
            "calendula-gcal-quick-add",
        ] {
            refusable!(key, IdReceipt);
        }
        refusable!("calendula-gcal-free-busy", crate::shared::raw_json::RawRead);
        for key in [
            "calendula-gcal-events-instances",
            "calendula-gcal-instances",
        ] {
            refusable!(key, crate::gcal::events::instances::GcalInstancesOutput);
        }
        refusable!("calendula-gcal-colors", crate::shared::raw_json::RawRead);
        refusable!(
            "calendula-gcal-settings",
            crate::gcal::settings::GcalSettingsOutput
        );
        refusable!(
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
            "calendula-vdir-item-add",
            crate::vdir::item::add::VdirItemAddOutput
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
