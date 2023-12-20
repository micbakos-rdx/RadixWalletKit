use identified_vec::{
    Identifiable, IdentifiedVecOf, IsIdentifiableVecOfVia, IsIdentifiedVec, IsIdentifiedVecOf,
};
use wallet_kit_common::error::common_error::CommonError;

use crate::{
    identified_vec_via::IdentifiedVecVia,
    v100::factors::{
        factor_source::FactorSource, factor_source_id::FactorSourceID,
        is_factor_source::IsFactorSource,
    },
};

impl Identifiable for FactorSource {
    type ID = FactorSourceID;

    fn id(&self) -> Self::ID {
        self.factor_source_id()
    }
}

/// A collection of FactorSources generated by a wallet or manually added by user.
/// MUST never be empty.
pub type FactorSources = IdentifiedVecVia<FactorSource>;

impl FactorSources {
    pub fn try_from_iter<I>(into_iter: I) -> Result<Self, CommonError>
    where
        I: IntoIterator<Item = FactorSource>,
    {
        let vec = IdentifiedVecOf::from_iter(into_iter);
        if vec.len() == 0 {
            return Err(CommonError::FactorSourcesMustNotBeEmpty);
        }

        Ok(Self::from_identified_vec_of(vec))
    }

    /// Panics if this `FactorSources` is empty.
    pub fn assert_not_empty(&self) {
        assert_ne!(
            self.len(),
            0,
            "FactorSources empty, which must never happen."
        )
    }
}

#[cfg(any(test, feature = "placeholder"))]
impl FactorSources {
    pub fn placeholder() -> Self {
        Self::try_from_iter([
            FactorSource::placeholder_device(),
            FactorSource::placeholder_ledger(),
        ])
        .unwrap()
    }

    pub fn placeholder_other() -> Self {
        Self::try_from_iter([
            FactorSource::placeholder_device_olympia(),
            FactorSource::placeholder_device_babylon(),
        ])
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use identified_vec::Identifiable;
    use wallet_kit_common::{
        error::common_error::CommonError, json::assert_eq_after_json_roundtrip,
    };

    use crate::v100::factors::{factor_source::FactorSource, is_factor_source::IsFactorSource};

    use super::FactorSources;

    #[test]
    fn identifiable_id_uses_factor_source_id() {
        assert_eq!(
            FactorSource::placeholder_device().id(),
            FactorSource::placeholder_device().factor_source_id()
        )
    }

    #[test]
    fn inequality() {
        assert_ne!(
            FactorSources::placeholder(),
            FactorSources::placeholder_other()
        );
    }

    #[test]
    fn err_when_try_from_iter_used_with_empty() {
        assert_eq!(
            FactorSources::try_from_iter([]),
            Err(CommonError::FactorSourcesMustNotBeEmpty)
        );
    }

    #[test]
    fn try_from_iter_ok_when_non_empty() {
        assert!(FactorSources::try_from_iter([FactorSource::placeholder_device()]).is_ok());
    }

    #[test]
    fn json_roundtrip_placeholder() {
        let sut = FactorSources::placeholder();
        assert_eq_after_json_roundtrip(
            &sut,
            r#"
            [
                {
                    "discriminator": "device",
                    "device": {
                        "id": {
                            "kind": "device",
                            "body": "3c986ebf9dcd9167a97036d3b2c997433e85e6cc4e4422ad89269dac7bfea240"
                        },
                        "common": {
                            "flags": ["main"],
                            "addedOn": "2023-09-11T16:05:56.000Z",
                            "cryptoParameters": {
                                "supportedCurves": ["curve25519"],
                                "supportedDerivationPathSchemes": ["cap26"]
                            },
                            "lastUsedOn": "2023-09-11T16:05:56.000Z"
                        },
                        "hint": {
                            "name": "Unknown Name",
                            "model": "iPhone",
                            "mnemonicWordCount": 24
                        }
                    }
                },
                {
                    "discriminator": "ledgerHQHardwareWallet",
                    "ledgerHQHardwareWallet": {
                        "id": {
                            "kind": "ledgerHQHardwareWallet",
                            "body": "3c986ebf9dcd9167a97036d3b2c997433e85e6cc4e4422ad89269dac7bfea240"
                        },
                        "common": {
                            "addedOn": "2023-09-11T16:05:56.000Z",
                            "cryptoParameters": {
                                "supportedCurves": ["curve25519"],
                                "supportedDerivationPathSchemes": ["cap26"]
                            },
                            "flags": ["main"],
                            "lastUsedOn": "2023-09-11T16:05:56.000Z"
                        },
                        "hint": {
                            "name": "Orange, scratched",
                            "model": "nanoS+"
                        }
                    }
                }
            ]
            "#,
        )
    }
}
