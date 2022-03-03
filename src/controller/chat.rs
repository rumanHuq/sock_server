use std::sync::Arc;

use axum::{
	extract::{
		ws::{Message, WebSocket},
		Extension, WebSocketUpgrade,
	},
	response::IntoResponse,
};
use futures::{SinkExt, StreamExt};

use crate::model::{AppState, Incoming, Outgoing};

fn get_username_if_not_exists(state: &Arc<AppState>, incoming: &Incoming) -> Option<String> {
	let mut user_set = state.user_set.lock().unwrap();
	if incoming.event != "registration" {
		return None;
	}
	if user_set.contains(&incoming.user) {
		return None;
	}
	user_set.insert(incoming.user.to_owned());
	Some(incoming.user.to_owned())
}

pub async fn websocket_handler(ws: WebSocketUpgrade, Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
	ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: Arc<AppState>) {
	let (mut sender, mut receiver) = socket.split();
	let mut username = String::new();

	while let Some(Ok(msg)) = receiver.next().await {
		if let Message::Text(payload) = msg {
			let incoming = serde_json::from_str::<Incoming>(&payload).unwrap();

			if let Some(new_user) = get_username_if_not_exists(&state, &incoming) {
				username = new_user;
			}
			if username.is_empty() {
				let outgoing = Outgoing {
					event: String::from("registration_fail"),
					payload: String::from("username already taken"),
					user: incoming.user,
				};
				let _ = sender
					.send(Message::Text(serde_json::to_string(&outgoing).unwrap()))
					.await;
				return; /* finish handle_websocket function execution */
			} else {
				break; /* let's go with the user after the loop */
			}
		}
	}

	/* after the loop */
	let mut subscription = state.tx.subscribe();
	let outgoing = Outgoing {
		event: String::from("registration_success"),
		payload: format!("{} joined", username),
		user: username,
	};
	let _ = state.tx.send(serde_json::to_string(&outgoing).unwrap());

	/* whenever, a broadcast is recieved, send to socket */
	let mut send_task = tokio::spawn(async move {
		while let Ok(incoming_msg) = subscription.recv().await {
			if sender.send(Message::Text(incoming_msg)).await.is_err() {
				break;
			}
		}
	});

	let tx = state.tx.clone();

	/* consecutive socket messages, when user is registred */
	let mut recv_task = tokio::spawn(async move {
		while let Some(Ok(Message::Text(payload))) = receiver.next().await {
			let incoming = serde_json::from_str::<Incoming>(&payload).unwrap();
			let msg = serde_json::to_string(&incoming).unwrap();
			let _ = tx.send(msg);
		}
	});

	/* when user leaves */
	tokio::select! {
		_=(&mut send_task) => recv_task.abort(),
		_=(&mut recv_task) => send_task.abort(),
	};
	let outgoing = Outgoing {
		event: String::from("user_left"),
		payload: format!(""),
		user: outgoing.user,
	};
	let _ = state.tx.send(serde_json::to_string(&outgoing).unwrap());
	state.user_set.lock().unwrap().remove(&outgoing.user.to_owned());
}
