use minidom::Element;

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
