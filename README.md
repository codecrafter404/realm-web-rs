# realm-web-rs
Implementation of the realm-web [npm package](https://www.npmjs.com/package/realm-web) in rust.
> Accessing Atlas App Services from a web-browser.

This crate may be useful to access a mongodb database through the Atlas Data Service Api in an ``wasm32-unknown-unknown target``.
## Caveats / limitations
- This crate doesn't implement the Realm Sync feature
- this client is implemented based on the [offical documentation](https://www.mongodb.com/docs/atlas/app-services/data-api/generated-endpoints/)