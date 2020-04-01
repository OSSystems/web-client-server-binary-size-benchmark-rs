// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use futures_util::lock::Mutex;
use std::sync::Arc;

pub struct LocalClient {
    client: awc::Client,
}

pub struct RemoteClient {
    info: Arc<Mutex<super::Info>>,
    client: awc::Client,
    remote: String,
}

pub struct App {
    info: Arc<Mutex<super::Info>>,
    client: RemoteClient,
}

// FIXME: add some real error handling
type Err = ();
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl super::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { client: awc::Client::default() }
    }

    async fn fetch_info(&mut self) -> Result<super::Info> {
        Ok(self.client.get("http://localhost:8001").send().await.unwrap().json().await.unwrap())
    }
}

#[async_trait::async_trait(?Send)]
impl super::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(remote: &str) -> Self {
        RemoteClient {
            info: Arc::default(),
            client: awc::Client::default(),
            remote: remote.to_owned(),
        }
    }

    async fn fetch_package(&mut self) -> Result<Option<(super::Package, super::Signature)>> {
        let mut response = self.client.get(&self.remote).send().await.unwrap();

        if let actix_web::http::StatusCode::OK = response.status() {
            let sign = super::Signature::from_str(
                &response.headers().get("Signature").unwrap().to_str().unwrap(),
            );
            let pkg = super::Package::parse(&response.body().await.unwrap()).unwrap();
            return Ok(Some((pkg, sign)));
        }

        Ok(None)
    }
}

#[async_trait::async_trait(?Send)]
impl super::AppImpl for App {
    type Err = Err;
    type RemoteClient = RemoteClient;

    fn new(client: RemoteClient) -> Self {
        let info = client.info.clone();
        App { info, client }
    }

    fn serve(&mut self) -> Result<()> {
        #[actix_web::get("/")]
        async fn info(
            info: actix_web::web::Data<Arc<Mutex<super::Info>>>,
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

    async fn map_info<F: FnOnce(&mut super::Info)>(&mut self, f: F) -> Result<()> {
        use std::ops::DerefMut;
        Ok(f(self.info.lock().await.deref_mut()))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
