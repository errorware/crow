//! Pure-Rust AWS Signature Version 4 (`AWS4-HMAC-SHA256`) request signer.
//!
//! Implements the AWS SigV4 specification without external SDKs:
//! - Canonical request construction
//! - String-to-sign derivation
//! - HMAC-SHA256 signing key derivation
//! - `Authorization` and `X-Amz-Date` header generation

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    let bytes = hmac_sha256(key, data);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Derive the AWS SigV4 4-tier signing key:
/// `kDate = HMAC("AWS4" + secret, date)`
/// `kRegion = HMAC(kDate, region)`
/// `kService = HMAC(kRegion, service)`
/// `kSigning = HMAC(kService, "aws4_request")`
pub fn derive_signing_key(secret_access_key: &str, date_stamp: &str, region: &str, service: &str) -> Vec<u8> {
    let k_secret = format!("AWS4{secret_access_key}");
    let k_date = hmac_sha256(k_secret.as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

pub struct SigV4Params<'a> {
    pub method: &'a str,
    pub uri_path: &'a str,
    pub query_string: &'a str,
    pub host: &'a str,
    pub region: &'a str,
    pub service: &'a str,
    pub access_key_id: &'a str,
    pub secret_access_key: &'a str,
    pub datetime: DateTime<Utc>,
    pub extra_headers: &'a [(&'a str, &'a str)],
    pub payload: &'a [u8],
}

pub struct SignedHeaders {
    pub authorization: String,
    pub x_amz_date: String,
    pub host: String,
}

/// Signs an HTTP request with AWS4-HMAC-SHA256.
pub fn sign_request(params: &SigV4Params) -> SignedHeaders {
    let datetime_str = params.datetime.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = params.datetime.format("%Y%m%d").to_string();

    // Canonical headers map: lowercase name -> trimmed value.
    // Use BTreeMap to ensure alphabetical sort order.
    let mut headers = BTreeMap::new();
    headers.insert("host".to_string(), params.host.trim().to_string());
    headers.insert("x-amz-date".to_string(), datetime_str.clone());

    for (name, val) in params.extra_headers {
        headers.insert(name.to_ascii_lowercase(), val.trim().to_string());
    }

    let mut canonical_headers = String::new();
    let mut signed_headers_list = Vec::new();

    for (name, val) in &headers {
        canonical_headers.push_str(name);
        canonical_headers.push(':');
        canonical_headers.push_str(val);
        canonical_headers.push('\n');
        signed_headers_list.push(name.as_str());
    }

    let signed_headers_str = signed_headers_list.join(";");
    let hashed_payload = sha256_hex(params.payload);

    let canonical_uri = if params.uri_path.is_empty() { "/" } else { params.uri_path };

    // 1. Canonical Request
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        params.method,
        canonical_uri,
        params.query_string,
        canonical_headers,
        signed_headers_str,
        hashed_payload
    );

    let hashed_canonical_request = sha256_hex(canonical_request.as_bytes());

    // 2. String to Sign
    let credential_scope = format!("{date_stamp}/{}/{}/aws4_request", params.region, params.service);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        datetime_str,
        credential_scope,
        hashed_canonical_request
    );

    // 3. Signature
    let signing_key = derive_signing_key(params.secret_access_key, &date_stamp, params.region, params.service);
    let signature = hmac_sha256_hex(&signing_key, string_to_sign.as_bytes());

    // 4. Authorization Header
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers_str}, Signature={signature}",
        params.access_key_id
    );

    SignedHeaders {
        authorization,
        x_amz_date: datetime_str,
        host: params.host.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_aws_sigv4_signing_key_derivation() {
        // Known AWS test vector:
        // Secret: wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY
        // Date: 20150830
        // Region: us-east-1
        // Service: iam
        let key = derive_signing_key(
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            "20150830",
            "us-east-1",
            "iam",
        );
        let key_hex = key.iter().map(|b| format!("{b:02x}")).collect::<String>();
        assert_eq!(
            key_hex,
            "c4afb1cc5771d871763a393e44b703571b55cc28424d1a5e86da6ed3c154a4b9"
        );
    }

    #[test]
    fn test_sign_request_structure() {
        let dt = Utc.with_ymd_and_hms(2026, 11, 5, 12, 0, 0).unwrap();
        let payload = b"Action=DescribeInstances&Version=2016-11-15";
        let headers = sign_request(&SigV4Params {
            method: "POST",
            uri_path: "/",
            query_string: "",
            host: "ec2.us-east-1.amazonaws.com",
            region: "us-east-1",
            service: "ec2",
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret_access_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            datetime: dt,
            extra_headers: &[("content-type", "application/x-www-form-urlencoded")],
            payload,
        });

        assert_eq!(headers.x_amz_date, "20261105T120000Z");
        assert_eq!(headers.host, "ec2.us-east-1.amazonaws.com");
        assert!(headers.authorization.starts_with("AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20261105/us-east-1/ec2/aws4_request"));
        assert!(headers.authorization.contains("SignedHeaders=content-type;host;x-amz-date"));
        assert!(headers.authorization.contains(", Signature="));
    }
}
