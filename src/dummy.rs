// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

pub struct LocalClient {
    requests: u32,
}

pub struct RemoteClient {
    requests: u32,
}

pub struct App {
    info: super::Info,
    client: RemoteClient,
}

type Err = ();
type Result<T> = std::result::Result<T, Err>;

#[async_trait::async_trait(?Send)]
impl super::LocalClientImpl for LocalClient {
    type Err = Err;

    fn new() -> Self {
        LocalClient { requests: 0 }
    }

    async fn fetch_info(&mut self) -> Result<super::Info> {
        let info = super::Info::default();
        let res = match self.requests {
            0 | 1 => info,
            2 => super::Info { current_version: String::from("0.0.2"), ..info },
            n => super::Info {
                current_version: String::from("0.0.2"),
                count_invalid_packages: n - 2,
            },
        };
        self.requests += 1;
        Ok(res)
    }
}

#[async_trait::async_trait(?Send)]
impl super::RemoteClientImpl for RemoteClient {
    type Err = Err;

    fn new(_: &str) -> Self {
        RemoteClient { requests: 0 }
    }

    async fn fetch_package(&mut self) -> Result<Option<(super::Package, super::Signature)>> {
        let res = match self.requests {
            0 => None,
            1 => Some((
                super::Package::parse(&super::Package::default().raw).unwrap(),
                super::Signature::from_str(super::Signature::VALID_SAMPLE),
            )),
            _ => Some((
                super::Package::parse(&super::Package::default().raw).unwrap(),
                super::Signature::from_str(super::Signature::INVALID_SAMPLE),
            )),
        };
        self.requests += 1;
        Ok(res)
    }
}

#[async_trait::async_trait(?Send)]
impl super::AppImpl for App {
    type Err = Err;
    type RemoteClient = RemoteClient;

    fn new(client: RemoteClient) -> Self {
        App { info: super::Info::default(), client }
    }

    fn serve(&mut self) -> Result<()> {
        Ok(())
    }

    async fn map_info<F: FnOnce(&mut super::Info)>(&mut self, f: F) -> Result<()> {
        Ok(f(&mut self.info))
    }

    async fn client(&mut self) -> Result<&mut RemoteClient> {
        Ok(&mut self.client)
    }
}
