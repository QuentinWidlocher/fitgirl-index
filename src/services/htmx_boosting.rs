use axum::response::IntoResponse;
use axum::{middleware::Next, response::Response};
use axum_extra::response::Html;
use hyper::{Body, Request};
use maud::PreEscaped;
use tower_livereload::predicate::Predicate;

use crate::components::layout::layout;

#[derive(Copy, Clone)]
struct HtmxBoostingPredicate;

impl Predicate<Request<Body>> for HtmxBoostingPredicate {
	fn check(&mut self, request: &Request<Body>) -> bool {
		request
			.headers()
			.keys()
			.all(|header| header != "Hx-Boosted")
	}
}

pub async fn htmx_boosting<B>(request: Request<B>, next: Next<B>) -> Response {
	let boosted = request
		.headers()
		.keys()
		.any(|header| header == "Hx-Boosted");

	let html = request.headers().iter().any(|(header_name, header_value)| {
		header_name == "accept" && header_value.to_str().unwrap_or("").contains("text/html")
	});

	if !html || boosted {
		next.run(request).await
	} else {
		// wrap response in layout()
		let response = next.run(request).await;
		let body = response.into_body();
		let body = hyper::body::to_bytes(body).await.unwrap();
		let body = String::from_utf8(body.to_vec()).unwrap();
		let body = layout(PreEscaped(body));
		Html(body).into_response()
	}
}
