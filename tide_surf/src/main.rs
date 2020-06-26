// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;
use futures_util::lock::Mutex;
use http_client::h1::H1Client;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bench::prelude::*;

#[async_std::main]
async fn main() {
    let (url, _guards) = bench::start_remote_mock();
    let local_client = LocalClient::new();
    let remote_client = RemoteClient::new(&url);
    let app = App::new(remote_client);
    bench::run(local_client, app).await;
}

struct LocalClient {
    client: surf::Client<H1Client>,
}

struct RemoteClient {
    client: surf::Client<H1Client>,
    remote: String,
}

struct App {
    info: Arc<Mutex<bench::Info>>,
    client: RemoteClient,
}

#[derive(Debug, From)]
enum Err {
    Http(tide::Error),
    Parsing(bench::ParsingError),
    Io(std::io::Error),
}
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl bench::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { client: surf::Client::new() }
    }

    async fn fetch_info(&mut self) -> Result<bench::Info> {
        Ok(self.client.get("http://127.0.0.1:8001").recv_json().await?)
    }
}

#[async_trait::async_trait(?Send)]
impl bench::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(remote: &str) -> Self {
        RemoteClient { client: surf::Client::new(), remote: remote.to_owned() }
    }

    async fn fetch_package(&mut self) -> Result<Option<(bench::Package, bench::Signature)>> {
        let mut response = self.client.get(&self.remote).await?;

        if let surf::http_types::StatusCode::Ok = response.status() {
            let sign =
                bench::Signature::from_base64_str(&response.header("signature").unwrap().as_str());
            let pkg = bench::Package::parse(&response.body_bytes().await?)?;
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
        let mut app = tide::with_state(state);
        app.at("/").get(|req: tide::Request<Arc<Mutex<bench::Info>>>| async move {
            let state = &req.state().lock().await;
            let mut res = tide::Response::new(200);
            res.set_body(tide::Body::from_json(&state.deref())?);
            Ok(res)
        });

        async_std::task::spawn(async { app.listen("127.0.0.1:8001").await });

        Ok(())
    }

    async fn map_info<F: FnOnce(&mut bench::Info)>(&mut self, f: F) -> Result<()> {
        Ok(f(self.info.lock().await.deref_mut()))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
