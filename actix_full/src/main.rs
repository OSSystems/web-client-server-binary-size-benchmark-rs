// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use futures_util::lock::Mutex;
use std::sync::Arc;

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
    info: Arc<Mutex<rwcst::Info>>,
    client: awc::Client,
    remote: String,
}

struct App {
    info: Arc<Mutex<rwcst::Info>>,
    client: RemoteClient,
}

// FIXME: add some real error handling
type Err = ();
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl rwcst::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { client: awc::Client::default() }
    }

    async fn fetch_info(&mut self) -> Result<rwcst::Info> {
        Ok(self.client.get("http://localhost:8001").send().await.unwrap().json().await.unwrap())
    }
}

#[async_trait::async_trait(?Send)]
impl rwcst::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(remote: &str) -> Self {
        RemoteClient {
            info: Arc::default(),
            client: awc::Client::default(),
            remote: remote.to_owned(),
        }
    }

    async fn fetch_package(&mut self) -> Result<Option<(rwcst::Package, rwcst::Signature)>> {
        let mut response = self.client.get(&self.remote).send().await.unwrap();

        if let actix_web::http::StatusCode::OK = response.status() {
            let sign = rwcst::Signature::from_str(
                &response.headers().get("Signature").unwrap().to_str().unwrap(),
            );
            let pkg = rwcst::Package::parse(&response.body().await.unwrap()).unwrap();
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
        let info = client.info.clone();
        App { info, client }
    }

    fn serve(&mut self) -> Result<()> {
        #[actix_web::get("/")]
        async fn info(
            info: actix_web::web::Data<Arc<Mutex<rwcst::Info>>>,
        ) -> Result<actix_web::HttpResponse> {
            use std::ops::Deref;
            Ok(actix_web::HttpResponse::Ok().json(info.lock().await.deref()))
        }

        let info_ref = self.info.clone();
        actix_rt::Arbiter::new().exec_fn(|| {
            actix_rt::Arbiter::spawn(async move {
                let info_ref = info_ref;
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

        // Give time for the server to actually start
        std::thread::sleep(std::time::Duration::from_secs(1));

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
