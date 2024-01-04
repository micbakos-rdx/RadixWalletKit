import radix_wallet_kit

extension Profile {
	fileprivate static let placeholder = newProfilePlaceholder()
}

func test() throws {
	let wallet = Wallet(
		profile: Profile.placeholder
	)
	do {
		let profile = wallet.profileSnapshot()
		assert(profile.networks.count > 1)
		let mainnet = profile.networks[0]
		assert(mainnet.id == .mainnet)
		let mainnetAccounts = mainnet.accounts
		assert(mainnetAccounts.count > 1)
		let account = mainnetAccounts[0]
		assert(account.displayName.value == "Alice")
		let address = account.address
		assert(
			address.address
				== "account_rdx12yy8n09a0w907vrjyj4hws2yptrm3rdjv84l9sr24e3w7pk7nuxst8"
		)
		let newName = try newDisplayName(name: "Satoshi")

		let renamed = try wallet.changeNameOfAccount(
			address: address, to: newName
		)
		assert(
			renamed
				.displayName.value == "Satoshi")

		assert(account.displayName.value == "Alice")  // all types are VALUE types, so the prev `let` variable should NOT have been changed (which would be the case if we used classes...)
	} catch {
		print("Failed to do stuff ❌ error: \(error)")
		return
	}

	do {
		let profile = wallet.profileSnapshot()
		let mainnet = profile.networks[0]
		let mainnetAccounts = mainnet.accounts
		let account = mainnetAccounts[0]
		assert(account.displayName.value == "Satoshi")
	}

}

try! test()
