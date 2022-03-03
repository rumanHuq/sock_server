use axum::response::Html;

pub mod chat;

pub async fn home() -> Html<&'static str> {
	Html(std::include_str!("../../web/index.html"))
}
