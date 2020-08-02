use anyhow::Context;
use chrono::Utc;
use google_firestore1::{
    Document, Firestore, GoogleFirestoreAdminV1Field, GoogleFirestoreAdminV1Index,
    ListDocumentsResponse, ProjectMethods, Value,
};
use hyper::client::Response;
use hyper::Client;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};
use tokio::io::ErrorKind;
use tokio::time::{delay_for, Duration};
use yup_oauth2::{
    ApplicationSecret, Authenticator, DefaultAuthenticatorDelegate, MemoryStorage,
    ServiceAccountAccess, ServiceAccountKey,
};

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let service_account_key = yup_oauth2::service_account_key_from_file(
        &"./src/firestore/service_account.json".to_string(),
    )
    .unwrap();
    let firestore_client = FirestoreClient::new(service_account_key);
    let result = firestore_client.fetch_documents("test");
    match result {
        Err(e) => match e {
            google_firestore1::Error::HttpError(_)
            | google_firestore1::Error::MissingAPIKey
            | google_firestore1::Error::MissingToken(_)
            | google_firestore1::Error::Cancelled
            | google_firestore1::Error::UploadSizeLimitExceeded(_, _)
            | google_firestore1::Error::Failure(_)
            | google_firestore1::Error::BadRequest(_)
            | google_firestore1::Error::FieldClash(_)
            | google_firestore1::Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res),
    }

    let map = [
        (
            "name".to_string(),
            Value {
                string_value: Some("chiba".to_string()),
                ..Value::default()
            },
        ),
        (
            "done".to_string(),
            Value {
                boolean_value: Some(true),
                ..Value::default()
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let result = firestore_client.create_document("test", map);
    match result {
        Err(e) => match e {
            google_firestore1::Error::HttpError(_)
            | google_firestore1::Error::MissingAPIKey
            | google_firestore1::Error::MissingToken(_)
            | google_firestore1::Error::Cancelled
            | google_firestore1::Error::UploadSizeLimitExceeded(_, _)
            | google_firestore1::Error::Failure(_)
            | google_firestore1::Error::BadRequest(_)
            | google_firestore1::Error::FieldClash(_)
            | google_firestore1::Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res),
    }
}

const PARENT_PATH_FIRESTORE: &str = "projects/rust-playground-ad891/databases/(default)/documents";

fn load_service_account(service_account: &str) -> Result<ServiceAccountKey, Error> {
    match serde_json::from_str(service_account) {
        Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{}", e))),
        Ok(decoded) => Ok(decoded),
    }
}

struct FirestoreClient<C, A> {
    hub: Firestore<C, A>,
}

impl FirestoreClient<Client, ServiceAccountAccess<Client>> {
    pub fn new(
        service_account_key: ServiceAccountKey,
    ) -> FirestoreClient<Client, ServiceAccountAccess<Client>> {
        let mut access = yup_oauth2::ServiceAccountAccess::new(
            service_account_key,
            hyper::Client::with_connector(hyper::net::HttpsConnector::new(
                hyper_rustls::TlsClient::new(),
            )),
        );
        let mut hub: Firestore<Client, ServiceAccountAccess<Client>> = Firestore::new(
            hyper::Client::with_connector(hyper::net::HttpsConnector::new(
                hyper_rustls::TlsClient::new(),
            )),
            access,
        );
        FirestoreClient { hub: hub }
    }

    fn create_document(
        &self,
        document_path: &str,
        document_value: HashMap<String, Value>,
    ) -> Result<Document, google_firestore1::Error> {
        let mut req = Document {
            fields: Some(document_value),
            ..Document::default()
        };

        self.hub
            .projects()
            .databases_documents_create_document(req, PARENT_PATH_FIRESTORE, document_path)
            .doit()
            .map(|value| value.1)
    }

    fn fetch_documents(
        &self,
        collection_path: &str,
    ) -> Result<ListDocumentsResponse, google_firestore1::Error> {
        self.hub
            .projects()
            .databases_documents_list(PARENT_PATH_FIRESTORE, collection_path)
            .doit()
            .map(|value| value.1)
    }
}
