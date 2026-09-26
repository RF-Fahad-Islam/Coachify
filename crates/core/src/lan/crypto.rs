use super::protocol::{get_local_ip_addresses, get_system_hostname, LanError};
use rcgen::{CertificateParams, KeyPair};
use rustls::client::danger::ServerCertVerifier;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::DigitallySignedStruct;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex, Once};

static INIT_CRYPTO: Once = Once::new();

pub fn ensure_crypto_provider_installed() {
    INIT_CRYPTO.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

pub fn compute_sha256_fingerprint(der_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(der_bytes);
    let hash = hasher.finalize();
    let hex_pairs: Vec<String> = hash.iter().map(|b| format!("{:02X}", b)).collect();
    format!("SHA256:{}", hex_pairs.join(":"))
}

pub fn generate_self_signed_cert() -> Result<(String, String, String), LanError> {
    ensure_crypto_provider_installed();

    let mut sans = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];

    let hostname = get_system_hostname();
    if !sans.contains(&hostname) {
        sans.push(hostname.clone());
    }

    for ip in get_local_ip_addresses() {
        if !sans.contains(&ip) {
            sans.push(ip);
        }
    }

    let mut params =
        CertificateParams::new(sans).map_err(|e| LanError::CertGenerationFailed(e.to_string()))?;

    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        format!("Coachify - {}", hostname),
    );

    let key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
        .map_err(|e| LanError::CertGenerationFailed(e.to_string()))?;

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| LanError::CertGenerationFailed(e.to_string()))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    let fingerprint = compute_sha256_fingerprint(cert.der());

    Ok((cert_pem, key_pem, fingerprint))
}

#[derive(Debug)]
pub struct FingerprintPinnedCertVerifier {
    pub expected_fingerprint_hex: String,
}

impl FingerprintPinnedCertVerifier {
    pub fn new(expected_fingerprint_hex: &str) -> Self {
        let cleaned = expected_fingerprint_hex
            .trim()
            .trim_start_matches("SHA256:")
            .trim_start_matches("sha256:")
            .trim_start_matches("SHA256")
            .trim_start_matches("sha256")
            .replace(':', "")
            .to_lowercase();
        Self {
            expected_fingerprint_hex: cleaned,
        }
    }
}

impl ServerCertVerifier for FingerprintPinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let mut hasher = Sha256::new();
        hasher.update(end_entity.as_ref());
        let cert_hash = hex::encode(hasher.finalize()).to_lowercase();

        if cert_hash == self.expected_fingerprint_hex {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[derive(Debug)]
pub struct PinnedCertVerifier {
    pub captured_der: Arc<Mutex<Option<Vec<u8>>>>,
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let mut lock = self.captured_der.lock().unwrap();
        *lock = Some(end_entity.as_ref().to_vec());
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
