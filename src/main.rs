#![warn(rust_2018_idioms)]
use zero2prod::configuration::{get_configuration, self};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::{run};
use zero2prod::telemetry::{get_subscriber, init_global_logger};

// use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
// use sqlx::PgPool;
use std::net::TcpListener;

// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //set_boxed_logger convenience wrapper, calls set_logger with a function/closure that calls box::leak on
    //on the boxed dyn log
    //used to configure the log crate logger, facade pattern
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    // Redirect all `log`'s events to our subscriber
//telemetry initialization
    let subscriber = get_subscriber("info", "rust_personal_learning", std::io::stdout);
    init_global_logger(subscriber);

    //settings
    let settings = get_configuration().expect("Failed to read configuration");
   let ret =  zero2prod::startup::Application::build(settings)?;

   ret.run_until_stopped().await?;
   Ok(())
}
