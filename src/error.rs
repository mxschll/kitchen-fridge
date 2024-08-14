use reqwest::StatusCode;
use url::Url;

use crate::item::ItemType;

#[derive(Debug)]
pub enum HttpStatusConstraint {
    Success,
    Specific(StatusCode),
}

/// Errors common to the Kitchen Fridge library
#[derive(thiserror::Error, Debug)]
pub enum KFError {
    #[error("{detail}; {type_:?} {url:?} already exists")]
    ItemAlreadyExists {
        type_: ItemType,
        detail: String,
        url: Url,
    },

    #[error("{detail}; {type_:?} {url:?} does not exist")]
    ItemDoesNotExist {
        type_: Option<ItemType>,
        detail: String,
        url: Url,
    },

    #[error("Unexpected HTTP status code {got:?} but expected {expected:?}")]
    UnexpectedHTTPStatusCode {
        expected: HttpStatusConstraint,
        got: StatusCode,
    },
}
