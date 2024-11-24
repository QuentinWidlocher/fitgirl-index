use std::usize;

use axum::{
	body::to_bytes,
	extract::Request,
	middleware::Next,
	response::{IntoResponse, Response},
};
use maud::PreEscaped;

use crate::components::layout::layout;

pub async fn htmx_boosting(request: Request, next: Next) -> Response {
	let boosted = request
		.headers()
		.keys()
		.any(|header| header == "Hx-Boosted");

	let html = request.headers().iter().any(|(header_name, header_value)| {
		header_name == "accept" && header_value.to_str().unwrap_or("").contains("text/html")
	});

	let response = next.run(request).await;

	if !html || boosted {
		response
	} else {
		// wrap response in layout()
		let body = response.into_body();
		let body = to_bytes(body, usize::MAX).await.unwrap();
		let body = String::from_utf8(body.to_vec()).unwrap();
		let body = layout(PreEscaped(body));
		body.into_response()
	}
}
