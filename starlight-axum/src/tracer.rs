use crate::resource::get_resource;
use axum::http;
use opentelemetry::global;
use opentelemetry::trace::TraceContextExt;
use opentelemetry_http::HeaderExtractor;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use std::sync::OnceLock;
use tower_http::trace::MakeSpan;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

static SDK_TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
pub fn get_tracer_provider() -> &'static SdkTracerProvider {
    SDK_TRACER_PROVIDER
        .get()
        .expect("Failed to get tracer provider")
}

pub fn get_or_init_tracer_provider(oltp_grpc_url: &str) -> SdkTracerProvider {
    SDK_TRACER_PROVIDER
        .get_or_init(|| {
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(oltp_grpc_url)
                .build()
                .expect("Failed to create exporter");

            SdkTracerProvider::builder()
                .with_resource(get_resource())
                .with_id_generator(RandomIdGenerator::default())
                .with_sampler(Sampler::AlwaysOn)
                .with_batch_exporter(exporter)
                .build()
        })
        .clone()
}

#[derive(Clone)]
pub struct ForwardSpan;

impl<B> MakeSpan<B> for ForwardSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> Span {
        // Check if there's a parent context from OpenTelemetry
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });

        let span = info_span!("http_request", method = %request.method(), uri = %request.uri(), version = ?request.version(), headers = ?request.headers());

        if parent_context.span().span_context().is_valid() {
            // Attach the parent span to the new span
            span.set_parent(parent_context);
        }

        span
    }
}
