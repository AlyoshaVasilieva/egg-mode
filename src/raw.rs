// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Raw access to request- and response-building primitives used internally by egg-mode.
//!
//! The functions and types exposed in this module allow you to access Twitter API functions that
//! aren't currently wrapped by egg-mode, or to provide parameters to Twitter that egg-mode doesn't
//! currently use. These functions also allow you to have more power in how you process the data
//! returned by Twitter. In return, much more knowledge of the Twitter API is required to
//! effectively use these functions.
//!
//! The functions in this module can be divided into two categories: assembling a request, and
//! executing it to get a response. Some wrapper types in egg-mode operate directly on requests, or
//! create their own, so constructors for those types are also exposed here.
//!
//! To start using the functions in this module, you'll need a [`Token`] from the authentication
//! flow. The parameters to an endpoint are represented by the [`ParamList`] type. They're
//! separated out so that they can be included as part of the OAuth signature given to Twitter as
//! part of the API call. This also means that the URL you give to the request functions should be
//! the base URL, with no parameters.
//!
//! [`Token`]: ../enum.Token.html
//! [`ParamList`]: struct.ParamList.html
//!
//! There are three basic request functions, based on how the endpoint expects to be called:
//!
//! * `request_get` assembles a GET request, with the given parameters appended to the URL as a
//!   query string. All GET endpoints that egg-mode currently wraps use this function to encode and
//!   sign the request.
//! * `request_post` assembles a POST request, with the given parameters included in the POST body
//!   formatted as `x-www-form-urlencoded` data. Most POST endpoints in the Twitter API are
//!   formatted using this function.
//! * `request_post_json` also assembles a POST request, but instead of taking a `ParamList`, it
//!   takes arbitrary data and formats it in the POST body as JSON. The provided data is *not* used
//!   as part of the OAuth signature. At time of writing (between releases 0.14 and 0.15) the only
//!   egg-mode endpoint that uses this function is [`media::set_metadata`].
//!
//! [`media::set_metadata`]: ../media/fn.set_metadata.html
//!
//! Once you have a `Request`, you can hand it to the `response_*` functions in this module to
//! process it. Which one you select depends on how much processing you want egg-mode to do with
//! the response:
//!
//! * At the most hands-off end, there's [`response_future`,] which is a small wrapper that just
//!   starts the request and hands off the `ResponseFuture` from `hyper` to give you the most power
//!   over handling the response data.
//! * In the middle, there's [`response_raw_bytes`], which wraps the `ResponseFuture` to return the
//!   headers and response body after inspecting the rate-limit headers and response code, and
//!   after inspecting the response to see whether it returned error data from Twitter.
//! * Finally there's [`response_json`], which picks up from `response_raw_bytes` to parse the
//!   response as JSON and deserialize it into the target type, alongside the rate-limit
//!   information from the response headers.
//!
//! [`response_future`]: fn.response_future.html
//! [`response_raw_bytes`]: fn.response_raw_bytes.html
//! [`response_json`]: fn.response_json.html
//!
//! In addition, there are `request_as_*` and `response_as_*` functions available to format a
//! request using one of the wrappers used in egg-mode. If the endpoint you're using is one that
//! currently uses one of these wrapper types or returns and accepts data the same way as one of
//! these endpoints, you can use these functions to get the same experience as the existing
//! wrappers in egg-mode. See the documentation for these functions to see their assumptions and
//! requirements.

use hyper::{Body, Request};

use crate::auth::Token;
use crate::cursor;
use crate::stream::TwitterStream;

use crate::tweet::Timeline as TweetTimeline;
use crate::direct::Timeline as DMTimeline;

pub use crate::common::ParamList;
pub use crate::common::Headers;

pub use crate::auth::get as request_get;
pub use crate::auth::post as request_post;
pub use crate::auth::post_json as request_post_json;

/// Assemble a GET request and convert it to a `Timeline` of tweets.
pub fn request_as_tweet_timeline(
    url: &'static str,
    token: &Token,
    params: Option<ParamList>
) -> TweetTimeline {
    TweetTimeline::new(url, params, token)
}

/// Assemble a GET request and convert it to a `Timeline` of direct messages.
pub fn request_as_dm_timeline(
    url: &'static str,
    token: &Token,
    params: Option<ParamList>
) -> DMTimeline {
    DMTimeline::new(url, params, token)
}

/// Assemble a GET request and convert it to a `CursorIter`.
pub fn request_as_cursor_iter<T: cursor::Cursor + serde::de::DeserializeOwned>(
    url: &'static str,
    token: &Token,
    params: Option<ParamList>,
    page_size: Option<i32>
) -> cursor::CursorIter<T> {
    cursor::CursorIter::new(url, token, params, page_size)
}

pub use crate::common::get_response as response_future;
pub use crate::common::raw_request as response_raw_bytes;
pub use crate::common::request_with_json_response as response_json;

/// Converts the given request into a `TwitterStream`.
///
/// This function can be used for endpoints that open a persistent stream, like `GET
/// statuses/sample`. If you know that the messages returned by the stream you're using will look
/// the same as `StreamMessage`, this can be a convenient way to customize a stream if you need to
/// use other endpoints or options not available to `StreamBuilder`.
pub fn response_as_stream(req: Request<Body>) -> TwitterStream {
    TwitterStream::new(req)
}
