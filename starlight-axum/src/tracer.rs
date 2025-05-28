use crate::resource::get_resource;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use std::sync::OnceLock;

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
