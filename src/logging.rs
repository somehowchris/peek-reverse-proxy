use tracing_log::LogTracer;
use tracing_subscriber::field::MakeExt;

use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

#[derive(PartialEq, Clone)]
pub enum PrintStyle {
    Pretty,
    Plain,
    Json,
}

impl From<&str> for PrintStyle {
    fn from(input: &str) -> Self {
        match input {
            "pretty" => PrintStyle::Pretty,
            "plain" => PrintStyle::Plain,
            "json" => PrintStyle::Json,
            _ => panic!("Unknown print style"),
        }
    }
}

impl PrintStyle {
    fn to_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Plain => "plain",
            Self::Pretty => "pretty",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LogLevel {
    /// Only shows errors and warnings: `"critical"`.
    Critical,
    /// Shows everything except debug and trace information: `"normal"`.
    Normal,
    /// Shows everything: `"debug"`.
    Debug,
    /// Shows nothing: "`"off"`".
    Off,
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        return match &*s.to_ascii_lowercase() {
            "critical" => LogLevel::Critical,
            "normal" => LogLevel::Normal,
            "debug" => LogLevel::Debug,
            "off" => LogLevel::Off,
            _ => panic!("a log level (off, debug, normal, critical)"),
        };
    }
}

/// Default layer formatting output
pub fn default_logging_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber,
    S: for<'span> LookupSpan<'span>,
{
    let field_format = tracing_subscriber::fmt::format::debug_fn(|writer, field, value| {
        // We'll format the field name and value separated with a colon.
        if field.name() == "message" {
            write!(writer, "{:?}", value)
        } else {
            write!(writer, "{}: {:?}", field, value)
        }
    })
    .delimited(", ")
    .display_messages();

    tracing_subscriber::fmt::layer()
        .fmt_fields(field_format)
        .with_test_writer()
}

/// Filter installed to filter messages down to a given log level
pub fn filter_layer(level: LogLevel) -> EnvFilter {
    let filter_str = match level {
        LogLevel::Critical => "warn,hyper=off,rustls=off",
        LogLevel::Normal => "info,hyper=off,rustls=off",
        LogLevel::Debug => "trace",
        LogLevel::Off => "off",
    };

    tracing_subscriber::filter::EnvFilter::try_new(filter_str).expect("filter string must parse")
}

/// Logging layer to format the log output as json
pub fn json_logging_layer<
    S: for<'a> tracing_subscriber::registry::LookupSpan<'a> + tracing::Subscriber,
>() -> impl tracing_subscriber::Layer<S> {
    tracing_subscriber::fmt::layer().json().with_test_writer()
}

/// Setup logging for this application
///
/// Uses tracing logs
pub fn setup_logging() -> (PrintStyle, LogLevel) {
    use tracing_subscriber::prelude::*;

    LogTracer::init().expect("Unable to setup log tracer!");

    let style: PrintStyle = PrintStyle::from(
        std::env::var("PRINT_STYLE")
            .unwrap_or_else(|_| PrintStyle::Pretty.to_str().to_string())
            .as_str(),
    );

    let log_level = LogLevel::from(
        std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "normal".to_string())
            .as_str(),
    );

    // Setup logger
    if style == PrintStyle::Json {
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry()
                .with(json_logging_layer())
                .with(filter_layer(log_level)),
        )
        .unwrap();
    } else {
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry()
                .with(default_logging_layer())
                .with(filter_layer(log_level)),
        )
        .unwrap();
    }

    info!("Using print style: {}", style.to_str());

    (style, log_level)
}
