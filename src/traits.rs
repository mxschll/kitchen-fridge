//! Traits used by multiple structs in this crate

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use csscolorparser::Color;
use tokio::sync::Mutex;
use url::Url;

use crate::calendar::SupportedComponents;
use crate::error::KFResult;
use crate::item::Item;
use crate::resource::Resource;
use crate::utils::prop::Property;
use crate::utils::sync::{SyncStatus, VersionTag};
use crate::utils::NamespacedName;

/// This trait must be implemented by data sources (either local caches or remote CalDAV clients)
///
/// Note that some concrete types (e.g. [`crate::cache::Cache`]) can also provide non-async versions of these functions
#[async_trait]
pub trait CalDavSource<T: BaseCalendar> {
    /// Returns the current calendars that this source contains
    /// This function may trigger an update (that can be a long process, or that can even fail, e.g. in case of a remote server)
    async fn get_calendars(&self) -> KFResult<HashMap<Url, Arc<Mutex<T>>>>;
    /// Returns the calendar matching the URL
    async fn get_calendar(&self, url: &Url) -> Option<Arc<Mutex<T>>>;
    /// Create a calendar if it did not exist, and return it
    async fn create_calendar(
        &mut self,
        url: Url,
        name: String,
        supported_components: SupportedComponents,
        color: Option<Color>,
    ) -> KFResult<Arc<Mutex<T>>>;

    /// Delete the calendar with the given URL within the source.
    ///
    /// Returns a copy of the calendar deleted if available.
    ///
    /// Returns Err if the calendar is not found in the source.
    async fn delete_calendar(&mut self, url: &Url) -> KFResult<Option<Arc<Mutex<T>>>>;
}

/// This trait contains functions that are common to all calendars
///
/// Note that some concrete types (e.g. [`crate::calendar::cached_calendar::CachedCalendar`]) can also provide non-async versions of these functions
#[async_trait]
pub trait BaseCalendar {
    /// Returns the calendar name
    fn name(&self) -> &str;

    /// Returns the calendar URL
    fn url(&self) -> &Url;

    /// Returns the supported kinds of components for this calendar
    fn supported_components(&self) -> crate::calendar::SupportedComponents;

    /// Returns the user-defined color of this calendar
    fn color(&self) -> Option<&Color>;

    /// Add an item into this calendar, and return its new sync status.
    /// For local calendars, the sync status is not modified.
    /// For remote calendars, the sync status is updated by the server
    async fn add_item(&mut self, item: Item) -> KFResult<SyncStatus>;

    /// Update an item that already exists in this calendar and returns its new `SyncStatus`
    /// This replaces a given item at a given URL
    async fn update_item(&mut self, item: Item) -> KFResult<SyncStatus>;

    /// Returns the requested WebDAV properties of the calendar collection.
    async fn get_properties_by_name(
        &self,
        names: &[NamespacedName],
    ) -> KFResult<Vec<Option<Property>>>;

    /// Sets the property with namespace and name prop.nsn() to have value prop.value
    /// For local calendars, the sync status is not modified.
    /// For remote calendars, the sync status is updated by the server (to Synced)
    async fn set_property(&mut self, prop: Property) -> KFResult<SyncStatus>;

    /// Returns whether this calDAV calendar supports to-do items
    fn supports_todo(&self) -> bool {
        self.supported_components()
            .contains(crate::calendar::SupportedComponents::TODO)
    }

    /// Returns whether this calDAV calendar supports calendar items
    fn supports_events(&self) -> bool {
        self.supported_components()
            .contains(crate::calendar::SupportedComponents::EVENT)
    }
}

/// Functions availabe for calendars that are backed by a CalDAV server
///
/// Note that some concrete types (e.g. [`crate::calendar::cached_calendar::CachedCalendar`]) can also provide non-async versions of these functions
#[async_trait]
pub trait DavCalendar: BaseCalendar {
    /// Create a new calendar
    fn new(
        name: String,
        resource: Resource,
        supported_components: SupportedComponents,
        color: Option<Color>,
    ) -> Self;

    /// Get the URLs and the version tags of every item in this calendar
    async fn get_item_version_tags(&self) -> KFResult<HashMap<Url, VersionTag>>;

    /// Returns a particular item
    async fn get_item_by_url(&self, url: &Url) -> KFResult<Option<Item>>;

    /// Returns a set of items.
    /// This is usually faster than calling multiple consecutive [`DavCalendar::get_item_by_url`], since it only issues one HTTP request.
    async fn get_items_by_url(&self, urls: &[Url]) -> KFResult<Vec<Option<Item>>>;

    /// Delete an item
    async fn delete_item(&mut self, item_url: &Url) -> KFResult<()>;

    /// Returns all known WebDAV properties of the calendar collection.
    async fn get_properties(&self) -> KFResult<Vec<Property>>;

    /// Returns the WebDAV property defined on the calendar collection.
    async fn get_property(&self, nsn: &NamespacedName) -> KFResult<Option<Property>>;

    /// Delete a property on the server.
    ///
    /// See also [`CompleteCalendar::mark_prop_for_deletion`] and [`CompleteCalendar::immediately_delete_prop`].
    async fn delete_property(&mut self, nsn: &NamespacedName) -> KFResult<()>;

    /// Get the URLs of all current items in this calendar
    async fn get_item_urls(&self) -> KFResult<HashSet<Url>> {
        let items = self.get_item_version_tags().await?;
        Ok(items.keys().cloned().collect())
    }

    // Note: the CalDAV protocol could also enable to do this:
    // fn get_current_version(&self) -> CTag
}

/// Functions availabe for calendars we have full knowledge of
///
/// Usually, these are local calendars fully backed by a local folder
///
/// Note that some concrete types (e.g. [`crate::calendar::cached_calendar::CachedCalendar`]) can also provide non-async versions of these functions
#[async_trait]
pub trait CompleteCalendar: BaseCalendar {
    /// Create a new calendar
    fn new(
        name: String,
        url: Url,
        supported_components: SupportedComponents,
        color: Option<Color>,
    ) -> Self;

    /// Get the URLs of all current items in this calendar
    async fn get_item_urls(&self) -> KFResult<HashSet<Url>>;

    /// Returns all items that this calendar contains
    async fn get_items(&self) -> KFResult<HashMap<Url, &Item>>;

    /// Returns all items that this calendar contains
    async fn get_items_mut(&mut self) -> KFResult<HashMap<Url, &mut Item>>;

    /// Returns a particular item
    async fn get_item_by_url<'a>(&'a self, url: &Url) -> Option<&'a Item>;

    /// Returns a particular item
    async fn get_item_by_url_mut<'a>(&'a mut self, url: &Url) -> Option<&'a mut Item>;

    /// Returns all known WebDAV properties of the calendar collection.
    async fn get_properties(&self) -> &HashMap<NamespacedName, Property>;

    async fn get_property_by_name(&self, name: &NamespacedName) -> Option<&Property>;

    async fn get_property_by_name_mut(&mut self, name: &NamespacedName) -> Option<&mut Property>;

    /// Adds a property; error if it already exists
    async fn add_property(&mut self, prop: Property) -> KFResult<()>;

    /// Updates a property; error if it does not exist
    async fn update_property(&mut self, prop: Property) -> KFResult<()>;

    /// Mark this calendar for deletion.
    /// This is required so that the upcoming sync will know it should also delete this calendar from the server
    /// (after which this object should be removed from its container)
    async fn mark_for_deletion(&mut self);

    /// Whether this calendar is flagged to be deleted on the next sync
    async fn marked_for_deletion(&self) -> bool;

    /// Mark an item for deletion.
    /// This is required so that the upcoming sync will know it should also also delete this task from the server
    /// (and then call [`CompleteCalendar::immediately_delete_item`] once it has been successfully deleted on the server)
    async fn mark_item_for_deletion(&mut self, item_id: &Url) -> KFResult<()>;

    /// Immediately remove an item. See [`CompleteCalendar::mark_item_for_deletion`]
    async fn immediately_delete_item(&mut self, item_id: &Url) -> KFResult<()>;

    /// Mark a prop for deletion.
    /// This is required so that the upcoming sync will know it should also also delete this prop from the server
    /// (and then call [`CompleteCalendar::immediately_delete_prop`] once it has been successfully deleted on the server)
    async fn mark_prop_for_deletion(&mut self, nsn: &NamespacedName) -> KFResult<()>;

    /// Immediately remove a prop. See [`CompleteCalendar::mark_prop_for_deletion`]
    async fn immediately_delete_prop(&mut self, nsn: &NamespacedName) -> KFResult<()>;
}
