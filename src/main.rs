#![allow(unused_variables)]

use client_server_testing::dummy as curr;
use client_server_testing::prelude::*;

fn main() {
    let server = curr::Server::new();
    let local_client = curr::LocalClient::new();
    let remote_client = curr::RemoteClient::new();
    server.run();
}
