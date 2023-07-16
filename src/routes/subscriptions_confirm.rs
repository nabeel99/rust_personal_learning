use actix_web::HttpResponse;
use actix_web::web;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token : String,
}

#[tracing::instrument(name = "Conifrm a pending subscriber",skip(parameters))]
//type safe api structured destructing from wrapper struct
pub async fn confirm(web::Query(parameters) : web::Query<Parameters>,
pool : web::Data<PgPool>,
) -> HttpResponse {
    let id = match get_subscriber_id_from_token(&pool,parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    match id {
        //Non existing token
        None => return HttpResponse::Unauthorized().finish(),
        Some(id) => if confirm_subscriber(&pool,&id).await.is_err() {
            return HttpResponse::InternalServerError().finish();
        },
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(name = "Get subscriber id from a token",skip(pool,token))]
pub async fn get_subscriber_id_from_token(pool : &PgPool,token : String) -> Result<Option<Uuid>,sqlx::Error> {
    let result = sqlx::query!(r#"SELECT subscriber_id from subscription_tokens where subscription_token = $1"#,token).fetch_optional(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e })?;
        //maps from option<record> to option<uuid> this is what map does for option type
    Ok(result.map(|r|r.subscriber_id) )
}
#[tracing::instrument(name = "Mark a subscriber as confirmed",skip(pool,subscriber_id))]
pub async fn confirm_subscriber(pool : &PgPool,subscriber_id : &Uuid) -> Result<(),sqlx::Error> {
    sqlx::query!(r#"UPDATE subscriptions SET status = 'confirmed' where id = $1"#,subscriber_id).execute(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(())

}
