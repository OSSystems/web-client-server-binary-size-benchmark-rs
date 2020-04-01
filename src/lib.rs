// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

pub mod prelude {
    pub use super::{AppImpl, LocalClientImpl, RemoteClientImpl};
}

pub mod dummy;

#[async_trait::async_trait(?Send)]
pub trait LocalClientImpl: Sized {
    type Err: std::fmt::Debug;
    fn new() -> Self;
    async fn fetch_info(&mut self) -> Result<Info, Self::Err>;
}

#[async_trait::async_trait(?Send)]
pub trait RemoteClientImpl: Sized {
    type Err;

    fn new(url: &str) -> Self;
    async fn fetch_package(&mut self) -> Result<Option<(Package, Signature)>, Self::Err>;
}

#[async_trait::async_trait(?Send)]
pub trait AppImpl: Sized {
    type RemoteClient: RemoteClientImpl;
    type Err: From<<Self::RemoteClient as RemoteClientImpl>::Err> + std::fmt::Debug;

    fn new(client: Self::RemoteClient) -> Self;
    fn serve(&mut self) -> Result<(), Self::Err>;

    async fn map_info<F: FnOnce(&mut Info)>(&mut self, f: F) -> Result<(), Self::Err>;
    async fn client(&mut self) -> Result<&mut Self::RemoteClient, Self::Err>;

    async fn process(&mut self) -> Result<(), Self::Err> {
        match self.client().await?.fetch_package().await? {
            None => {}
            Some((pkg, sig)) => {
                if sig.validate(&pkg) {
                    self.map_info(move |info| info.current_version = pkg.version).await?;
                    return Ok(());
                }
                self.map_info(move |info| info.count_invalid_packages += 1).await?;
            }
        }

        Ok(())
    }
}

pub async fn run<C: LocalClientImpl, A: AppImpl>(mut client: C, mut app: A) {
    app.serve().unwrap(); // Start serving the app for the local client

    let info = client.fetch_info().await.unwrap();
    assert_eq!(info, Info::default(), "Info should be default as nothing has run so far");

    app.process().await.unwrap();
    let info = client.fetch_info().await.unwrap();
    assert_eq!(info, Info::default(), "Info should still be default as update will not apply yet");

    app.process().await.unwrap();
    let info = client.fetch_info().await.unwrap();
    assert_eq!(
        info,
        Info { current_version: String::from("0.0.2"), count_invalid_packages: 0 },
        "Info should show the updated current_version"
    );

    app.process().await.unwrap();
    let info = client.fetch_info().await.unwrap();
    assert_eq!(
        info,
        Info { current_version: String::from("0.0.2"), count_invalid_packages: 1 },
        "Info should show the updated current_version with the updated count of invalid packages"
    );

    app.process().await.unwrap();
    let info = client.fetch_info().await.unwrap();
    assert_eq!(
        info,
        Info { current_version: String::from("0.0.2"), count_invalid_packages: 2 },
        "Info should show increase in the count of invalid packages"
    );
}

#[derive(Debug)]
pub struct Signature(pub(crate) Vec<u8>);

impl Signature {
    const INVALID_SAMPLE: &'static str = r#"Hx6kv5dndxA/3qi9QAgXlaiyCrhKZLE7TLXVHVIU9XNq0qIyuRWCDaBDSXCbFKTgd26gBY6q30FHpxrDuf09UPnznluxv/0LbGbwyyskj4c5CZwQIGCcj+5a+ypV68G7hzFsaY3l7COvtGfQPnFT3B7JovqoLTpNgh/VtI0PHDo="#;
    /// Get a valid signature. Static signature generated with:
    /// ```shell
    /// echo -n '{"product":"fooobarrr","version":"0.0.2"}' | \
    ///   openssl dgst -sha256 -sign fixtures/ssh/key | base64
    /// ```
    const VALID_SAMPLE: &'static str = r#"xcPhKCRaL3YheiVvJOhypjFKW7e8sJzyIve2k+Higp+BtB5ED31rW3wl/noDqvIA7YVyWVnEE/nzRfRrjNOE1ylbxwUuOsjRamCr2y6C8q7rBshA6msRmwsVAmIKHcjGWhL/p1bF9WjS7vNbItx0ujHuDlqgTwutvM9XN702IjE="#;

    pub fn from_str(content: &str) -> Self {
        Signature(openssl::base64::decode_block(content).unwrap().to_vec())
    }

    pub fn validate(&self, pkg: &Package) -> bool {
        use openssl::{hash::MessageDigest, pkey::PKey, rsa::Rsa, sign::Verifier};
        let fun = move || {
            let content = &std::fs::read("fixtures/ssh/key.pub").unwrap();
            let key = Rsa::public_key_from_pem(content)?;
            let key = PKey::from_rsa(key)?;
            let mut ver = Verifier::new(MessageDigest::sha256(), &key)?;
            Result::<bool, openssl::error::ErrorStack>::Ok(ver.verify_oneshot(&self.0, &pkg.raw)?)
        };
        fun().unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct Package {
    product_uid: String,
    version: String,
    raw: Vec<u8>,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            product_uid: String::from("fooobarrr"),
            version: String::from("0.0.2"),
            raw: br#"{"product":"fooobarrr","version":"0.0.2"}"#.to_vec(),
        }
    }
}

impl Package {
    pub(crate) fn parse(content: &[u8]) -> serde_json::Result<Self> {
        #[derive(Deserialize)]
        struct PackageAux {
            #[serde(rename = "product")]
            product_uid: String,
            version: String,
        }

        let update_package = serde_json::from_slice::<PackageAux>(content)?;
        Ok(Package {
            product_uid: update_package.product_uid,
            version: update_package.version,
            raw: content.to_vec(),
        })
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Info {
    current_version: String,
    count_invalid_packages: u32,
}

impl Default for Info {
    fn default() -> Self {
        Info { current_version: String::from("0.0.1"), count_invalid_packages: 0 }
    }
}
