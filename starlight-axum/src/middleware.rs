use axum::extract::Request;
use axum::http::{HeaderName, StatusCode};
use axum::middleware;
use axum::middleware::{FromFnLayer, Next};
use axum::response::{IntoResponse, Response};
use tower::{Layer, MakeService, ServiceBuilder};
use tower::layer::util::{Identity, Stack};
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::request_id::{
    MakeRequestUuid, PropagateRequestId, PropagateRequestIdLayer, SetRequestId, SetRequestIdLayer,
};
use tower_http::{ServiceBuilderExt};
use tower_otel_http_metrics::HTTPMetricsLayer;
use tracing::Span;
use crate::logger::print_request_response;
use crate::meter::{get_meter_provider, GLOBAL_METER};
use crate::tracer::ForwardSpan;

pub fn generate_request_id_middleware() -> ServiceBuilder<
    Stack<PropagateRequestIdLayer, Stack<SetRequestIdLayer<MakeRequestUuid>, Identity>>,
> {
    let x_request_id = HeaderName::from_static(starlight_protocol::constants::STARLIGHT_REQUEST_ID);

    ServiceBuilder::new()
        .set_request_id(x_request_id.clone(), MakeRequestUuid)
        .propagate_request_id(x_request_id)
}

pub fn trim_slash_path() -> ServiceBuilder<Stack<NormalizePathLayer, Identity>> {
    ServiceBuilder::new().trim_trailing_slash()
}

pub fn common_middleware() -> SetRequestId<
    PropagateRequestId<ServiceBuilder<Stack<NormalizePathLayer, Identity>>>,
    MakeRequestUuid,
> {
    generate_request_id_middleware().service(trim_slash_path())
}

pub fn oltp_middleware() -> HTTPMetricsLayer {
    tower_otel_http_metrics::HTTPMetricsLayerBuilder::builder()
            .with_meter(GLOBAL_METER.clone())
            .build()
            .expect( "Failed to build HTTP metrics layer")
}

// pub fn trace_middleware() -> Box<dyn Layer<HttpMakeClassifier, Service=()>> {
//     TraceLayer::new_for_http()
//         .make_span_with(ForwardSpan)
//         .on_request(|request: &Request<_>, span: &Span| {
//             let headers = format!("{:?}", request.headers());
//             span.record("http.headers", &tracing::field::display(headers));
//             // span.enter();
//         })
//         .on_response(|response: &Response, latency: Duration, span: &Span| {
//             span.record(
//                 "http.status_code",
//                 &tracing::field::display(response.status()),
//             );
//             // span.enter();
//         }).
// }
