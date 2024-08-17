use reqwest::StatusCode;
use url::Url;

use crate::{calendar::remote_calendar::RemoteCalendarError, item::ItemType};

#[derive(Debug)]
pub enum HttpStatusConstraint {
    Success,
    Specific(StatusCode),
}

/// Errors common to the Kitchen Fridge library
#[derive(thiserror::Error, Debug)]
pub enum KFError {
    #[error(
        "Calendar at URL {0} didn't appear in the client cache after being created on the server"
    )]
    CalendarDidNotSyncAfterCreation(Url),

    #[error("Error parsing '{text}': {source}")]
    DOMParseError {
        /// The text being parsed
        text: String,

        source: minidom::Error,
    },

    #[error("Generic error: {0}")]
    GenericError(#[from] Box<dyn std::error::Error>),

    #[error("HTTP request {method} {url} resulted in an error: {source}")]
    HttpRequestError {
        url: Url,
        method: http::Method,
        source: reqwest::Error,
    },

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

    #[error("Missing DOM element {0}")]
    MissingExpectedDOMElement(String),

    #[error("An error occurred while mocking behavior: {0}")]
    #[cfg(feature = "local_calendar_mocks_remote_calendars")]
    MockError(#[from] crate::mock_behaviour::MockError),

    #[error("Remote calendar error: {0}")]
    RemoteCalendarError(#[from] RemoteCalendarError),

    #[error("Unexpected HTTP status code {got:?} but expected {expected:?}")]
    UnexpectedHTTPStatusCode {
        expected: HttpStatusConstraint,
        got: StatusCode,
    },
}

pub type KFResult<T> = Result<T, KFError>;
