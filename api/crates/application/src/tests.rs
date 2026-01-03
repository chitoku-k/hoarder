#[cfg(feature = "tls")]
use openssl::{asn1::Asn1Time, hash::MessageDigest, nid::Nid, pkey::PKey, rsa::Rsa, x509::{X509, X509NameBuilder, extension::{BasicConstraints, SubjectAlternativeName}}};
#[cfg(feature = "tls")]
use tempfile::NamedTempFile;
#[cfg(feature = "tls")]
use tokio::{fs::File, io::AsyncWriteExt};

mod mocks;

mod server;

#[cfg(feature = "tls")]
async fn generate_certificate() -> (Vec<u8>, NamedTempFile, NamedTempFile) {
    let mut ca_name_builder = X509NameBuilder::new().unwrap();
    ca_name_builder.append_entry_by_nid(Nid::COMMONNAME, "Root CA").unwrap();

    let ca_name = ca_name_builder.build();
    let ca_rsa = Rsa::generate(2048).unwrap();
    let ca_pkey = PKey::from_rsa(ca_rsa).unwrap();

    let mut ca_builder = X509::builder().unwrap();
    ca_builder.set_version(2).unwrap();
    ca_builder.set_subject_name(&ca_name).unwrap();
    ca_builder.set_issuer_name(&ca_name).unwrap();
    ca_builder.set_pubkey(&ca_pkey).unwrap();
    ca_builder.set_not_before(&Asn1Time::days_from_now(0).unwrap()).unwrap();
    ca_builder.set_not_after(&Asn1Time::days_from_now(365).unwrap()).unwrap();
    ca_builder.append_extension(BasicConstraints::new().critical().ca().build().unwrap()).unwrap();
    ca_builder.sign(&ca_pkey, MessageDigest::sha256()).unwrap();

    let mut leaf_name_builder = X509NameBuilder::new().unwrap();
    leaf_name_builder.append_entry_by_nid(Nid::COMMONNAME, "localhost").unwrap();

    let leaf_rsa = Rsa::generate(2048).unwrap();
    let leaf_pkey = PKey::from_rsa(leaf_rsa).unwrap();
    let leaf_name = leaf_name_builder.build();

    let mut leaf_builder = X509::builder().unwrap();
    leaf_builder.set_version(2).unwrap();
    leaf_builder.set_subject_name(&leaf_name).unwrap();
    leaf_builder.set_issuer_name(&ca_name).unwrap();
    leaf_builder.set_pubkey(&leaf_pkey).unwrap();
    leaf_builder.set_not_before(&Asn1Time::days_from_now(0).unwrap()).unwrap();
    leaf_builder.set_not_after(&Asn1Time::days_from_now(365).unwrap()).unwrap();
    leaf_builder.append_extension(SubjectAlternativeName::new().ip("127.0.0.1").ip("::1").build(&leaf_builder.x509v3_context(None, None)).unwrap()).unwrap();
    leaf_builder.sign(&ca_pkey, MessageDigest::sha256()).unwrap();

    let ca = ca_builder.build().to_pem().unwrap();
    let cert = leaf_builder.build().to_pem().unwrap();
    let key = leaf_pkey.private_key_to_pem_pkcs8().unwrap();

    let cert_file = NamedTempFile::new().unwrap();
    File::from_std(cert_file.reopen().unwrap()).write_all(&cert).await.unwrap();

    let key_file = NamedTempFile::new().unwrap();
    File::from_std(key_file.reopen().unwrap()).write_all(&key).await.unwrap();

    (ca, cert_file, key_file)
}
