//! The server's certificate and key, made once and kept in its data folder.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use rcgen::{CertificateParams, KeyPair, date_time_ymd};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

/// Name the certificate is made for. Players ask for it in the handshake.
pub const SERVER_NAME: &str = "abstractions";

/// File of the certificate in the data folder.
const CERT_FILE: &str = "cert.der";

/// File of the private key in the data folder.
const KEY_FILE: &str = "key.der";

/// The certificate a server shows and the key that proves it is its own.
#[derive(Debug)]
pub struct Identity {
    /// The self-signed certificate, in DER form.
    cert: CertificateDer<'static>,
    /// The private key, in PKCS #8 DER form.
    key: PrivatePkcs8KeyDer<'static>,
}

impl Identity {
    /// Loads `dir/cert.der` and `dir/key.der`, or makes and saves both when neither exists.
    ///
    /// Existing files are never replaced: a new certificate would make every player refuse
    /// the server. When only one of the two files exists, this fails instead.
    pub fn load_or_create(dir: &Path) -> io::Result<Self> {
        let cert_path = dir.join(CERT_FILE);
        let key_path = dir.join(KEY_FILE);
        match (read_optional(&cert_path)?, read_optional(&key_path)?) {
            (Some(cert), Some(key)) => Ok(Self {
                cert: cert.into(),
                key: key.into(),
            }),
            (None, None) => {
                let made = Self::create()?;
                fs::create_dir_all(dir)?;
                write_private(&key_path, made.key.secret_pkcs8_der())?;
                write_private(&cert_path, &made.cert)?;
                Ok(made)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "only one of {CERT_FILE} and {KEY_FILE} exists in {}",
                    dir.display()
                ),
            )),
        }
    }

    /// Makes a new key and a self-signed certificate for [`SERVER_NAME`] that never expires.
    fn create() -> io::Result<Self> {
        let pair = KeyPair::generate().map_err(io::Error::other)?;
        let mut params =
            CertificateParams::new(vec![SERVER_NAME.to_string()]).map_err(io::Error::other)?;
        params.not_before = date_time_ymd(2026, 1, 1);
        params.not_after = date_time_ymd(9999, 12, 31);
        let cert = params.self_signed(&pair).map_err(io::Error::other)?;
        Ok(Self {
            cert: cert.der().clone(),
            key: pair.serialize_der().into(),
        })
    }

    /// Returns the certificate, in DER form.
    pub fn cert(&self) -> &CertificateDer<'static> {
        &self.cert
    }

    /// Returns a copy of the private key, for the TLS configuration.
    pub fn key(&self) -> PrivateKeyDer<'static> {
        PrivateKeyDer::Pkcs8(self.key.clone_key())
    }

    /// Returns the fingerprint of the certificate.
    pub fn fingerprint(&self) -> [u8; 32] {
        super::fingerprint(&self.cert)
    }
}

/// Reads a whole file, or returns nothing when it does not exist.
fn read_optional(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Writes `bytes` to `path` through a temporary file, readable by the owner only on unix.
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("tmp");
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)
}
