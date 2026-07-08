# Integración OpenTelemetry

vcf-cribador usa `tracing` para logging estructurado. OpenTelemetry se integra como capa adicional de exportación de spans a colectores OTLP (SigNoz, Jaeger, Grafana Tempo, etc.).

## Activación

Añadir al `Cargo.toml`:

```toml
[features]
otel = [
    "dep:opentelemetry",
    "dep:opentelemetry_sdk",
    "dep:opentelemetry-otlp",
    "dep:tracing-opentelemetry",
    "dep:tokio",
]

[dependencies]
opentelemetry = { version = "0.24", features = ["trace"], optional = true }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"], optional = true }
opentelemetry-otlp = { version = "0.17", features = ["tonic", "tls"], optional = true }
tracing-opentelemetry = { version = "0.25", optional = true }
tokio = { version = "1", features = ["rt", "macros"], optional = true }
```

En `src/main.rs`, antes de `main()`:

```rust
#[cfg(feature = "otel")]
static mut OTEL_GUARD: Option<opentelemetry_sdk::trace::TracerProvider> = None;

#[cfg(feature = "otel")]
fn init_otel_subscriber() -> anyhow::Result<()> {
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::layer::SubscriberExt;

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".into());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint);

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default().with_resource(
                opentelemetry_sdk::Resource::new(vec![
                    KeyValue::new("service.name", "vcf-cribador"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]),
            ),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let tracer = opentelemetry::global::tracer_provider().tracer("vcf-cribador");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer().with_env_filter(
            EnvFilter::from_default_env().add_directive("vcf_cribador=info".parse()?),
        ))
        .init();

    unsafe { OTEL_GUARD = Some(provider) };
    Ok(())
}
```

## Compilación

```bash
cargo build --release --features otel
```

## Uso con SigNoz

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
vcf-cribador cribar contactos.vcf -o limpio.vcf
```

Cada ejecución del pipeline generará spans jerárquicos:
```
main
├── parse_vcards
├── normalize_contacts
├── classify_contacts
├── screen_contacts
├── deduplicate
└── write_outputs
```

## Sin endpoint

Si la variable `OTEL_EXPORTER_OTLP_ENDPOINT` no está configurada, el subscriber
funciona en modo normal (solo logging por consola). Zero overhead cuando no se activa.
