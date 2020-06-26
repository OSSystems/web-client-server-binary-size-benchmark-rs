// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use derive_more::{Display, Error, From};
use futures_util::lock::Mutex;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Response, Server, StatusCode,
};
use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bench::prelude::*;

#[tokio::main]
async fn main() {
    let (url, _guards) = bench::start_remote_mock();
    let local_client = LocalClient::new();
    let remote_client = RemoteClient::new(&url);
    let app = App::new(remote_client);
    bench::run(local_client, app).await;
}

struct LocalClient {
    client: reqwest::Client,
}

struct RemoteClient {
    client: reqwest::Client,
    remote: String,
}

struct App {
    info: Arc<Mutex<bench::Info>>,
    client: RemoteClient,
}

#[derive(Debug, Display, Error, From)]
enum Err {
    Server(hyper::Error),
    Client(reqwest::Error),
    Parsing(bench::ParsingError),
}
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl bench::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { client: reqwest::Client::new() }
    }

    async fn fetch_info(&mut self) -> Result<bench::Info> {
        Ok(self.client.get("http://localhost:8001").send().await?.json().await?)
    }
}

#[async_trait::async_trait(?Send)]
impl bench::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(remote: &str) -> Self {
        RemoteClient { client: reqwest::Client::new(), remote: remote.to_owned() }
    }

    async fn fetch_package(&mut self) -> Result<Option<(bench::Package, bench::Signature)>> {
        let response = self.client.get(&self.remote).send().await?;

        if let reqwest::StatusCode::OK = response.status() {
            let sign = bench::Signature::from_base64_str(
                &response.headers().get("Signature").unwrap().to_str().unwrap(),
            );
            let pkg = bench::Package::parse(&response.bytes().await?)?;
            return Ok(Some((pkg, sign)));
        }

        Ok(None)
    }
}

#[async_trait::async_trait(?Send)]
impl bench::AppImpl for App {
    type Err = Err;
    type RemoteClient = RemoteClient;

    fn new(client: RemoteClient) -> Self {
        let info = Arc::default();
        App { info, client }
    }

    fn serve(&mut self) -> Result<()> {
        let state = self.info.clone();
        let make_svc = make_service_fn(move |_conn| {
            let state = state.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let state = state.clone();
                    async move {
                        match (req.method(), req.uri().path()) {
                            (&Method::GET, "/") => {
                                let state = state.lock().await;
                                let body = serde_json::to_string(&state.deref())?;
                                Ok(Response::new(Body::from(body)))
                            }
                            _ => {
                                let mut not_found = Response::default();
                                *not_found.status_mut() = StatusCode::NOT_FOUND;
                                Ok::<_, Err>(not_found)
                            }
                        }
                    }
                }))
            }
        });
        let addr = ([127, 0, 0, 1], 8001).into();

        tokio::spawn(async move { Server::bind(&addr).serve(make_svc).await });

        Ok(())
    }

    async fn map_info<F: FnOnce(&mut bench::Info)>(&mut self, f: F) -> Result<()> {
        Ok(f(self.info.lock().await.deref_mut()))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
