# crow-provider-core

Contracts for [Crow](https://github.com/errorware/crow)'s provider plugins.

- **`hosts`**: the `provider.hosts` category (where servers come from): `ListInstances`, `PowerControl`, `Snapshots`.
- **`Provider`**: every provider implements it and hands out the category traits it supports. `check_declarations` makes sure the manifest declares exactly what the code implements, so a UI never offers an action that isn't there.
- **HTTP is injected**: providers build plain `HttpRequest`s. `CurlHttp` sends them through the system `curl` with the whole request, credentials included, on stdin. Tests use `testing::RecordedHttp` (feature `testing`).
- **Sync and data-shaped**: no async runtime, no HTTP-library types in the API, so providers could later run out of process.
- **Credentials are `SecretValue`s**: `Debug` of a request prints `[secret]`.
