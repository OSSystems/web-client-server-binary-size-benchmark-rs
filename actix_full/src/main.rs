// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use futures_util::lock::Mutex;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use rwcst::prelude::*;

#[actix_rt::main]
async fn main() {
    let (url, _guards) = rwcst::start_remote_mock();
    let local_client = LocalClient::new();
    let remote_client = RemoteClient::new(&url);
    let app = App::new(remote_client);
    rwcst::run(local_client, app).await;
}

struct LocalClient {
    client: awc::Client,
}

struct RemoteClient {
    client: awc::Client,
    remote: String,
}

struct App {
    info: Arc<Mutex<rwcst::Info>>,
    client: RemoteClient,
}

#[derive(Debug, derive_more::From)]
enum Err {
    Server(actix_web::Error),
    Client(awc::error::SendRequestError),
    JsonPayload(awc::error::JsonPayloadError),
    Payload(awc::error::PayloadError),
    Parsing(rwcst::ParsingError),
}
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl rwcst::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { client: awc::Client::default() }
    }

    async fn fetch_info(&mut self) -> Result<rwcst::Info> {
        Ok(self.client.get("http://localhost:8001").send().await?.json().await?)
    }
}

#[async_trait::async_trait(?Send)]
impl rwcst::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(remote: &str) -> Self {
        use openssl::ssl::{SslConnector, SslMethod};
        RemoteClient {
            client: awc::Client::build()
                .connector(
                    awc::Connector::new()
                        .ssl(SslConnector::builder(SslMethod::tls()).unwrap().build())
                        .finish(),
                )
                .finish(),
            remote: remote.to_owned(),
        }
    }

    async fn fetch_package(&mut self) -> Result<Option<(rwcst::Package, rwcst::Signature)>> {
        let mut response = self.client.get(&self.remote).send().await?;

        if let actix_web::http::StatusCode::OK = response.status() {
            let sign = rwcst::Signature::from_base64_str(
                &response.headers().get("Signature").unwrap().to_str().unwrap(),
            );
            let pkg = rwcst::Package::parse(&response.body().await?)?;
            return Ok(Some((pkg, sign)));
        }

        Ok(None)
    }
}

#[async_trait::async_trait(?Send)]
impl rwcst::AppImpl for App {
    type Err = Err;
    type RemoteClient = RemoteClient;

    fn new(client: RemoteClient) -> Self {
        let info = Arc::default();
        App { info, client }
    }

    fn serve(&mut self) -> Result<()> {
        #[actix_web::get("/")]
        async fn info(
            info: actix_web::web::Data<Arc<Mutex<rwcst::Info>>>,
        ) -> actix_web::HttpResponse {
            actix_web::HttpResponse::Ok().json(info.lock().await.deref())
        }

        let info_ref = self.info.clone();
        // Start server a new thread since the runtime is single threaded
        actix_rt::Arbiter::new().exec_fn(|| {
            actix_rt::Arbiter::spawn(async {
                actix_web::HttpServer::new(move || {
                    actix_web::App::new().data(info_ref.clone()).service(info)
                })
                .workers(1)
                .bind("localhost:8001")
                .unwrap()
                .run()
                .await
                .unwrap();
            })
        });

        Ok(())
    }

    async fn map_info<F: FnOnce(&mut rwcst::Info)>(&mut self, f: F) -> Result<()> {
        Ok(f(self.info.lock().await.deref_mut()))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
