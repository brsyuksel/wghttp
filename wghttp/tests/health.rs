use actix_web::{test, App};
use wghttp::routes::health::health;

#[actix_web::test]
async fn test_health_route_returns_ok() {
    let app = test::init_service(App::new().service(health)).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(resp.status(), 200);
}
