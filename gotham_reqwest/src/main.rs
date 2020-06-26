// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
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

#[derive(Debug, From)]
enum Err {
    Server(gotham::error::Error),
    Client(reqwest::Error),
    Parsing(bench::ParsingError),
    MutexPosion,
}

impl<T> From<std::sync::PoisonError<T>> for Err {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Err::MutexPosion
    }
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
        use gotham::{
            handler::{HandlerResult, IntoHandlerError},
            helpers::http::response::create_response,
            hyper::StatusCode,
            middleware::state::StateMiddleware,
            pipeline::{single::single_pipeline, single_middleware},
            router::builder::{build_router, DefineSingleRoute, DrawRoutes},
            state::State,
        };
        use gotham_derive::StateData;

        #[derive(Clone, StateData)]
        struct Info(Arc<Mutex<bench::Info>>);

        async fn handle(state: State) -> HandlerResult {
            let res = match serde_json::to_string(state.borrow::<Info>().0.deref()) {
                Ok(body) => {
                    let response =
                        create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
                    Ok((state, response))
                }
                Err(e) => Err((state, e.into_handler_error())),
            };
            return res;
        }

        let info = self.info.clone();
        let srv = gotham::init_server("127.0.0.1:8001", {
            let middleware = StateMiddleware::new(Info(info.clone()));
            let pipeline = single_middleware(middleware);
            let (chain, pipelines) = single_pipeline(pipeline);

            build_router(chain, pipelines, |route| {
                route.get("/").to_async(handle);
            })
        });

        tokio::spawn(async { srv.await.unwrap() });

        Ok(())
    }

    async fn map_info<F: FnOnce(&mut bench::Info)>(&mut self, f: F) -> Result<()> {
        Ok(f(self.info.lock()?.deref_mut()))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
