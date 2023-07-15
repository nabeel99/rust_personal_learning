use tracing::subscriber::{set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};
pub fn get_subscriber<Sink>(
    env_filter_name: &str,
    name: &str,
    s: Sink,
) -> impl Subscriber + Send + Sync
//MakeWriter is a kind of trait which is implemented for specific lifetimes
//we use the following syntax to denote that our generici impl makewrite for all types of lifetimes
//we also need it to be send and sync + static(valid for the entire lifetime of the program)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    //layered tracing, env_filer->jsonstoragelayer->bunyan formatter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter_name));
    let formatting_layer = BunyanFormattingLayer::new(name.into(), s);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_global_logger(sub: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(sub).expect("failed to set a subscriber");
}
