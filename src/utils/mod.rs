//! Some utility functions

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self};
use std::hash::Hash;
use std::io::{stdin, stdout, Read, Write};
use std::sync::{Arc, Mutex};

use minidom::Element;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::item::SyncStatus;
use crate::traits::CompleteCalendar;
use crate::traits::DavCalendar;
use crate::Item;

/// Walks an XML tree and returns every element that has the given name
pub fn find_elems<S: AsRef<str>>(root: &Element, searched_name: S) -> Vec<&Element> {
    let searched_name = searched_name.as_ref();
    let mut elems: Vec<&Element> = Vec::new();

    for el in root.children() {
        if el.name() == searched_name {
            elems.push(el);
        } else {
            let ret = find_elems(el, searched_name);
            elems.extend(ret);
        }
    }
    elems
}

/// Walks an XML tree until it finds an elements with the given name
pub fn find_elem<S: AsRef<str>>(root: &Element, searched_name: S) -> Option<&Element> {
    let searched_name = searched_name.as_ref();
    if root.name() == searched_name {
        return Some(root);
    }

    for el in root.children() {
        if el.name() == searched_name {
            return Some(el);
        } else {
            let ret = find_elem(el, searched_name);
            if ret.is_some() {
                return ret;
            }
        }
    }
    None
}

pub fn print_xml(element: &Element) {
    let mut writer = std::io::stdout();

    let mut xml_writer = minidom::quick_xml::Writer::new_with_indent(std::io::stdout(), 0x20, 4);
    let _ = element.to_writer(&mut xml_writer);
    let _ = writer.write(&[0x0a]);
}

/// A debug utility that pretty-prints calendars
pub async fn print_calendar_list<C>(cals: &HashMap<Url, Arc<Mutex<C>>>)
where
    C: CompleteCalendar,
{
    for (url, cal) in cals {
        println!("CAL {} ({})", cal.lock().unwrap().name(), url);
        match cal.lock().unwrap().get_items().await {
            Err(_err) => continue,
            Ok(map) => {
                for (_, item) in map {
                    print_task(item);
                }
            }
        }
    }
}

/// A debug utility that pretty-prints calendars
pub async fn print_dav_calendar_list<C>(cals: &HashMap<Url, Arc<Mutex<C>>>)
where
    C: DavCalendar,
{
    for (url, cal) in cals {
        println!("CAL {} ({})", cal.lock().unwrap().name(), url);
        match cal.lock().unwrap().get_item_version_tags().await {
            Err(_err) => continue,
            Ok(map) => {
                for (url, version_tag) in map {
                    println!("    * {} (version {:?})", url, version_tag);
                }
            }
        }
    }
}

pub fn print_task(item: &Item) {
    if let Item::Task(task) = item {
        let completion = if task.completed() { "✓" } else { " " };
        let sync = match task.sync_status() {
            SyncStatus::NotSynced => ".",
            SyncStatus::Synced(_) => "=",
            SyncStatus::LocallyModified(_) => "~",
            SyncStatus::LocallyDeleted(_) => "x",
        };
        println!("    {}{} {}\t{}", completion, sync, task.name(), task.url());
    }
}

/// Compare keys of two hashmaps for equality
pub fn keys_are_the_same<T, U, V>(left: &HashMap<T, U>, right: &HashMap<T, V>) -> bool
where
    T: Hash + Eq + Clone + std::fmt::Display,
{
    if left.len() != right.len() {
        log::debug!("Count of keys mismatch: {} and {}", left.len(), right.len());
        return false;
    }

    let keys_l: HashSet<T> = left.keys().cloned().collect();
    let keys_r: HashSet<T> = right.keys().cloned().collect();
    let result = keys_l == keys_r;
    if !result {
        log::debug!("Keys of a map mismatch");
        for key in keys_l {
            log::debug!("   left: {}", key);
        }
        log::debug!("RIGHT:");
        for key in keys_r {
            log::debug!("  right: {}", key);
        }
    }
    result
}

/// Wait for the user to press enter
pub fn pause() {
    let mut stdout = stdout();
    stdout.write_all(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read_exact(&mut [0]).unwrap();
}

/// Generate a random URL with a given prefix
pub fn random_url(parent_calendar: &Url) -> Url {
    let random = uuid::Uuid::new_v4().to_hyphenated().to_string();
    parent_calendar.join(&random).unwrap(/* this cannot panic since we've just created a string that is a valid URL */)
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub(crate) struct NamespacedName {
    pub xmlns: String,
    pub name: String,
}
impl NamespacedName {
    pub(crate) fn new<S1: ToString, S2: ToString>(xmlns: S1, name: S2) -> Self {
        Self {
            xmlns: xmlns.to_string(),
            name: name.to_string(),
        }
    }

    /// Uses namespace mappings to simplify the representation of this name
    /// For example, https://example.com/api/item becomes b:item if namespace https://example.com/api/ has symbol b in the namespace mapping
    pub(crate) fn with_symbolized_prefix(&self, namespaces: &Namespaces) -> String {
        let sym = namespaces.sym(&self.xmlns).unwrap();
        format!("{}:{}", sym, self.name)
    }
}
impl fmt::Display for NamespacedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.xmlns.as_str())?;
        fmt::Write::write_char(f, ':')?;
        f.write_str(self.name.as_str())
    }
}

/// Utility to track XML namespace symbol mappings, as used in xmlns attribute declarations
///
/// Includes a default mapping of xmlns:d="DAV:"
pub(crate) struct Namespaces {
    available_syms: VecDeque<char>,
    mapping: HashMap<String, char>,
}

impl Namespaces {
    pub(crate) fn new() -> Self {
        let mut mapping = HashMap::new();
        mapping.insert("DAV:".into(), 'd');

        Self {
            available_syms: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcefghijklmnopqrstuvwxyz" //NOTE the missing 'd'
                .chars()
                .collect(),
            mapping,
        }
    }

    /// Maps the namespace to an unassigned symbol and returns it
    pub(crate) fn add<S: ToString>(&mut self, ns: S) -> char {
        let sym = self
            .available_syms
            .pop_back()
            .expect("Ran out of namespace symbols");

        self.mapping.insert(ns.to_string(), sym);

        sym
    }

    pub(crate) fn decl(&self) -> String {
        let mut s = String::new();
        for (k, v) in &self.mapping {
            s.push(' ');
            s.push_str("xmlns:");
            s.push(*v);
            s.push('=');
            s.push('"');
            s.push_str(k.as_str());
            s.push('"');
        }
        s
    }

    pub(crate) fn prefixes(&self) -> std::collections::hash_map::Values<String, char> {
        self.mapping.values()
    }

    pub(crate) fn sym(&self, ns: &String) -> Option<char> {
        self.mapping.get(ns).cloned()
    }
}

/// A WebDAV property.
///
/// Similar to ical Property but allowing arbitrary namespaces and tracking of sync status
/// This should allow for user-defined properties
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct Property {
    pub nsn: NamespacedName,
    pub value: String,
    pub sync_status: SyncStatus,
}

impl Property {
    pub fn new<S1: ToString, S2: ToString>(xmlns: S1, name: S2, value: String) -> Self {
        Self {
            nsn: NamespacedName {
                xmlns: xmlns.to_string(),
                name: name.to_string(),
            },
            value,
            sync_status: SyncStatus::NotSynced,
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
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.xmlns())?;

        fmt::Write::write_char(f, ':')?;

        f.write_str(self.name())?;

        fmt::Write::write_char(f, '=')?;

        f.write_str(self.value.as_str())
    }
}

impl Into<NamespacedName> for Property {
    fn into(self) -> NamespacedName {
        self.nsn
    }
}
