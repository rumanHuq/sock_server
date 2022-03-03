mod controller;
mod model;
use std::{
	collections::HashSet,
	net::SocketAddr,
	sync::{Arc, Mutex},
};

use axum::{
	http::StatusCode,
	routing::{get, get_service},
	AddExtensionLayer, Router,
};
use controller::home;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
	let user_set = Mutex::new(HashSet::new());
	let (tx, _rx) = broadcast::channel(100);
	let app_state = Arc::new(model::AppState { user_set, tx });

	let app = Router::new()
		.route("/", get(home))
		.route("/websocket", get(controller::chat::websocket_handler))
		.nest(
			"/static",
			get_service(ServeDir::new("./web")).handle_error(|error: std::io::Error| async move {
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Unhandled internal error: {}", error),
				)
			}),
		)
		.layer(AddExtensionLayer::new(app_state));

	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

	axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
