# crow-provider-upcloud

UpCloud provider for [Crow](https://github.com/errorware/crow), API 1.3. Category `provider.hosts`.

- **instances.list**: `GET /server` plus `GET /ip_address`, two calls however many servers there are.
- **instances.power**: start, soft stop, soft restart (never forced).
- **No snapshots**: UpCloud backs up storage devices, not servers, so the capability isn't declared and Crow doesn't offer it.

Settings: an API token, or an API sub-account's username and password.
