# peek reverse proxy

Sometimes you do not have a network traffic interceptor, proxy such as burp or F12 debug tools to inspect requests. This simple reverse proxy is a simple solution to that.
> DISCLAIMER: do not use this as a production ready thing, was never designed for this

## Purpose

Sometimes there are is just that one environment where you can't have a debug mode.

If you've mocked an API just to look at the request sent fear no more. This client lets you host a http listener to peek into request details such as bodies, queries, headers and proxy the request to the destination at the same time.

## Usage

### Install

You can use this crate via several distributions:
 - `cargo install` via crates.io
 - `cargo install` from source
 - `docker`, `podman` or any OCI container runtime
 - `cargo binstall`
 - binaries from gh releases

#### Cargo

To install this crate via `cargo` do the following:
```sh
cargo install peek-reverse-proxy
```

#### From Source

```
git clone https://github.com/somehowchris/peek-reverse-proxy.git

cd peek-reverse-proxy

cargo install --path .
```

### Run it

Once installed, you can run it via:
```sh
peek-reverse-proxy
```

#### Configuration

Env variables allow you to configure things to your needs:
- `HOST_ADDRESS`: address on which to listen on i.e. `0.0.0.0:8080`
- `DESTINATION_URL`: destination url including host and scheme i.e. `https://www.google.com`
- (optional) `LOG_LEVEL`: level of logs to log __off__, __debug__, __normal__, __critical__, defaults to `normal`
- (optional) `PRINT_STYLE`: print style for logs, either __json__ (outputs everything in json style), __plain__ (outputs everything in standard log formats but has no json field formatting), __pretty__ (just as plain, but formats outputs of json fields with serde_jsons pretty option), defaults to `pretty`


For example:

```sh
export HOST_ADDRESS="0.0.0.0:8080"
export DESTINATION_URL="https://www.google.com"
export PRINT_STYLE="json"
export LOG_LEVEL="normal"

peek-reverse-proxy
```


