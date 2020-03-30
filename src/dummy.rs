pub struct RemoteClient;
pub struct LocalClient;
pub struct Server;

impl super::RemoteClientImpl for RemoteClient {
    fn new() -> Self {
        RemoteClient
    }
}

impl super::LocalClientImpl for LocalClient {
    fn new() -> Self {
        LocalClient
    }
}

impl super::ServerImpl for Server {
    fn new() -> Self {
        Server
    }

    fn run(self) {}
}
