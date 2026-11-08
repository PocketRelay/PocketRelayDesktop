use axum::{Router, routing::get};

pub mod gaw;
pub mod public;
pub mod qos;

/// Function for configuring the provided service config with all the
/// application routes.
pub fn router() -> Router {
    Router::new()
        // Galaxy at war
        .route(
            "/authentication/sharedTokenLogin",
            get(gaw::shared_token_login),
        )
        .nest(
            "/galaxyatwar",
            Router::new()
                .route("/getRatings/{id}", get(gaw::get_ratings))
                .route("/increaseRatings/{id}", get(gaw::increase_ratings)),
        )
        // Quality of service
        .nest(
            "/qos",
            Router::new()
                .route("/qos", get(qos::qos))
                .route("/firewall", get(qos::firewall))
                .route("/firetype", get(qos::firetype)),
        )
        // Public content fallback
        .fallback_service(public::PublicContent)
}
