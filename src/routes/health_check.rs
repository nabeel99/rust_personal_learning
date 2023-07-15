use actix_web::{Responder,HttpResponse,HttpRequest};
pub async fn check_health(_: HttpRequest) -> impl Responder {
    dbg!("Here in health_check");
    HttpResponse::Ok()
}
