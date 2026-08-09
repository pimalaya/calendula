//! Wire spellings shared by the `gcal` subcommands.
//!
//! The Calendar API names its access roles and ACL scope types in
//! lowerCamelCase, which is what a listing prints and what a flag
//! takes, so the mapping is written once here rather than in each
//! command.

use anyhow::{Result, bail};
use io_gcal::v3::rest::{
    acl::{GcalAccessRole, GcalAclScopeType},
    events::{GcalEventDateTime, GcalEventStatus},
};

/// The wire spelling of an event status.
pub fn event_status(status: GcalEventStatus) -> &'static str {
    match status {
        GcalEventStatus::Confirmed => "confirmed",
        GcalEventStatus::Tentative => "tentative",
        GcalEventStatus::Cancelled => "cancelled",
    }
}

/// A boundary as one printable stamp: the timestamp of a timed
/// occurrence, the date of an all-day one, empty when it carries
/// neither.
pub fn boundary(boundary: Option<&GcalEventDateTime>) -> String {
    boundary
        .and_then(|boundary| boundary.date_time.clone().or_else(|| boundary.date.clone()))
        .unwrap_or_default()
}

/// The wire spelling of an access role.
pub fn access_role(role: GcalAccessRole) -> &'static str {
    match role {
        GcalAccessRole::None => "none",
        GcalAccessRole::FreeBusyReader => "freeBusyReader",
        GcalAccessRole::Reader => "reader",
        GcalAccessRole::WriterWithoutPrivateAccess => "writerWithoutPrivateAccess",
        GcalAccessRole::Writer => "writer",
        GcalAccessRole::Owner => "owner",
    }
}

/// Reads an access role from a flag, case-insensitively, naming every
/// accepted value when it does not match.
pub fn parse_access_role(value: &str) -> Result<GcalAccessRole> {
    const ROLES: [GcalAccessRole; 6] = [
        GcalAccessRole::None,
        GcalAccessRole::FreeBusyReader,
        GcalAccessRole::Reader,
        GcalAccessRole::WriterWithoutPrivateAccess,
        GcalAccessRole::Writer,
        GcalAccessRole::Owner,
    ];

    for role in ROLES {
        if value.eq_ignore_ascii_case(access_role(role)) {
            return Ok(role);
        }
    }

    let accepted: Vec<&str> = ROLES.into_iter().map(access_role).collect();
    bail!(
        "Invalid role `{value}`, expected one of {}",
        accepted.join(", ")
    )
}

/// The wire spelling of an ACL scope type.
pub fn scope_type(scope: GcalAclScopeType) -> &'static str {
    match scope {
        GcalAclScopeType::Default => "default",
        GcalAclScopeType::User => "user",
        GcalAclScopeType::Group => "group",
        GcalAclScopeType::Domain => "domain",
    }
}

/// Reads an ACL scope type from a flag, case-insensitively.
pub fn parse_scope_type(value: &str) -> Result<GcalAclScopeType> {
    const SCOPES: [GcalAclScopeType; 4] = [
        GcalAclScopeType::Default,
        GcalAclScopeType::User,
        GcalAclScopeType::Group,
        GcalAclScopeType::Domain,
    ];

    for scope in SCOPES {
        if value.eq_ignore_ascii_case(scope_type(scope)) {
            return Ok(scope);
        }
    }

    let accepted: Vec<&str> = SCOPES.into_iter().map(scope_type).collect();
    bail!(
        "Invalid scope `{value}`, expected one of {}",
        accepted.join(", ")
    )
}

/// An iCalendar UTC stamp (`YYYYMMDDTHHMMSSZ`) as the RFC 3339
/// timestamp the Calendar API takes, so `--from` / `--to` are spelled
/// the same way across the whole CLI.
pub fn rfc3339(stamp: &str) -> String {
    let bytes = stamp.as_bytes();

    let shaped = bytes.len() == 16
        && bytes[8] == b'T'
        && bytes[15] == b'Z'
        && bytes[..8].iter().all(u8::is_ascii_digit)
        && bytes[9..15].iter().all(u8::is_ascii_digit);

    if !shaped {
        return stamp.to_owned();
    }

    format!(
        "{}-{}-{}T{}:{}:{}Z",
        &stamp[..4],
        &stamp[4..6],
        &stamp[6..8],
        &stamp[9..11],
        &stamp[11..13],
        &stamp[13..15],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_round_trips_through_its_wire_spelling() {
        for role in [
            GcalAccessRole::None,
            GcalAccessRole::FreeBusyReader,
            GcalAccessRole::Reader,
            GcalAccessRole::WriterWithoutPrivateAccess,
            GcalAccessRole::Writer,
            GcalAccessRole::Owner,
        ] {
            assert_eq!(parse_access_role(access_role(role)).unwrap(), role);
        }

        // Case-insensitive, and an unknown value names what is accepted.
        assert_eq!(parse_access_role("OWNER").unwrap(), GcalAccessRole::Owner);
        let err = parse_access_role("admin").unwrap_err().to_string();
        assert!(err.contains("owner"), "unexpected error: {err}");
    }

    #[test]
    fn every_scope_round_trips_through_its_wire_spelling() {
        for scope in [
            GcalAclScopeType::Default,
            GcalAclScopeType::User,
            GcalAclScopeType::Group,
            GcalAclScopeType::Domain,
        ] {
            assert_eq!(parse_scope_type(scope_type(scope)).unwrap(), scope);
        }

        assert!(parse_scope_type("everyone").is_err());
    }

    #[test]
    fn an_ical_utc_stamp_becomes_the_rfc_3339_form_and_anything_else_passes_through() {
        assert_eq!(rfc3339("20260801T000000Z"), "2026-08-01T00:00:00Z");
        assert_eq!(rfc3339("20260801"), "20260801");
        assert_eq!(rfc3339(""), "");
    }
}
