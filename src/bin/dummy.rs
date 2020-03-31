// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use client_server_testing::{dummy, prelude::*};

fn main() {
    let local_client = dummy::LocalClient::new();
    let remote_client = dummy::RemoteClient::new("https://foo.bar");
    let app = dummy::App::new(remote_client);
    client_server_testing::run(local_client, app);
}
