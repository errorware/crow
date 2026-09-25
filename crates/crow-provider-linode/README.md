# crow-provider-linode

Linode (Akamai Cloud) provider for [Crow](https://github.com/errorware/crow), API v4. Category `provider.hosts`.

- **instances.list**: every page of `GET /linode/instances`: label, status, public addresses first, region, plan, tags.
- **instances.power**: boot, reboot, shut down.
- **snapshots**: Linode's manual backup snapshot. It needs the Backup service on the instance, and a new snapshot replaces the previous one.

Setting: `api_token`, a personal access token. *Linodes: read only* is enough to list; power and snapshots need *read/write*.

Live check (read-only): `CROW_LINODE_TOKEN=… cargo test -p crow-provider-linode -- --ignored --nocapture`.
