use anyhow::Context;
use chrono::Utc;

use hyper::client::Response;
use hyper::Client;
use oauth::{
    ApplicationSecret, Authenticator, DefaultAuthenticatorDelegate, GetToken, MemoryStorage,
    ServiceAccountAccess, ServiceAccountKey,
};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};
use tokio::io::ErrorKind;
use tokio::time::{delay_for, Duration};
use yup_oauth2 as oauth;

use google_apis::google::firestore::v1::value::ValueType::{
    BooleanValue, DoubleValue, IntegerValue, ReferenceValue, StringValue, TimestampValue,
};
use google_apis::google::firestore::v1::{DocumentMask, GetDocumentRequest};
use google_apis::google::firestore::v1::{ListDocumentsRequest, Value};
use google_apis::v1::firestore_client::FirestoreClient;
use hyper_native_tls::NativeTlsClient;
use tonic::metadata::MetadataValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::Request;

const ENDPOINT: &str = "https://firestore.googleapis.com";
const PARENT_PATH_FIRESTORE: &str = "projects/rust-playground-ad891/databases/(default)/documents";
const CERTIFICATES: &[u8] = include_bytes!("./roots.pem");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let service_account_key =
        oauth::service_account_key_from_file(&"./src/firestore/service_account.json".to_string())
            .unwrap();

    let mut access = yup_oauth2::ServiceAccountAccess::new(
        service_account_key,
        hyper::Client::with_connector(hyper::net::HttpsConnector::new(
            NativeTlsClient::new().unwrap(),
        )),
    );
    let scopes = &["https://www.googleapis.com/auth/datastore"];
    let token = access.token(scopes).unwrap();
    println!("{:?}", token);

    let tls_config = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(CERTIFICATES))
        .domain_name("firestore.googleapis.com");

    let channel = Channel::from_static(ENDPOINT)
        .tls_config(tls_config)?
        .connect()
        .await?;

    let mut service = FirestoreClient::with_interceptor(channel, move |mut req: Request<()>| {
        let formatted_token = format!("Bearer {}", token.access_token.as_str());
        req.metadata_mut().insert(
            "authorization",
            MetadataValue::from_str(formatted_token.as_str()).unwrap(),
        );
        Ok(req)
    });

    let response = service
        .list_documents(ListDocumentsRequest {
            parent: PARENT_PATH_FIRESTORE.to_string(),
            collection_id: "test".to_string(),
            page_size: 0,
            page_token: "".to_string(),
            order_by: "".to_string(),
            mask: None,
            show_missing: false,
            consistency_selector: None,
        })
        .await?;
    let documents = response.into_inner().documents;
    for document in documents {
        println!(
            "{:?}",
            document_from_map(document.name, &document.fields).unwrap()
        );
    }

    Ok(())
}

#[derive(Debug)]
struct Task {
    document_id: String,
    name: String,
    done: bool,
}

fn document_from_map(document_id: String, fields: &HashMap<String, Value>) -> Result<Task, Error> {
    let name_type = fields.get("name").unwrap().clone().value_type.unwrap();
    let name = match name_type {
        StringValue(value) => Ok(value.clone()),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid Type")),
    }?;
    let done_type = fields
        .get("done")
        .unwrap()
        .clone()
        .value_type
        .unwrap()
        .clone();
    let done = match done_type {
        BooleanValue(value) => Ok(value.clone()),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid Type")),
    }?;
    Ok(Task {
        document_id,
        name,
        done,
    })
}

fn load_service_account(service_account: &str) -> Result<ServiceAccountKey, Error> {
    match serde_json::from_str(service_account) {
        Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{}", e))),
        Ok(decoded) => Ok(decoded),
    }
}

// struct FirestoreClient<C, A> {
//     hub: Firestore<C, A>,
// }
//
// impl FirestoreClient<Client, ServiceAccountAccess<Client>> {
//     pub fn new(
//         service_account_key: ServiceAccountKey,
//     ) -> FirestoreClient<Client, ServiceAccountAccess<Client>> {
//         let mut access = yup_oauth2::ServiceAccountAccess::new(
//             service_account_key,
//             hyper::Client::with_connector(hyper::net::HttpsConnector::new(
//                 hyper_rustls::TlsClient::new(),
//             )),
//         );
//         let mut hub: Firestore<Client, ServiceAccountAccess<Client>> = Firestore::new(
//             hyper::Client::with_connector(hyper::net::HttpsConnector::new(
//                 hyper_rustls::TlsClient::new(),
//             )),
//             access,
//         );
//         FirestoreClient { hub: hub }
//     }
//
//     fn create_document(
//         &self,
//         document_path: &str,
//         document_value: HashMap<String, Value>,
//     ) -> Result<Document, google_firestore1::Error> {
//         let mut req = Document {
//             fields: Some(document_value),
//             ..Document::default()
//         };
//
//         self.hub
//             .projects()
//             .databases_documents_create_document(req, PARENT_PATH_FIRESTORE, document_path)
//             .doit()
//             .map(|value| value.1)
//     }
//
//     fn fetch_documents(
//         &self,
//         collection_path: &str,
//     ) -> Result<ListDocumentsResponse, google_firestore1::Error> {
//         self.hub
//             .projects()
//             .databases_documents_list(PARENT_PATH_FIRESTORE, collection_path)
//             .doit()
//             .map(|value| value.1)
//     }
// }
