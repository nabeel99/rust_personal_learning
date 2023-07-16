use actix_web::{web, HttpResponse};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use uuid::Uuid;
use sqlx::Transaction;
use sqlx::Postgres;

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
) -> HttpResponse {
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
    let mut transaction = match _pool_connection.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let new_subscriber = match NewSubscriber::try_from(form.0) {
        Ok(new_sub) => new_sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let sub_id = match insert_subscriber(&new_subscriber, &mut transaction).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    //generate a token
    let subscription_token = generate_subscription_token();
    //store the token against subscriber id
    if store_token(&mut transaction, sub_id, &subscription_token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }
    if send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    {
        return HttpResponse::InternalServerError().finish();
    }
    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();

    }
    HttpResponse::Ok().finish()
}
#[tracing::instrument(name = "Storing a newly generated token", skip(pool, sub_id, token))]
pub async fn store_token(pool: &mut Transaction<'_, Postgres>, sub_id: Uuid, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens(
        subscription_token,subscriber_id) VALUES($1,$2)"#,
        token,
        sub_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
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
) -> bool {
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
        .await
        .is_err()
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
    .map_err(|e| {
        tracing::error!("Failed to execute query , Error : {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}
