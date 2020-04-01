// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use rwcst::{dummy, prelude::*};

#[tokio::main]
async fn main() {
    let (url, _guards) = rwcst::start_remote_mock();
    let local_client = dummy::LocalClient::new();
    let remote_client = dummy::RemoteClient::new(&url);
    let app = dummy::App::new(remote_client);
    rwcst::run(local_client, app).await;
}
