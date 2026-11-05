# Epic Specification: AWS EC2 Provider & Cloud Instance Provisioning (`ERR-219`..`ERR-226`)

## 1. Goal

Bring **AWS EC2** into Crow as a first-class, compiled-in Tier 1 `provider.hosts` plugin (`crow-provider-aws`) alongside **Linode** and **UpCloud**, and extend the `provider.hosts` contract with **direct instance provisioning (`ProvisionInstance`)** so operators can spin up fresh Ubuntu/Debian servers pre-seeded with a Crow Vault SSH key on **AWS EC2**, **Linode**, or **UpCloud** without leaving Crow.

Every AWS API call runs over `crow_provider_core::http::Http` (system `curl` with stdin config, zero SDK bloat, zero credentials in `argv`), signed with pure-Rust **AWS Signature Version 4 (`AWS4-HMAC-SHA256`)**, and is 100% traced and secret-redacted in the **Outbound Wire** ledger.

---

## 2. Ticket Breakdown (`ERR-219`..`ERR-226`)

- **`ERR-219` — `crow-provider-core`: `ProvisionInstance` trait & raw/form HTTP payloads**
  - Extend `HttpRequest` and `CurlHttp` to support form-encoded (`application/x-www-form-urlencoded`) payloads alongside JSON.
  - Add `ProvisionSpec`, `ProvisionCatalog`, `ProvisionOption`, and `ProvisionInstance` in `crates/crow-provider-core/src/hosts.rs`.
- **`ERR-220` — `crow-provider-aws`: Pure-Rust AWS SigV4 signer & EC2 XML client**
  - Create `crates/crow-provider-aws` with `AWS4-HMAC-SHA256` signing, `manifest.toml` (`access_key_id`, `secret_access_key`, `region`, `extra_regions`), and EC2 Query API XML decoding.
- **`ERR-221` — `crow-provider-aws`: `ListInstances`, `PowerControl`, and EBS `Snapshots`**
  - Implement multi-region `DescribeInstances`, `StartInstances`, `RebootInstances`, `StopInstances`, and root-volume EBS `DescribeSnapshots` / `CreateSnapshot`.
- **`ERR-222` — `ProvisionInstance` across AWS EC2, Linode, and UpCloud**
  - Implement `ProvisionInstance` for AWS EC2 (`ImportKeyPair` + Ubuntu/Debian AMI lookup + `RunInstances`), Linode (`POST /linode/instances`), and UpCloud (`POST /server`).
- **`ERR-223` — Built-in plugin wiring & AWS region tables**
  - Register `"aws"` in `src/plugins.rs`, `src/providers.rs`, `Cargo.toml`, and `src/region.rs`.
- **`ERR-224` — Provision Instance modal & Fleet onboarding handoff**
  - Add the **PROVISION INSTANCE** modal in Settings → Providers / Import Instances modal with Provider Account, Label, Region, Plan, OS Image, and Vault SSH Key selection.
- **`ERR-225` — Unit tests for AWS SigV4, EC2 XML, and provisioning**
  - Unit tests across `crow-provider-aws`, `crow-provider-linode`, `crow-provider-upcloud`, and `src/providers.rs`.
- **`ERR-226` — Localization (`en.ftl`, `es.ftl`, `ja.ftl`)**
  - Full English, Spanish, and Japanese translations verified by `i18n::guard` and `--check-language`.
