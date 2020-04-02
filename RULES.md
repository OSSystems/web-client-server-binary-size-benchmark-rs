# Implementations rules

Here are some details of how implementations should be made.

## Asynchronous API

The server and client are not necessarily required to be `async/await`,
but they need to follow the specified API to make it compatible with the rest of the code base.
Blocking methods wrapped around an `async` block is acceptable as we have ways to work around that.

## Internal State

The internal state is represented by the `Info` structure defined publicly at the [lib](src/lib.rs#L178),
but the handling of the internal state is done by the lib itself.
Implementations only need to provided the specified [trait methods](src/lib.rs#L36):

```Rust
async fn map_info<F: FnOnce(&mut Info)>(&mut self, f: F) -> Result<(), Self::Err>;
async fn client(&mut self) -> Result<&mut Self::RemoteClient, Self::Err>;
```

Where `map_info` will run a closure to change the internal state's value,
and `client` offers the caller access to the `RemoteClient`.

## Client's and Server Methods

The local server can be started at any port,
provided that the local client can connect to it.
The server only needs to respond to a single request:

```
URL: "/"
Method: GET
Responses: 200
Response 200:
  Header:
    content-type: application/json
  Body:
    "json formated Info structure"
```

The remote client should make it's requests to the mock server.
This mock server can be started by calling [start_remote_mock](src/lib.rs#L90),
and the `Vec` argument of the function should not be dropped until the main finishes.
The single request that has to be made to the mock is described as follow:
```
URL: "/"
Method: GET
Responses: [200, 404]
Response 200:
  Header:
    content-type: application/json
    Signature: "some_base64_string"
  Body:
    "json formated package structure"
Response 404:
  Header:
  Body:
```

## Main implementation

The main only has to do three basic things,
(1) initialize it's own structures;
(2) call [run](src/lib.rs#L55) function from the lib;
(3) `.await` for it's completion.

The `run` function will perform a couple of requests and assert everything is working as intended.
It's main propose is to ensure that all the needed functionalities are linked to the main binary,
so we can analyze it's performance and usage.
