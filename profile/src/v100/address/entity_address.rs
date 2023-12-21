use hd::cap26::cap26_path::paths::is_entity_path::IsEntityPath;
use radix_engine_common::crypto::PublicKey as EnginePublicKey;
use radix_engine_toolkit::functions::derive::{
    virtual_account_address_from_public_key, virtual_identity_address_from_public_key,
};
use radix_engine_toolkit_json::models::scrypto::node_id::SerializableNodeIdInternal;
use wallet_kit_common::network_id::NetworkID;

use crate::v100::{
    entity::abstract_entity_type::AbstractEntityType,
    factors::hd_transaction_signing_factor_instance::HDFactorInstanceTransactionSigning,
};
use wallet_kit_common::error::common_error::CommonError as Error;

use super::decode_address_helper::decode_address;

/// An address of an entity, provides default implementation of `try_from_bech32`
/// to decode a bech32 encoded address string into Self.
pub trait EntityAddress: Sized {
    fn entity_type() -> AbstractEntityType;

    // Underscored to decrease visibility. You SHOULD NOT call this function directly,
    // instead use `try_from_bech32` which performs proper validation. Impl types SHOULD
    // `panic` if `address` does not start with `Self::entity_type().hrp()`
    fn __with_address_and_network_id(address: &str, network_id: NetworkID) -> Self;

    /// Creates a new address from `public_key` and `network_id` by bech32 encoding
    /// it.
    #[cfg(not(tarpaulin_include))] // false negative
    fn from_public_key<P>(public_key: P, network_id: NetworkID) -> Self
    where
        P: Into<EnginePublicKey> + Clone,
    {
        let component = match Self::entity_type() {
            AbstractEntityType::Account => virtual_account_address_from_public_key(&public_key),
            AbstractEntityType::Identity => virtual_identity_address_from_public_key(&public_key),
            AbstractEntityType::Resource => panic!("resource"),
        };

        let node = SerializableNodeIdInternal {
            network_id: network_id.discriminant(),
            node_id: component.into_node_id(),
        };

        let address = format!("{node}");
        return Self::__with_address_and_network_id(&address, network_id);
    }

    fn from_hd_factor_instance_virtual_entity_creation<E: IsEntityPath>(
        hd_factor_instance_virtual_entity_creation: HDFactorInstanceTransactionSigning<E>,
    ) -> Self {
        Self::from_public_key(
            hd_factor_instance_virtual_entity_creation
                .public_key()
                .public_key()
                .clone(),
            hd_factor_instance_virtual_entity_creation
                .path()
                .network_id(),
        )
    }

    fn try_from_bech32(s: &str) -> Result<Self, Error> {
        let (network_id, entity_type, hrp, _) = decode_address(s)?;
        if entity_type != Self::entity_type() {
            return Err(Error::MismatchingEntityTypeWhileDecodingAddress);
        }

        if !hrp.starts_with(&entity_type.hrp()) {
            return Err(Error::MismatchingHRPWhileDecodingAddress);
        }

        return Ok(Self::__with_address_and_network_id(s, network_id));
    }
}
