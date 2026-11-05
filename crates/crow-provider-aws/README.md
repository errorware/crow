# crow-provider-aws

AWS EC2 provider plugin for Crow (`provider.hosts`).

Implements:
- `instances.list`: multi-region `DescribeInstances` with status, IP, and tag mapping.
- `instances.power`: `StartInstances`, `StopInstances`, `RebootInstances`.
- `instances.create`: SSH key import (`ImportKeyPair`), official AMI resolution, and `RunInstances`.
- `snapshots`: EBS volume snapshot management (`DescribeSnapshots`, `CreateSnapshot`).

All outbound requests use pure-Rust AWS Signature Version 4 (`AWS4-HMAC-SHA256`) and run over system `curl` via `crow-provider-core::http::Http`.
