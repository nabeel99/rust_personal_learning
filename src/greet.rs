use actix_web::{HttpRequest,Responder};
pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    //this works
    //"hello"
    format!("Hello, {}", name)
}