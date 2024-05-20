// SPDX-FileCopyrightText: The Ferrocene Developers
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::keys::algorithms::Algorithm;
use crate::keys::newtypes::{PayloadBytes, PrivateKeyBytes, PublicKeyBytes, SignatureBytes};
use crate::Error;
use elliptic_curve::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::{PublicKey, SecretKey};

pub(super) struct EcdsaP256Sha256Asn1SpkiDer;

impl Algorithm for EcdsaP256Sha256Asn1SpkiDer {
    fn sign(
        &self,
        private_key: &PrivateKeyBytes<'_>,
        payload: &PayloadBytes<'_>,
    ) -> Result<SignatureBytes<'static>, Error> {
        let key = SigningKey::from(
            SecretKey::from_pkcs8_der(private_key.as_bytes())
                .map_err(|e| Error::InvalidKey(e.to_string()))?,
        );

        let signature: Signature = key.sign(payload.as_bytes());
        Ok(SignatureBytes::owned(
            signature.to_der().to_bytes().to_vec(),
        ))
    }

    fn verify(
        &self,
        public_key: &PublicKeyBytes<'_>,
        payload: &PayloadBytes<'_>,
        signature: &SignatureBytes<'_>,
    ) -> Result<(), Error> {
        let key = VerifyingKey::from(
            PublicKey::from_public_key_der(public_key.as_bytes())
                .map_err(|e| Error::InvalidKey(e.to_string()))?,
        );

        let signature =
            Signature::from_der(signature.as_bytes()).map_err(|_| Error::VerificationFailed)?;
        key.verify(payload.as_bytes(), &signature)
            .map_err(|_| Error::VerificationFailed)
    }

    fn generate_private_key(&self) -> Result<PrivateKeyBytes<'static>, Error> {
        let key = SecretKey::random(&mut rand_core::OsRng);
        Ok(PrivateKeyBytes::owned(
            key.to_pkcs8_der()
                .expect("generated private key cannot be encoded")
                .to_bytes()
                .to_vec(),
        ))
    }

    fn derive_public_key_from_private_key(
        &self,
        private_key: &PrivateKeyBytes<'_>,
    ) -> Result<PublicKeyBytes<'static>, Error> {
        let key = SecretKey::from_pkcs8_der(private_key.as_bytes())
            .map_err(|e| Error::InvalidKey(e.to_string()))?;
        Ok(PublicKeyBytes::owned(
            key.public_key()
                .to_public_key_der()
                .map_err(|e| Error::InvalidKey(e.to_string()))?
                .to_vec(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{base64_decode, base64_encode};

    // Manually generated by invoking the methods.
    const PRIVATE_KEY: &str = "MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgneZv/FWLuK6glg1byqqauteGYe0VDoHmpK9jvt+yzuqhRANCAASi3D+Cfz/MWR26spM2VWBEmV+uhT5k9VdGFRIyuv1F6Rjjfma7EAWg+m3cU8L+BtYeYxx0hGmkQK591DUnLnnO";
    const PUBLIC_KEY: &str = "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEotw/gn8/zFkdurKTNlVgRJlfroU+ZPVXRhUSMrr9RekY435muxAFoPpt3FPC/gbWHmMcdIRppECufdQ1Jy55zg==";
    const SIGNATURE: &str = "MEQCIC6vI/lSbiusPBh7RSyw0N7en09FPXXF4AoP8va/j461AiAEfOAn6TSx6eAwxqm92luJZWu06R2JFVq1NA333s7njA==";
    const PLAINTEXT: PayloadBytes<'static> = PayloadBytes::borrowed(b"Hello world");

    #[test]
    fn test_generated_keys_are_not_equal() -> Result<(), Error> {
        let key_a = EcdsaP256Sha256Asn1SpkiDer.generate_private_key()?;
        let key_b = EcdsaP256Sha256Asn1SpkiDer.generate_private_key()?;

        assert_ne!(key_a, key_b);
        Ok(())
    }

    #[test]
    fn test_derive_public_key() -> Result<(), Error> {
        assert_eq!(
            PUBLIC_KEY,
            base64_encode(
                EcdsaP256Sha256Asn1SpkiDer
                    .derive_public_key_from_private_key(&PrivateKeyBytes::owned(b64(PRIVATE_KEY)))?
                    .as_bytes()
            ),
        );
        Ok(())
    }

    #[test]
    fn test_verify() {
        assert!(EcdsaP256Sha256Asn1SpkiDer
            .verify(
                &PublicKeyBytes::owned(b64(PUBLIC_KEY)),
                &PLAINTEXT,
                &SignatureBytes::owned(b64(SIGNATURE))
            )
            .is_ok());

        let mut broken_signature = b64(SIGNATURE);
        broken_signature[0] = broken_signature[0].wrapping_add(1);
        assert!(EcdsaP256Sha256Asn1SpkiDer
            .verify(
                &PublicKeyBytes::owned(b64(PUBLIC_KEY)),
                &PLAINTEXT,
                &SignatureBytes::owned(broken_signature)
            )
            .is_err());
    }

    #[test]
    fn test_sign() -> Result<(), Error> {
        let signature = EcdsaP256Sha256Asn1SpkiDer
            .sign(&PrivateKeyBytes::owned(b64(PRIVATE_KEY)), &PLAINTEXT)?;
        assert_eq!(SIGNATURE, base64_encode(signature.as_bytes()));

        Ok(())
    }

    fn b64(encoded: &str) -> Vec<u8> {
        base64_decode(encoded).unwrap()
    }
}
