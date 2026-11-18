//! # Calendar/set
//!
//! `Calendar/set` (draft-ietf-jmap-calendars), which io-jmap 0.10 does not
//! cover, sent through [`JmapClientStd::send_raw`]: one create, update or
//! destroy at a time. It gives way to io-jmap's own `JmapCalendarSet` the
//! day a release carries it.

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow};
use io_jmap::{
    calendars::JMAP_CALENDARS_CAPABILITY,
    client::JmapClientStd,
    rfc8620::{JMAP_CORE_CAPABILITY, error::JmapMethodError, request::JmapBatch},
};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::jmap::error::set_error;

/// The client id a create names its one calendar by.
const CREATE_ID: &str = "c0";

/// A per-object error of a `Calendar/set` response (RFC 8620 section
/// 5.3).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct JmapCalendarSetItemError {
    /// The error type, empty when the server named none.
    #[serde(default, rename = "type")]
    pub type_name: String,
    /// The optional human-readable detail.
    #[serde(default)]
    pub description: Option<String>,
    /// The invalid property names of an `invalidProperties`.
    #[serde(default)]
    pub properties: Vec<String>,
}

/// The part of a `Calendar/set` response the writes read.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarSetResponse {
    #[serde(default)]
    created: Option<BTreeMap<String, Map<String, Value>>>,
    #[serde(default)]
    not_created: Option<BTreeMap<String, JmapCalendarSetItemError>>,
    #[serde(default)]
    not_updated: Option<BTreeMap<String, JmapCalendarSetItemError>>,
    #[serde(default)]
    not_destroyed: Option<BTreeMap<String, JmapCalendarSetItemError>>,
}

/// Creates the calendar `calendar` describes, returning the id the
/// server minted.
pub fn create(client: &mut JmapClientStd, calendar: Map<String, Value>) -> Result<String> {
    let args = json!({"create": {CREATE_ID: calendar}});
    let mut out = send(client, args)?;

    if let Some(err) = out.not_created.and_then(|mut errs| errs.remove(CREATE_ID)) {
        return Err(set_error("JMAP Calendar create rejected", &err));
    }

    out.created
        .as_mut()
        .and_then(|created| created.remove(CREATE_ID))
        .and_then(|created| created.get("id")?.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| anyhow!("JMAP Calendar/set answered no id for the calendar created"))
}

/// Applies `patch`, a PatchObject, to the calendar `id`.
pub fn update(client: &mut JmapClientStd, id: &str, patch: Map<String, Value>) -> Result<()> {
    let args = json!({"update": {id: patch}});
    let out = send(client, args)?;

    if let Some(err) = out.not_updated.and_then(|mut errs| errs.remove(id)) {
        return Err(set_error("JMAP Calendar update rejected", &err));
    }

    Ok(())
}

/// Destroys the calendar `id` and the events it holds
/// (`onDestroyRemoveEvents`).
pub fn destroy(client: &mut JmapClientStd, id: &str) -> Result<()> {
    let args = json!({"destroy": [id], "onDestroyRemoveEvents": true});
    let out = send(client, args)?;

    if let Some(err) = out.not_destroyed.and_then(|mut errs| errs.remove(id)) {
        return Err(set_error("JMAP Calendar destroy rejected", &err));
    }

    Ok(())
}

/// Sends one `Calendar/set` call of `args`, addressed to the calendar
/// account, and reads its response, a method error failing it.
fn send(client: &mut JmapClientStd, mut args: Value) -> Result<CalendarSetResponse> {
    let session = client
        .session()
        .ok_or_else(|| anyhow!("The JMAP session is not open"))?;
    let account_id = session.account_for(JMAP_CALENDARS_CAPABILITY)?.to_owned();
    args["accountId"] = Value::String(account_id);

    let mut batch = JmapBatch::new();
    batch.add("Calendar/set", args);
    let using = vec![
        JMAP_CORE_CAPABILITY.to_owned(),
        JMAP_CALENDARS_CAPABILITY.to_owned(),
    ];
    let response = client.send_raw(batch.into_request(using))?;

    let (name, args, _) = response
        .method_responses
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("JMAP Calendar/set answered no method response"))?;

    if name == "error" {
        let err: JmapMethodError = serde_json::from_str(args.get())
            .context("Cannot read the method error JMAP Calendar/set answered")?;
        return Err(anyhow::Error::new(err).context("JMAP Calendar/set failed"));
    }

    serde_json::from_str(args.get()).context("Cannot read the JMAP Calendar/set response")
}
