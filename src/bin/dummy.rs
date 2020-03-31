// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use rwcst::{dummy, prelude::*};

fn main() {
    let local_client = dummy::LocalClient::new();
    let remote_client = dummy::RemoteClient::new("https://foo.bar");
    let app = dummy::App::new(remote_client);
    rwcst::run(local_client, app);
}
