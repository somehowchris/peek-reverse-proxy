# peek reverse proxy

Sometimes you do not have a network traffic interceptor, proxy such as burp or F12 debug tools to inspect requests. This simple reverse proxy is a simple solution to that.
> DISCLAIMER: do not use this as a production ready thing, was never designed for this

## Purpose

Sometimes there are is just that one environment where you can't have a debug mode.

If you've mocked an API just to look at the request sent fear no more. This client lets you host a http listener to peek into request details such as bodies, queries, headers and proxy the request to the destination at the same time.

## Usage

### Cargo

### From Source

## TODO
 - [ ] Docs
 - [ ] Readme
 - [ ] CI/CD
 - [ ] Publish