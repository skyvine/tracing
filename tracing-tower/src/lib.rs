#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/tokio-rs/tracing/main/assets/logo-type.png",
    issue_tracker_base_url = "https://github.com/tokio-rs/tracing/issues/"
)]
#![warn(
    missing_debug_implementations,
    // missing_docs, // TODO: add documentation!
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_interfaces,
    private_bounds,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::fmt;
use tower_service::Service;
use tracing::Level;

pub mod request_span;
pub mod service_span;

#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub mod http;

pub type InstrumentedService<S, R> = service_span::Service<request_span::Service<S, R>>;

pub trait InstrumentableService<Request>
where
    Self: Service<Request> + Sized,
{
    fn instrument<G>(self, svc_span: G) -> InstrumentedService<Self, Request>
    where
        G: GetSpan<Self>,
        Request: fmt::Debug,
    {
        let req_span: fn(&Request) -> tracing::Span =
            |request| tracing::span!(Level::TRACE, "request", ?request);
        let svc_span = svc_span.span_for(&self);
        self.trace_requests(req_span).trace_service(svc_span)
    }

    fn trace_requests<G>(self, get_span: G) -> request_span::Service<Self, Request, G>
    where
        G: GetSpan<Request> + Clone,
    {
        request_span::Service::new(self, get_span)
    }

    fn trace_service<G>(self, get_span: G) -> service_span::Service<Self>
    where
        G: GetSpan<Self>,
    {
        let span = get_span.span_for(&self);
        service_span::Service::new(self, span)
    }
}

impl<S, R> InstrumentableService<R> for S where S: Service<R> + Sized {}

pub trait GetSpan<T>: crate::sealed::Sealed<T> {
    fn span_for(&self, target: &T) -> tracing::Span;
}

impl<T, F> crate::sealed::Sealed<T> for F where F: Fn(&T) -> tracing::Span {}

impl<T, F> GetSpan<T> for F
where
    F: Fn(&T) -> tracing::Span,
{
    #[inline]
    fn span_for(&self, target: &T) -> tracing::Span {
        (self)(target)
    }
}

impl<T> crate::sealed::Sealed<T> for tracing::Span {}

impl<T> GetSpan<T> for tracing::Span {
    #[inline]
    fn span_for(&self, _: &T) -> tracing::Span {
        self.clone()
    }
}

mod sealed {
    pub trait Sealed<T = ()> {}
}
