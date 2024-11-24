use chrono::NaiveDateTime;
use derive_builder::Builder;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

pub mod genres;
pub mod last_releases;
pub mod release;
pub mod search;

pub async fn get_client() -> Client {
	let mut builder =
		SslConnector::builder(SslMethod::tls()).expect("unable to create sslconnector builder");
	builder.set_verify(SslVerifyMode::NONE);
	let connector = MakeTlsConnector::new(builder.build());
	let db_url = std::env::var("DB_URL").expect("MAILCOACH_API_TOKEN must be set.");

	let x = tokio_postgres::connect(db_url.as_str(), connector).await;
	if let Err(e) = &x {
		eprintln!("connection error: {}", e);
	}
	let (client, connection) = x.unwrap();

	tokio::spawn(async move {
		if let Err(e) = connection.await {
			eprintln!("connection error: {}", e);
		}
	});

	client
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
	pub name: String,
	pub link: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Mirror {
	pub links: Vec<Link>,
}

#[derive(Builder, Debug)]
#[builder(setter(into))]
#[builder(derive(Debug))]
pub struct Release {
	pub title: String,
	pub link: String,
	pub published: NaiveDateTime,
	pub cover_src: String,
	pub original_size: String,
	pub repack_size: String,
	pub mirrors: Vec<Mirror>,
	pub screenshots: Vec<String>,
	pub repack_description: String,
	pub game_description: String,
}

pub struct ReleaseWithInfo {
	pub release: Release,
	pub genres: Vec<String>,
}
