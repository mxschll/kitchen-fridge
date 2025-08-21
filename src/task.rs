//! To-do tasks (iCal `VTODO` item)

use std::fmt::Display;

use chrono::{DateTime, Utc};
use ical::property::Property;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::utils::{
    random_url,
    sync::{SyncStatus, Syncable},
};

/// RFC5545 defines the completion as several optional fields, yet some combinations make no sense.
/// This enum provides an API that forbids such impossible combinations.
///
/// * `COMPLETED` is an optional timestamp that tells whether this task is completed
/// * `STATUS` is an optional field, that can be set to `NEEDS-ACTION`, `COMPLETED`, or others.
///
/// Even though having a `COMPLETED` date but a `STATUS:NEEDS-ACTION` is theorically possible, it obviously makes no sense. This API ensures this cannot happen
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CompletionStatus {
    Completed(Option<DateTime<Utc>>),
    Uncompleted,
}
impl CompletionStatus {
    pub fn is_completed(&self) -> bool {
        matches!(self, CompletionStatus::Completed(_))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relationship {
    /// The ical RELATED-TO property, see https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.5
    ///
    /// This is the UID of a task to which this task is related.
    related_to: String,

    /// The ical RELTYPE parameter as found on a RELATED-TO property.
    ///
    /// See https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.15
    reltype: String,
}
impl Relationship {
    pub fn new(related_to: String, reltype: String) -> Self {
        Self {
            related_to,
            reltype,
        }
    }
}
impl Display for Relationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.reltype.as_str() {
            "PARENT" => {}
            x => {
                f.write_str("RELTYPE=")?;
                f.write_str(x)?;
                f.write_str(":")?;
            }
        }

        f.write_str(self.related_to.as_str())
    }
}

/// A to-do task
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    /// The task URL
    url: Url,

    /// Persistent, globally unique identifier for the calendar component
    /// The [RFC](https://tools.ietf.org/html/rfc5545#page-117) recommends concatenating a timestamp with the server's domain name.
    /// UUID are even better so we'll generate them, but we have to support tasks from the server, that may have any arbitrary strings here.
    uid: String,

    /// The sync status of this item
    sync_status: SyncStatus,
    /// The time this item was created.
    /// This is not required by RFC5545. This will be populated in tasks created by this crate, but can be None for tasks coming from a server
    creation_date: Option<DateTime<Utc>>,
    /// The last time this item was modified
    last_modified: DateTime<Utc>,
    /// The completion status of this task
    completion_status: CompletionStatus,

    /// The display name of the task
    name: String,

    /// The PRODID, as defined in iCal files
    ical_prod_id: String,

    /// Related items, derived from the RELATED-TO property.
    relationships: Vec<Relationship>,

    /// Extra parameters that have not been parsed from the iCal file (because they're not supported (yet) by this crate).
    /// They are needed to serialize this item into an equivalent iCal file
    extra_parameters: Vec<Property>,
}

impl Task {
    /// Create a brand new Task that is not on a server yet.
    /// This will pick a new (random) task ID.
    pub fn new(name: String, completed: bool, parent_calendar_url: &Url) -> Self {
        let new_url = random_url(parent_calendar_url);
        let new_sync_status = SyncStatus::NotSynced;
        let new_uid = Uuid::new_v4().to_hyphenated().to_string();
        let new_creation_date = Some(Utc::now());
        let new_last_modified = Utc::now();
        let new_completion_status = if completed {
            CompletionStatus::Completed(Some(Utc::now()))
        } else {
            CompletionStatus::Uncompleted
        };
        let ical_prod_id = crate::ical::default_prod_id();
        let extra_parameters = Vec::new();
        Self::new_with_parameters(
            name,
            new_uid,
            new_url,
            new_completion_status,
            new_sync_status,
            new_creation_date,
            new_last_modified,
            ical_prod_id,
            Vec::new(),
            extra_parameters,
        )
    }

    /// Create a new Task instance, that may be synced on the server already
    pub fn new_with_parameters(
        name: String,
        uid: String,
        new_url: Url,
        completion_status: CompletionStatus,
        sync_status: SyncStatus,
        creation_date: Option<DateTime<Utc>>,
        last_modified: DateTime<Utc>,
        ical_prod_id: String,
        relationships: Vec<Relationship>,
        extra_parameters: Vec<Property>,
    ) -> Self {
        Self {
            url: new_url,
            uid,
            name,
            completion_status,
            sync_status,
            creation_date,
            last_modified,
            ical_prod_id,
            relationships,
            extra_parameters,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn uid(&self) -> &str {
        &self.uid
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn completed(&self) -> bool {
        self.completion_status.is_completed()
    }
    pub fn ical_prod_id(&self) -> &str {
        &self.ical_prod_id
    }
    pub fn last_modified(&self) -> &DateTime<Utc> {
        &self.last_modified
    }
    pub fn creation_date(&self) -> Option<&DateTime<Utc>> {
        self.creation_date.as_ref()
    }
    pub fn completion_status(&self) -> &CompletionStatus {
        &self.completion_status
    }
    pub fn relationships(&self) -> &Vec<Relationship> {
        &self.relationships
    }
    /// The UID of the parent of this task, if any
    pub fn parent(&self) -> Option<&String> {
        self.relationships
            .iter()
            .find(|r| r.reltype == "PARENT")
            .map(|r| &r.related_to)
    }
    pub fn set_parent(&mut self, parent_uid: String) {
        match self.parent().cloned() {
            Some(parent) => {
                self.relationships
                    .iter_mut()
                    .find(|r| r.reltype == "PARENT" && r.related_to == parent)
                    .unwrap()
                    .related_to = parent_uid;
            }
            None => {
                self.relationships
                    .push(Relationship::new(parent_uid, "PARENT".to_string()));
            }
        }
    }
    pub fn extra_parameters(&self) -> &[Property] {
        &self.extra_parameters
    }

    #[cfg(any(test, feature = "integration_tests"))]
    pub fn has_same_observable_content_as(&self, other: &Task) -> bool {
        self.url == other.url
        && self.uid == other.uid
        && self.name == other.name
        // sync status must be the same variant, but we ignore its embedded version tag
        && std::mem::discriminant(&self.sync_status) == std::mem::discriminant(&other.sync_status)
        // completion status must be the same variant, but we ignore its embedded completion date (they are not totally mocked in integration tests)
        && std::mem::discriminant(&self.completion_status) == std::mem::discriminant(&other.completion_status)
        // last modified dates are ignored (they are not totally mocked in integration tests)
    }

    fn update_last_modified(&mut self) {
        self.last_modified = Utc::now();
    }

    /// Rename a task.
    /// This updates its "last modified" field
    pub fn set_name(&mut self, new_name: String) {
        self.mark_modified_since_last_sync();
        self.update_last_modified();
        self.name = new_name;
    }
    #[cfg(feature = "local_calendar_mocks_remote_calendars")]
    /// Rename a task, but forces a "master" SyncStatus, just like CalDAV servers are always "masters"
    pub fn mock_remote_calendar_set_name(&mut self, new_name: String) {
        self.sync_status = SyncStatus::random_synced();
        self.update_last_modified();
        self.name = new_name;
    }

    /// Set the completion status
    pub fn set_completion_status(&mut self, new_completion_status: CompletionStatus) {
        self.mark_modified_since_last_sync();
        self.update_last_modified();
        self.completion_status = new_completion_status;
    }
    #[cfg(feature = "local_calendar_mocks_remote_calendars")]
    /// Set the completion status, but forces a "master" SyncStatus, just like CalDAV servers are always "masters"
    pub fn mock_remote_calendar_set_completion_status(
        &mut self,
        new_completion_status: CompletionStatus,
    ) {
        self.sync_status = SyncStatus::random_synced();
        self.completion_status = new_completion_status;
    }
}

impl Syncable for Task {
    fn value(&self) -> &String {
        &self.name
    }

    fn sync_status(&self) -> &SyncStatus {
        &self.sync_status
    }

    fn set_sync_status(&mut self, new_status: SyncStatus) {
        self.sync_status = new_status;
    }
}
