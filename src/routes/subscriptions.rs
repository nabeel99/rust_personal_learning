use std::fmt::Formatter;

use actix_web::{web, HttpResponse, ResponseError};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::StatusCode;
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::Transaction;
use uuid::Uuid;
//extension traits
use anyhow::Context;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};
#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}
//Using 25 characters we get roughly ~10^45 possible tokens -
fn generate_subscription_token() -> String {
    // lazily seeded rng generator
    let mut rng = thread_rng();
    //iterate a randomly generated CSPRG and take the first 25
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
#[tracing::instrument(name="Adding a Subscriber",
skip(form,_pool_connection,email_client,base_url),
fields(
    //in order to use the request_id passed from request id
        // request_id=%Uuid::new_v4(),
        subscriber_email=%form.email,
        subscriber_name=%form.name
))]
pub async fn subscribe(
    form: web::Form<FormData>,
    //retrieves a connection from the application state
    //looks for the closest resource with the given type
    //PgPool is a type alias for Pool<POSTGRES>
    _pool_connection: web::Data<PgPool>,
    //get email client from app context
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    dbg!("Here in sub");
    // let request_id = Uuid::new_v4();
    // let request_span = tracing::info_span!(
    //     "Adding A new subscriber",
    //     %request_id,
    //     subscriber_email=%_form.email,
    //     subscriber_name=%_form.name
    // );
    // let _request_span_guard = request_span.enter();
    //query logic
    let mut transaction =  _pool_connection.begin().await.context("Failed to acquire a Postgres connection from the pool")?;

    let new_subscriber =
         NewSubscriber::try_from(form.0).map_err(SubscribeError::ValidationError)?;
    let sub_id = insert_subscriber(&new_subscriber, &mut transaction).await.context("Failed to insert new subscriber in the database.")?;
    //generate a token
    let subscription_token = generate_subscription_token();
    //store the token against subscriber id
    store_token(&mut transaction, sub_id, &subscription_token)
        .await.context("Failed to store the confirmation token for a new subscriber.")?;
   send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,).await.context("Failed to send a confirmation email.")?;
    
     transaction.commit().await.context("Failed to commit SQL transaction to store a new subscriber.",)?;
    Ok(HttpResponse::Ok().finish())
}
#[tracing::instrument(name = "Storing a newly generated token", skip(pool, sub_id, token))]
pub async fn store_token(
    pool: &mut Transaction<'_, Postgres>,
    sub_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens(
        subscription_token,subscriber_id) VALUES($1,$2)"#,
        token,
        sub_id
    )
    .execute(pool)
    .await
    //remove error log spam
    ?;
    Ok(())
}
#[tracing::instrument(
    name = "Sends a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &str,
) -> Result<(),reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, token
    );

    email_client
        .send_email(
            new_subscriber.email,
            "WELCOME",
            &format!(
                "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
                confirmation_link
            ),
        )
        .await?;
    Ok(())
}
//if the build fails check if db_url env is set or there is an offline db build
#[tracing::instrument(
    name = "Saving Details to the Database",
    skip(new_subscriber, pool_connection)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool_connection: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions(id,email,name,subscribed_at,status)
        VALUES($1,$2,$3,$4,'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool_connection)
    //decorating it by adding a logger in case error is returned
    .await
    //remove error log spam
    ?;

    Ok(subscriber_id)
}

//#[from] #[source] means the same, from implies source
#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error{"0"}]
    ValidationError(String),
    // #[error("Failed to acquire a Postgres connection from the pool")]
    // PoolError(#[source] sqlx::Error),
    // #[error("Failed to insert new subscriber in the database.")]
    // InsertSubscriberError(#[source] sqlx::Error),
    // #[error("Failed to store the confirmation token for a new subscriber.")]
    // StoreTokenError,
    // #[error("Failed to commit SQL transaction to store a new subscriber.")]
    // TransactionCommitError(#[source] sqlx::Error),
    // #[error("Failed to send a confirmation email.")]
    // SendEmailError(#[from] reqwest::Error),
    // Transparent delegates both `Display`'s and `source`'s implementation // to the type wrapped by `UnexpectedError`.
    // #[error("{1}")]
    // Unexpectederror(#[source] Box<dyn std::error::Error>,String),
    #[error(transparent)]
    Unexpectederror(#[from] anyhow::Error)
}
impl ResponseError for SubscribeError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::Unexpectederror(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
//until backtrace becomes stable use this to print chain of errors
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused By:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
