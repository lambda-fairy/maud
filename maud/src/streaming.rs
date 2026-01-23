//! # Streaming generation
//!
//! Generation of streaming responses

pub use async_stream;

use futures::{
    future::{Ready, ready},
    stream::{Once, once},
};
pub use maud_macros::streaming_html;

use crate::{Markup, Render};

/// A streaming document.
///
/// The macro [`streaming_html`] extends to [`StreamingMarkup<impl Stream<Item = Markup> + Send + 'static>`]
#[derive(Debug, Clone, Copy)]
pub struct StreamingMarkup<S>(pub S);

impl<T> From<T> for StreamingMarkup<Once<Ready<Markup>>>
where
    T: Render,
{
    fn from(value: T) -> Self {
        Self(once(ready(value.render())))
    }
}

#[cfg(feature = "axum")]
mod axum_support {
    use core::convert::Infallible;

    use crate::{PreEscaped, streaming::StreamingMarkup};
    use alloc::string::String;
    use axum_core::{
        body::Body,
        response::{IntoResponse, Response},
    };
    use futures::{Stream, StreamExt};
    use http::{HeaderValue, header};

    impl<S> IntoResponse for StreamingMarkup<S>
    where
        S: Stream<Item = PreEscaped<String>> + Send + 'static,
    {
        fn into_response(self) -> Response {
            let headers = [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            )];

            let body = Body::from_stream(self.0.map(|PreEscaped(s)| Ok::<_, Infallible>(s)));

            (headers, body).into_response()
        }
    }
}
