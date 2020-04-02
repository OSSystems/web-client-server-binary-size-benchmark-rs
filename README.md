# (wip) rust-web-client-server-testing

This small benchmark is intended to compare how different combinations of HTTP
Client and HTTP Server will affect the binary size of applications.

The base application we will be covering here has some basic requirements that
are based on needs that we have for our real-world app.
The app also have some usage of OpenSSL,
which is also a requirement of our app,
as we need to handle RSA signature validation,
so if any implementation make use of code defined there will have a valid advantage.
This are the requirement for all binaries implementation:

* A running server to respond to requests from a local client;
* A client to make local requests to the server;
* A second client to make requests to a remote server;
* Clients needs to handle:
  * JSON serialization and deserialization;
  * HTTP Header extraction;
* Share state between the Remote Client and the App;
* Asynchronous type signature;

## Implementations rules

The [RULES.md](RULES.md) file describe how implementations should be made.

## Contestants

| Implementation | Client |   Server  |                         Description                   |
|:--------------:|:------:|:---------:|:-----------------------------------------------------:|
|      dummy     |    -   |     -     | Used as size baseline for the lib                     |
|   actix_full   |   awc  | actix-web | Implementation fully based on actix-web using OpenSSL |

## (wip) Binary size 

Output generated from `cargo bloat`.

FIXME: Maybe would be a good a idea to prepare a script for filtering relevant
(changed from dummy)
creates from cargo bloat's output.
This probably would make analysis more easy to visualize.

### dummy
```
 File  .text     Size Crate
 5.4%  25.4% 250.3KiB std
 4.9%  23.1% 228.1KiB regex_syntax
 3.3%  15.3% 150.8KiB regex
 2.1%   9.9%  97.2KiB mockito
 1.6%   7.5%  74.4KiB aho_corasick
 0.9%   4.1%  40.3KiB [Unknown]
 0.7%   3.5%  34.2KiB serde_json
 0.7%   3.3%  32.9KiB assert_json_diff
 0.4%   2.0%  19.8KiB rwcst
 0.2%   1.1%  10.4KiB rand_chacha
 0.1%   0.6%   5.8KiB openssl
 0.1%   0.4%   4.0KiB tokio
 0.1%   0.3%   3.4KiB httparse
 0.1%   0.3%   3.2KiB memchr
 0.1%   0.3%   2.8KiB ryu
 0.0%   0.2%   2.3KiB getrandom
 0.0%   0.2%   1.8KiB thread_local
 0.0%   0.2%   1.6KiB percent_encoding
 0.0%   0.1%    1016B serde
 0.0%   0.1%     984B cc
 0.0%   0.2%   1.5KiB And 7 more crates. Use -n N to show more.
21.4% 100.0% 985.9KiB .text section size, the file size is 4.5MiB

Note: numbers above are a result of guesswork. They are not 100% correct and never will be.
```

### actix_full
```
 File  .text     Size Crate
 8.1%  22.2% 872.1KiB std
 5.8%  15.7% 618.0KiB awc
 2.3%   6.2% 244.3KiB h2
 2.1%   5.9% 230.2KiB regex_syntax
 2.1%   5.7% 223.8KiB regex
 1.5%   4.1% 163.2KiB actix_server
 1.5%   4.1% 161.8KiB trust_dns_proto
 1.3%   3.7% 144.0KiB actix_http
 1.1%   3.0% 119.3KiB tokio
 1.0%   2.6% 102.4KiB [Unknown]
 0.9%   2.5%  97.4KiB mockito
 0.8%   2.3%  90.5KiB trust_dns_resolver
 0.7%   1.9%  74.4KiB aho_corasick
 0.7%   1.9%  73.8KiB actix_rt
 0.7%   1.9%  73.2KiB actix_web
 0.5%   1.3%  52.7KiB http
 0.5%   1.3%  50.4KiB url
 0.4%   1.2%  47.4KiB serde_json
 0.4%   1.1%  42.7KiB rustc_demangle
 0.3%   0.8%  32.9KiB assert_json_diff
 3.0%   8.2% 322.8KiB And 57 more crates. Use -n N to show more.
36.6% 100.0%   3.8MiB .text section size, the file size is 10.5MiB

Note: numbers above are a result of guesswork. They are not 100% correct and never will be.
```

### Conclusion
TBD
