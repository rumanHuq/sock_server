use std::{collections::HashSet, sync::Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

pub struct AppState {
	pub user_set: Mutex<HashSet<String>>,
	pub tx: broadcast::Sender<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Incoming {
	pub user: String,
	pub event: String,
	pub payload: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Outgoing {
	pub user: String,
	pub event: String,
	pub payload: String,
}
