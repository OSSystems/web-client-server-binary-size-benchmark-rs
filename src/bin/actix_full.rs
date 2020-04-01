// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use rwcst::{actix_full, prelude::*};

#[actix_rt::main]
async fn main() {
    let (url, _guards) = rwcst::start_remote_mock();
    let local_client = actix_full::LocalClient::new();
    let remote_client = actix_full::RemoteClient::new(&url);
    let app = actix_full::App::new(remote_client);
    rwcst::run(local_client, app).await;
}
