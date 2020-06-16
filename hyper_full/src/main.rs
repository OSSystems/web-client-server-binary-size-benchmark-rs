// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use derive_more::{Display, Error, From};
use futures_util::lock::Mutex;
use std::{convert::Infallible, sync::Arc};

use bytes::buf::BufExt;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use rwcst::prelude::*;

#[tokio::main]
async fn main() {
    let (url, _guards) = rwcst::start_remote_mock();
    let local_client = LocalClient::new();
    let remote_client = RemoteClient::new(&url);
    let app = App::new(remote_client);
    rwcst::run(local_client, app).await;
}

struct LocalClient {
    client: hyper::Client<hyper::client::HttpConnector, hyper::body::Body>,
}

struct RemoteClient {
    client: hyper::Client<hyper::client::HttpConnector, hyper::body::Body>,
    remote: String,
}

struct App {
    info: Arc<Mutex<rwcst::Info>>,
    client: RemoteClient,
}

#[derive(Debug, Display, From, Error)]
enum Err {
    Hyper(hyper::Error),
    Parsing(rwcst::ParsingError),
    Http(http::Error),
    Uri(http::uri::InvalidUri),
}
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl rwcst::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { client: hyper::Client::new() }
    }

    async fn fetch_info(&mut self) -> Result<rwcst::Info> {
        let res = self.client.get("http://localhost:8001".parse()?).await?;
        let body = hyper::body::aggregate(res).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }
}

#[async_trait::async_trait(?Send)]
impl rwcst::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(remote: &str) -> Self {
        RemoteClient { client: hyper::Client::new(), remote: remote.to_owned() }
    }

    async fn fetch_package(&mut self) -> Result<Option<(rwcst::Package, rwcst::Signature)>> {
        let response = self.client.get(self.remote.clone().parse()?).await?;

        if let StatusCode::OK = response.status() {
            let sign = rwcst::Signature::from_base64_str(
                &response.headers().get("Signature").unwrap().to_str().unwrap(),
            );
            let pkg = rwcst::Package::parse(&hyper::body::to_bytes(response.into_body()).await?)?;
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
        async fn handle(
            req: Request<Body>,
            state: Arc<Mutex<rwcst::Info>>,
        ) -> Result<Response<Body>> {
            match (req.method(), req.uri().path()) {
                (&Method::GET, "/") => {
                    use std::ops::Deref;

                    let state = state.lock().await;
                    let body = serde_json::to_string(&state.deref())?;
                    Ok(Response::new(Body::from(body)))
                }
                _ => {
                    let mut not_found = Response::default();
                    *not_found.status_mut() = StatusCode::NOT_FOUND;
                    Ok(not_found)
                }
            }
        }
        let state = self.info.clone();
        let make_svc = make_service_fn(move |_conn| {
            let state = state.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let state = state.clone();
                    handle(req, state)
                }))
            }
        });
        let addr = ([127, 0, 0, 1], 8001).into();

        tokio::spawn(async move { Server::bind(&addr).serve(make_svc).await });

        Ok(())
    }

    async fn map_info<F: FnOnce(&mut rwcst::Info)>(&mut self, f: F) -> Result<()> {
        use std::ops::DerefMut;
        Ok(f(self.info.lock().await.deref_mut()))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
