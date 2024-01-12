use crate::prelude::*;

/// The address of an Account, a bech32 encoding of a public key hash
/// that starts with the prefix `"account_"`, dependent on NetworkID, meaning the same
/// public key used for two AccountAddresses on two different networks will not have
/// the same address.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    derive_more::Display,
    PartialOrd,
    Ord,
    uniffi::Record,
)]
#[display("{address}")]
pub struct ResourceAddress {
    pub address: String,
    pub network_id: NetworkID,
}

impl Serialize for ResourceAddress {
    /// Serializes this `ResourceAddress` into its bech32 address string as JSON.
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.address)
    }
}

impl<'de> serde::Deserialize<'de> for ResourceAddress {
    /// Tries to deserializes a JSON string as a bech32 address into an `ResourceAddress`.
    #[cfg(not(tarpaulin_include))] // false negative
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<ResourceAddress, D::Error> {
        let s = String::deserialize(d)?;
        ResourceAddress::try_from_bech32(&s).map_err(de::Error::custom)
    }
}

#[uniffi::export]
pub fn new_resource_address(bech32: String) -> Result<ResourceAddress> {
    ResourceAddress::try_from_bech32(bech32.as_str())
}

impl EntityAddress for ResourceAddress {
    fn entity_type() -> AbstractEntityType {
        AbstractEntityType::Resource
    }

    // Underscored to decrease visibility. You SHOULD NOT call this function directly,
    // instead use `try_from_bech32` which performs proper validation. Impl types SHOULD
    // `panic` if `address` does not start with `Self::entity_type().hrp()`
    fn __with_address_and_network_id(address: &str, network_id: NetworkID) -> Self {
        assert!(address.starts_with(&Self::entity_type().hrp()), "Invalid address, you SHOULD NOT call this function directly, you should use `try_from_bech32` instead.");
        return Self {
            address: address.to_string(),
            network_id,
        };
    }
}

impl TryFrom<&str> for ResourceAddress {
    type Error = CommonError;

    fn try_from(value: &str) -> Result<Self> {
        ResourceAddress::try_from_bech32(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn display() {
        let s = "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd";
        let a = ResourceAddress::try_from_bech32(s).unwrap();
        assert_eq!(format!("{a}"), s);
    }

    #[test]
    fn json_roundtrip() {
        let a: ResourceAddress =
            "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"
                .try_into()
                .unwrap();

        assert_json_value_eq_after_roundtrip(
            &a,
            json!("resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"),
        );
        assert_json_roundtrip(&a);
        assert_json_value_ne_after_roundtrip(
            &a,
            json!("resource_rdx1tkk83magp3gjyxrpskfsqwkg4g949rmcjee4tu2xmw93ltw2cz94sq"),
        );
    }

    #[test]
    fn network_id_stokenet() {
        let a: ResourceAddress =
            "resource_tdx_2_1tkckx9fynl9f7756z8wxphq7wce6vk874nuq4f2nnxgh3nzrwhjdlp"
                .try_into()
                .unwrap();
        assert_eq!(a.network_id, NetworkID::Stokenet);
    }

    #[test]
    fn network_id_mainnet() {
        let a: ResourceAddress =
            "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"
                .try_into()
                .unwrap();
        assert_eq!(a.network_id, NetworkID::Mainnet);
    }
}

#[cfg(test)]
mod uniffi_tests {
    use crate::{new_resource_address, EntityAddress};

    use super::ResourceAddress;

    #[test]
    fn new() {
        let s = "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd";
        let a = ResourceAddress::try_from_bech32(s).unwrap();
        let b = new_resource_address(s.to_string()).unwrap();
        assert_eq!(b.address, s);
        assert_eq!(a, b);
    }
}
