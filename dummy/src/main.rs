// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

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
    requests: u32,
}

struct RemoteClient {
    requests: u32,
}

struct App {
    info: rwcst::Info,
    client: RemoteClient,
}

type Err = ();
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl rwcst::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { requests: 0 }
    }

    async fn fetch_info(&mut self) -> Result<rwcst::Info> {
        let info = rwcst::Info::default();
        let res = match self.requests {
            0 | 1 => info,
            2 => rwcst::Info { current_version: String::from("0.0.2"), ..info },
            n => rwcst::Info {
                current_version: String::from("0.0.2"),
                count_invalid_packages: n - 2,
            },
        };
        self.requests += 1;
        Ok(res)
    }
}

#[async_trait::async_trait(?Send)]
impl rwcst::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(_: &str) -> Self {
        RemoteClient { requests: 0 }
    }

    async fn fetch_package(&mut self) -> Result<Option<(rwcst::Package, rwcst::Signature)>> {
        let res = match self.requests {
            0 => None,
            1 => Some((
                rwcst::Package::parse(&rwcst::Package::default().raw).unwrap(),
                rwcst::Signature::from_str(rwcst::Signature::VALID_SAMPLE),
            )),
            _ => Some((
                rwcst::Package::parse(&rwcst::Package::default().raw).unwrap(),
                rwcst::Signature::from_str(rwcst::Signature::INVALID_SAMPLE),
            )),
        };
        self.requests += 1;
        Ok(res)
    }
}

#[async_trait::async_trait(?Send)]
impl rwcst::AppImpl for App {
    type Err = Err;
    type RemoteClient = RemoteClient;

    fn new(client: RemoteClient) -> Self {
        App { info: rwcst::Info::default(), client }
    }

    fn serve(&mut self) -> Result<()> {
        Ok(())
    }

    async fn map_info<F: FnOnce(&mut rwcst::Info)>(&mut self, f: F) -> Result<()> {
        Ok(f(&mut self.info))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
