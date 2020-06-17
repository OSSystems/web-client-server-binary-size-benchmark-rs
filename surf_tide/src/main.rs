// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;
use futures_util::lock::Mutex;
use http_client::native::NativeClient;
use std::sync::Arc;

use rwcst::prelude::*;

#[async_std::main]
async fn main() {
    let (url, _guards) = rwcst::start_remote_mock();
    let local_client = LocalClient::new();
    let remote_client = RemoteClient::new(&url);
    let app = App::new(remote_client);
    rwcst::run(local_client, app).await;
}

struct LocalClient {
    client: surf::Client<NativeClient>,
}

struct RemoteClient {
    client: surf::Client<NativeClient>,
    remote: String,
}

struct App {
    info: Arc<Mutex<rwcst::Info>>,
    client: RemoteClient,
}

#[derive(Debug, From)]
enum Err {
    Http(tide::Error),
    Parsing(rwcst::ParsingError),
    Io(std::io::Error),
}
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl rwcst::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { client: surf::Client::new() }
    }

    async fn fetch_info(&mut self) -> Result<rwcst::Info> {
        Ok(self.client.get("http://localhost:8001").recv_json().await?)
    }
}

#[async_trait::async_trait(?Send)]
impl rwcst::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(remote: &str) -> Self {
        RemoteClient { client: surf::Client::new(), remote: remote.to_owned() }
    }

    async fn fetch_package(&mut self) -> Result<Option<(rwcst::Package, rwcst::Signature)>> {
        let mut response = self.client.get(&self.remote).await?;

        if let surf::http_types::StatusCode::Ok = response.status() {
            let sign =
                rwcst::Signature::from_base64_str(&response.header("signature").unwrap().as_str());
            let pkg = rwcst::Package::parse(&response.body_bytes().await?)?;
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
        use std::ops::Deref;

        let state = self.info.clone();
        let mut app = tide::with_state(state);
        app.at("/").get(|req: tide::Request<Arc<Mutex<rwcst::Info>>>| async move {
            let state = &req.state().lock().await;
            let mut res = tide::Response::new(200);
            res.set_body(tide::Body::from_json(&state.deref())?);
            Ok(res)
        });

        async_std::task::spawn(async { app.listen("127.0.0.1:8001").await });

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
