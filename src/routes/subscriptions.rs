use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}
#[tracing::instrument(name="Adding a Subscriber",
skip(_form,_pool_connection),
fields(
    //in order to use the request_id passed from request id
        // request_id=%Uuid::new_v4(),
        subscriber_email=%_form.email,
        subscriber_name=%_form.name
))]
pub async fn subscribe(
    _form: web::Form<FormData>,
    //retrieves a connection from the application state
    //looks for the closest resource with the given type
    //PgPool is a type alias for Pool<POSTGRES>
    _pool_connection: web::Data<PgPool>,
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
    match insert_subscriber(&_form, &_pool_connection).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
//if the build fails check if db_url env is set or there is an offline db build
#[tracing::instrument(name = "Saving Details to the Database", skip(data, pool_connection))]
pub async fn insert_subscriber(
    data: &FormData,
    pool_connection: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions(id,email,name,subscribed_at)
        VALUES($1,$2,$3,$4)
        "#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now()
    )
    .execute(pool_connection)
    //decorating it by adding a logger in case error is returned
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query , Error : {:?}", e);
        e
    })?;

    Ok(())
}
