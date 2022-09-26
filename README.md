# Kraken Binary

The Kraken system is an asynchronous webserver with scanning functionality written in Rust. Its purpose is to get different informations about a website.

## Compiling
### Preparations
* Install Rust: https://www.rust-lang.org/
* OpenSSL 1.0.1, 1.0.2, 1.1.0, or 1.1.1 with headers (see https://github.com/sfackler/rust-openssl) 

The system has been testet on Linux but it should run on other *NIX systems as well.

### Release Build
```sh
cargo build --release
```

## Config

Kraken needs an ini Config with the name `kraken.ini` in the same directory from which you run the kraken binary.

```sh
[server]
ip = 127.0.0.1
port = 8080
web-path = ./public
[requester]
url = http://data.php
user = extension
password = (redacted)
[scheduler]
update-rate = 600
[database]
customer-data = mysql://admin:[passwort]@localhost/database
```