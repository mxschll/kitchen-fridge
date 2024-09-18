use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    sync::{SyncStatus, Syncable},
    NamespacedName,
};

lazy_static::lazy_static! {
    // WebDAV properties
    pub(crate) static ref PROP_DISPLAY_NAME: NamespacedName = NamespacedName::new("DAV:", "displayname");
    pub(crate) static ref PROP_RESOURCE_TYPE: NamespacedName = NamespacedName::new("DAV:", "resourcetype");
    pub(crate) static ref PROP_ALLPROP: NamespacedName = NamespacedName::new("DAV:", "allprop");

    // CalDAV properties
    pub(crate) static ref PROP_SUPPORTED_CALENDAR_COMPONENT_SET: NamespacedName = NamespacedName::new("urn:ietf:params:xml:ns:caldav", "supported-calendar-component-set");

    // iCal properties
    pub(crate) static ref PROP_CALENDAR_COLOR: NamespacedName = NamespacedName::new("http://apple.com/ns/ical/", "calendar-color");
}
/// A WebDAV property.
///
/// Similar to ical Property but allowing arbitrary namespaces and tracking of sync status
/// This should allow for user-defined properties
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct Property {
    nsn: NamespacedName,
    value: String,
    sync_status: SyncStatus,
}

impl Property {
    /// Defaults sync state to SyncStatus::default(), i.e. NotSynced
    pub fn new<S1: ToString, S2: ToString>(xmlns: S1, name: S2, value: String) -> Self {
        Self {
            nsn: NamespacedName {
                xmlns: xmlns.to_string(),
                name: name.to_string(),
            },
            value,
            sync_status: SyncStatus::default(),
        }
    }

    pub fn nsn(&self) -> &NamespacedName {
        &self.nsn
    }

    pub fn xmlns(&self) -> &str {
        self.nsn.xmlns.as_str()
    }

    pub fn name(&self) -> &str {
        self.nsn.name.as_str()
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn set_value(&mut self, new_value: String) {
        self.mark_modified_since_last_sync();
        self.value = new_value;
    }

    pub fn mark_for_deletion(&mut self) {
        self.sync_status = SyncStatus::LocallyDeleted(self.value.clone().into());
    }

    /// Set property value, but forces a "master" SyncStatus, just like CalDAV servers are always "masters"
    #[cfg(feature = "local_calendar_mocks_remote_calendars")]
    pub fn mock_remote_calendar_set_value(&mut self, new_value: String) {
        // self.sync_status = SyncStatus::random_synced();

        // self.update_last_modified();
        self.value = new_value;
        self.mark_synced();
    }
}
impl Syncable for Property {
    fn value(&self) -> &String {
        &self.value
    }

    fn sync_status(&self) -> &SyncStatus {
        &self.sync_status
    }

    fn set_sync_status(&mut self, new_status: SyncStatus) {
        self.sync_status = new_status;
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.xmlns())?;

        fmt::Write::write_char(f, ':')?;

        f.write_str(self.name())?;

        fmt::Write::write_char(f, '=')?;

        f.write_str(self.value.as_str())?;

        f.write_str("; ")?;

        write!(f, "{}", self.sync_status)
    }
}

pub fn print_property(prop: &Property) {
    let sync = prop.sync_status.symbol();
    println!("     {} prop {}", sync, prop);
}
