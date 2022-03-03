mod controller;
mod model;
use std::{
	collections::HashSet,
	net::SocketAddr,
	sync::{Arc, Mutex},
};

use axum::{routing::get, AddExtensionLayer, Router};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
	let user_set = Mutex::new(HashSet::new());
	let (tx, _rx) = broadcast::channel(100);
	let app_state = Arc::new(model::AppState { user_set, tx });

	let app = Router::new()
		.route("/websocket", get(controller::chat::websocket_handler))
		.layer(AddExtensionLayer::new(app_state));

	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

	axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
