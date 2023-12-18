use hierarchical_deterministic::{
    bip32::hd_path_component::HDPathValue,
    cap26::cap26_path::paths::is_entity_path::HasEntityPath,
    derivation::{derivation::Derivation, mnemonic_with_passphrase::MnemonicWithPassphrase},
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, cmp::Ordering, fmt::Display};
use wallet_kit_common::network_id::NetworkID;

use crate::v100::{
    address::{account_address::AccountAddress, entity_address::EntityAddress},
    entity::{display_name::DisplayName, entity_flags::EntityFlags},
    entity_security_state::{
        entity_security_state::EntitySecurityState,
        unsecured_entity_control::UnsecuredEntityControl,
    },
    factors::{
        factor_sources::{
            device_factor_source::device_factor_source::DeviceFactorSource,
            private_hierarchical_deterministic_factor_source::PrivateHierarchicalDeterministicFactorSource,
        },
        hd_transaction_signing_factor_instance::HDFactorInstanceAccountCreation,
    },
};

use super::{
    appearance_id::AppearanceID, on_ledger_settings::on_ledger_settings::OnLedgerSettings,
};

/// A network unique account with a unique public address and a set of cryptographic
/// factors used to control it.
///
/// Used to own and control assets on the radix network. Uniquely identified by an
/// account address, e.g.
///
/// `account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease`
///
/// But most commonly users see the address on its abbreviated form:
///
/// `acco...please`
///
/// Accounts have a display name and an appearance id.
///
/// An account can be either controlled by a "Babylon" DeviceFactorSource or a
/// Legacy one imported from Olympia, or a Ledger hardware wallet, which too might
/// have been imported from Olympia.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// The ID of the network this account can be used with.
    #[serde(rename = "networkID")]
    network_id: NetworkID,

    /// A globally unique identifier of this account, being a human readable
    /// address of an account. Always starts with `"account_"``, for example:
    ///
    /// `account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease`
    ///
    /// Most commonly the user will see this address in its abbreviated
    /// form which is:
    ///
    /// `acco...please`
    ///
    /// No two addresses will ever be the same even for the same factor source
    /// but on different networks, since the public keys controlling the
    /// accounts depend on the network id.
    address: AccountAddress,

    /// An off-ledger display name or description chosen by the user when she
    /// created this account.
    display_name: RefCell<DisplayName>,

    /// Security state of this account, either "securified" or not.
    security_state: EntitySecurityState,

    /// The visual cue user learns to associated this account with, typically
    /// a beautiful colorful gradient.
    #[serde(rename = "appearanceID")]
    appearance_id: RefCell<AppearanceID>,

    /// An order set of `EntityFlag`s used to describe certain Off-ledger
    /// user state about Accounts or Personas, such as if an entity is
    /// marked as hidden or not.
    flags: RefCell<EntityFlags>,

    /// The on ledger synced settings for this account, contains e.g.
    /// ThirdPartyDeposit settings, with deposit rules for assets.
    on_ledger_settings: RefCell<OnLedgerSettings>,
}

impl Account {
    pub fn new(
        account_creating_factor_instance: HDFactorInstanceAccountCreation,
        display_name: DisplayName,
        appearance_id: AppearanceID,
    ) -> Self {
        let address = AccountAddress::from_hd_factor_instance_virtual_entity_creation(
            account_creating_factor_instance.clone(),
        );
        Self {
            network_id: account_creating_factor_instance.network_id(),
            address,
            display_name: RefCell::new(display_name),
            security_state: UnsecuredEntityControl::with_account_creating_factor_instance(
                account_creating_factor_instance,
            )
            .into(),
            appearance_id: RefCell::new(appearance_id),
            flags: RefCell::new(EntityFlags::default()),
            on_ledger_settings: RefCell::new(OnLedgerSettings::default()),
        }
    }
}

// Getters
impl Account {
    pub fn network_id(&self) -> NetworkID {
        self.network_id.clone()
    }

    pub fn address(&self) -> AccountAddress {
        self.address.clone()
    }

    /// Returns this accounts `display_name` as **a clone**.
    ///
    /// Use [`self::set_display_name()`] to update it.
    pub fn display_name(&self) -> String {
        self.display_name.borrow().clone().to_string()
    }

    pub fn flags(&self) -> EntityFlags {
        self.flags.borrow().clone()
    }

    pub fn appearance_id(&self) -> AppearanceID {
        self.appearance_id.borrow().clone()
    }

    pub fn on_ledger_settings(&self) -> OnLedgerSettings {
        self.on_ledger_settings.borrow().clone()
    }
}

// Setters
impl Account {
    pub fn set_display_name(&self, new: DisplayName) {
        *self.display_name.borrow_mut() = new;
    }

    pub fn set_flags(&self, new: EntityFlags) {
        *self.flags.borrow_mut() = new;
    }

    pub fn set_appearance_id(&self, new: AppearanceID) {
        *self.appearance_id.borrow_mut() = new;
    }

    pub fn set_on_ledger_settings(&self, new: OnLedgerSettings) {
        *self.on_ledger_settings.borrow_mut() = new;
    }

    pub fn update_on_ledger_settings<F>(&self, update: F)
    where
        F: Fn(&mut OnLedgerSettings) -> (),
    {
        update(&mut self.on_ledger_settings.borrow_mut())
    }
}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.security_state, &other.security_state) {
            (EntitySecurityState::Unsecured(l), EntitySecurityState::Unsecured(r)) => l
                .transaction_signing
                .derivation_path()
                .last_component()
                .cmp(r.transaction_signing.derivation_path().last_component()),
        }
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.display_name(), self.address)
    }
}

impl Account {
    /// Instantiates an account with a display name, address and appearance id.
    pub fn placeholder_with_values(
        address: AccountAddress,
        display_name: DisplayName,
        appearance_id: AppearanceID,
    ) -> Self {
        Self {
            network_id: address.network_id,
            address,
            display_name: RefCell::new(display_name),
            appearance_id: RefCell::new(appearance_id),
            flags: RefCell::new(EntityFlags::default()),
            on_ledger_settings: RefCell::new(OnLedgerSettings::default()),
            security_state: EntitySecurityState::placeholder(),
        }
    }

    fn placeholder_at_index_name(index: HDPathValue, name: &str) -> Self {
        let mwp = MnemonicWithPassphrase::placeholder();
        let bdfs = DeviceFactorSource::babylon(true, mwp.clone(), "iPhone");
        let private_hd_factor_source = PrivateHierarchicalDeterministicFactorSource::new(mwp, bdfs);
        let account_creating_factor_instance = private_hd_factor_source
            .derive_account_creation_factor_instance(NetworkID::Mainnet, index);

        Self::new(
            account_creating_factor_instance,
            DisplayName::new(name).unwrap(),
            AppearanceID::try_from(index as u8).unwrap(),
        )
    }

    /// A `Mainnet` account named "Alice", a placeholder used to facilitate unit tests, with
    /// derivation index 0,
    pub fn placeholder_alice() -> Self {
        Self::placeholder_at_index_name(0, "Alice")
    }

    /// A `Mainnet` account named "Bob", a placeholder used to facilitate unit tests, with
    /// derivation index 1.
    pub fn placeholder_bob() -> Self {
        Self::placeholder_at_index_name(1, "Bob")
    }
}

// CFG test
#[cfg(test)]
impl Account {
    /// A placeholder used to facilitate unit tests.
    pub fn placeholder() -> Self {
        Self::placeholder_mainnet()
    }

    pub fn placeholder_mainnet() -> Self {
        Self::placeholder_with_values(
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
                .try_into()
                .unwrap(),
            DisplayName::default(),
            AppearanceID::default(),
        )
    }

    pub fn placeholder_stokenet() -> Self {
        Self::placeholder_with_values(
            "account_tdx_2_12ygsf87pma439ezvdyervjfq2nhqme6reau6kcxf6jtaysaxl7sqvd"
                .try_into()
                .unwrap(),
            DisplayName::default(),
            AppearanceID::default(),
        )
    }

    pub fn placeholder_nebunet() -> Self {
        Self::placeholder_with_values(
            "account_tdx_b_1p8ahenyznrqy2w0tyg00r82rwuxys6z8kmrhh37c7maqpydx7p"
                .try_into()
                .unwrap(),
            DisplayName::default(),
            AppearanceID::default(),
        )
    }

    pub fn placeholder_kisharnet() -> Self {
        Self::placeholder_with_values(
            "account_tdx_c_1px26p5tyqq65809em2h4yjczxcxj776kaun6sv3dw66sc3wrm6"
                .try_into()
                .unwrap(),
            DisplayName::default(),
            AppearanceID::default(),
        )
    }

    pub fn placeholder_adapanet() -> Self {
        Self::placeholder_with_values(
            "account_tdx_a_1qwv0unmwmxschqj8sntg6n9eejkrr6yr6fa4ekxazdzqhm6wy5"
                .try_into()
                .unwrap(),
            DisplayName::default(),
            AppearanceID::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use wallet_kit_common::json::assert_eq_after_json_roundtrip;

    use crate::v100::{
        address::account_address::AccountAddress,
        entity::{
            account::{
                appearance_id::AppearanceID,
                on_ledger_settings::{
                    on_ledger_settings::OnLedgerSettings,
                    third_party_deposits::{
                        asset_exception::AssetException,
                        deposit_address_exception_rule::DepositAddressExceptionRule,
                        deposit_rule::DepositRule, depositor_address::DepositorAddress,
                        third_party_deposits::ThirdPartyDeposits,
                    },
                },
            },
            display_name::DisplayName,
            entity_flag::EntityFlag,
            entity_flags::EntityFlags,
        },
    };

    use super::Account;

    #[test]
    fn new_with_address_only() {
        let address: AccountAddress =
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
                .try_into()
                .unwrap();
        let account = Account::placeholder_with_values(
            address.clone(),
            DisplayName::default(),
            AppearanceID::default(),
        );
        assert_eq!(account.address, address);
    }

    #[test]
    fn appearance_id_get_set() {
        let account = Account::placeholder();
        assert_eq!(account.appearance_id(), AppearanceID::default());
        let new_appearance_id = AppearanceID::new(1).unwrap();
        account.set_appearance_id(new_appearance_id);
        assert_eq!(account.appearance_id(), new_appearance_id);
    }

    #[test]
    fn display() {
        let account = Account::placeholder();
        assert_eq!(
            format!("{account}"),
            "Unnamed | account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
        );
    }

    #[test]
    fn compare() {
        assert!(Account::placeholder_alice() < Account::placeholder_bob());
    }

    #[test]
    fn display_name_get_set() {
        let account = Account::placeholder_with_values(
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
                .try_into()
                .unwrap(),
            DisplayName::new("Test").unwrap(),
            AppearanceID::default(),
        );
        assert_eq!(account.display_name(), "Test");
        let new_display_name = DisplayName::new("New").unwrap();
        account.set_display_name(new_display_name.clone());
        assert_eq!(account.display_name(), new_display_name.to_string());
    }

    #[test]
    fn flags_get_set() {
        let account = Account::placeholder_with_values(
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
                .try_into()
                .unwrap(),
            DisplayName::new("Test").unwrap(),
            AppearanceID::default(),
        );
        assert_eq!(account.flags(), EntityFlags::default());
        let new_flags = EntityFlags::with_flag(EntityFlag::DeletedByUser);
        account.set_flags(new_flags.clone());
        assert_eq!(account.flags(), new_flags);
    }

    #[test]
    fn on_ledger_settings_get_set() {
        let account = Account::placeholder_with_values(
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
                .try_into()
                .unwrap(),
            DisplayName::new("Test").unwrap(),
            AppearanceID::default(),
        );
        assert_eq!(account.on_ledger_settings(), OnLedgerSettings::default());
        let excp1 = AssetException::new(
            "resource_rdx1tkk83magp3gjyxrpskfsqwkg4g949rmcjee4tu2xmw93ltw2cz94sq"
                .try_into()
                .unwrap(),
            DepositAddressExceptionRule::Allow,
        );
        let excp2 = AssetException::new(
            "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"
                .try_into()
                .unwrap(),
            DepositAddressExceptionRule::Allow,
        );
        let new_third_party_dep = ThirdPartyDeposits::with_rule_and_lists(
            DepositRule::DenyAll,
            BTreeSet::from_iter([excp1, excp2].into_iter()),
            BTreeSet::from_iter(
                [DepositorAddress::ResourceAddress(
                    "resource_rdx1tkk83magp3gjyxrpskfsqwkg4g949rmcjee4tu2xmw93ltw2cz94sq"
                        .try_into()
                        .unwrap(),
                )]
                .into_iter(),
            ),
        );
        let new_on_ledger_settings = OnLedgerSettings::new(new_third_party_dep);
        account.set_on_ledger_settings(new_on_ledger_settings.clone());
        assert_eq!(account.on_ledger_settings(), new_on_ledger_settings);

        assert_eq!(
            account
                .on_ledger_settings()
                .third_party_deposits()
                .deposit_rule(),
            DepositRule::DenyAll
        );
        account.update_on_ledger_settings(|o| {
            o.update_third_party_deposits(|t| t.set_deposit_rule(DepositRule::AcceptAll))
        });
        assert_eq!(
            account
                .on_ledger_settings()
                .third_party_deposits()
                .deposit_rule(),
            DepositRule::AcceptAll
        );
    }

    #[test]
    fn json_roundtrip_alice() {
        let model = Account::placeholder_alice();
        assert_eq_after_json_roundtrip(
            &model,
            r#"
            {
				"securityState": {
					"unsecuredEntityControl": {
						"transactionSigning": {
							"badge": {
								"virtualSource": {
									"hierarchicalDeterministicPublicKey": {
										"publicKey": {
											"curve": "curve25519",
											"compressedData": "d24cc6af91c3f103d7f46e5691ce2af9fea7d90cfb89a89d5bba4b513b34be3b"
										},
										"derivationPath": {
											"scheme": "cap26",
											"path": "m/44H/1022H/1H/525H/1460H/0H"
										}
									},
									"discriminator": "hierarchicalDeterministicPublicKey"
								},
								"discriminator": "virtualSource"
							},
							"factorSourceID": {
								"fromHash": {
									"kind": "device",
									"body": "3c986ebf9dcd9167a97036d3b2c997433e85e6cc4e4422ad89269dac7bfea240"
								},
								"discriminator": "fromHash"
							}
						}
					},
					"discriminator": "unsecured"
				},
				"networkID": 1,
				"appearanceID": 0,
				"flags": [],
				"displayName": "Alice",
				"onLedgerSettings": {
					"thirdPartyDeposits": {
						"depositRule": "acceptAll",
						"assetsExceptionList": [],
						"depositorsAllowList": []
					}
				},
				"flags": [],
				"address": "account_rdx12yy8n09a0w907vrjyj4hws2yptrm3rdjv84l9sr24e3w7pk7nuxst8"
			}
            "#,
        );
    }

    #[test]
    fn json_roundtrip_bob() {
        let model = Account::placeholder_bob();
        assert_eq_after_json_roundtrip(
            &model,
            r#"
            {
				"securityState": {
					"unsecuredEntityControl": {
						"transactionSigning": {
							"badge": {
								"virtualSource": {
									"hierarchicalDeterministicPublicKey": {
										"publicKey": {
											"curve": "curve25519",
											"compressedData": "08740a2fd178c40ce71966a6537f780978f7f00548cfb59196344b5d7d67e9cf"
										},
										"derivationPath": {
											"scheme": "cap26",
											"path": "m/44H/1022H/1H/525H/1460H/1H"
										}
									},
									"discriminator": "hierarchicalDeterministicPublicKey"
								},
								"discriminator": "virtualSource"
							},
							"factorSourceID": {
								"fromHash": {
									"kind": "device",
									"body": "3c986ebf9dcd9167a97036d3b2c997433e85e6cc4e4422ad89269dac7bfea240"
								},
								"discriminator": "fromHash"
							}
						}
					},
					"discriminator": "unsecured"
				},
				"networkID": 1,
				"appearanceID": 1,
				"flags": [],
				"displayName": "Bob",
				"onLedgerSettings": {
					"thirdPartyDeposits": {
						"depositRule": "acceptAll",
						"assetsExceptionList": [],
						"depositorsAllowList": []
					}
				},
				"flags": [],
				"address": "account_rdx129a9wuey40lducsf6yu232zmzk5kscpvnl6fv472r0ja39f3hced69"
			}
            "#,
        );
    }
}
