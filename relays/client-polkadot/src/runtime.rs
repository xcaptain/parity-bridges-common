// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Types that are specific to the Polkadot runtime.

use bp_messages::{LaneId, UnrewardedRelayersState};
use bp_polkadot_core::{AccountAddress, Balance, PolkadotLike};
use bp_runtime::Chain;
use codec::{Compact, Decode, Encode};
use frame_support::weights::Weight;
use sp_runtime::FixedU128;

/// Unchecked Polkadot extrinsic.
pub type UncheckedExtrinsic = bp_polkadot_core::UncheckedExtrinsic<Call>;

/// Kusama account ownership digest from Polkadot.
///
/// The byte vector returned by this function should be signed with a Kusama account private key.
/// This way, the owner of `kusam_account_id` on Polkadot proves that the Kusama account private key
/// is also under his control.
pub fn polkadot_to_kusama_account_ownership_digest<Call, AccountId, SpecVersion>(
	kusama_call: &Call,
	kusam_account_id: AccountId,
	kusama_spec_version: SpecVersion,
) -> Vec<u8>
where
	Call: codec::Encode,
	AccountId: codec::Encode,
	SpecVersion: codec::Encode,
{
	pallet_bridge_dispatch::account_ownership_digest(
		kusama_call,
		kusam_account_id,
		kusama_spec_version,
		bp_runtime::POLKADOT_CHAIN_ID,
		bp_runtime::KUSAMA_CHAIN_ID,
	)
}

/// Polkadot Runtime `Call` enum.
///
/// The enum represents a subset of possible `Call`s we can send to Polkadot chain.
/// Ideally this code would be auto-generated from metadata, because we want to
/// avoid depending directly on the ENTIRE runtime just to get the encoding of `Dispatchable`s.
///
/// All entries here (like pretty much in the entire file) must be kept in sync with Polkadot
/// `construct_runtime`, so that we maintain SCALE-compatibility.
///
/// See: [link](https://github.com/paritytech/kusama/blob/master/runtime/kusam/src/lib.rs)
#[allow(clippy::large_enum_variant)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Call {
	/// System pallet.
	#[codec(index = 0)]
	System(SystemCall),
	/// Balances pallet.
	#[codec(index = 5)]
	Balances(BalancesCall),
	/// Kusama bridge pallet.
	#[codec(index = 110)]
	BridgeKusamaGrandpa(BridgeKusamaGrandpaCall),
	/// Kusama messages pallet.
	#[codec(index = 111)]
	BridgeKusamaMessages(BridgeKusamaMessagesCall),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum SystemCall {
	#[codec(index = 1)]
	remark(Vec<u8>),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum BalancesCall {
	#[codec(index = 0)]
	transfer(AccountAddress, Compact<Balance>),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum BridgeKusamaGrandpaCall {
	#[codec(index = 0)]
	submit_finality_proof(
		Box<<PolkadotLike as Chain>::Header>,
		bp_header_chain::justification::GrandpaJustification<<PolkadotLike as Chain>::Header>,
	),
	#[codec(index = 1)]
	initialize(bp_header_chain::InitializationData<<PolkadotLike as Chain>::Header>),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum BridgeKusamaMessagesCall {
	#[codec(index = 2)]
	update_pallet_parameter(BridgeKusamaMessagesParameter),
	#[codec(index = 3)]
	send_message(
		LaneId,
		bp_message_dispatch::MessagePayload<
			bp_polkadot::AccountId,
			bp_kusama::AccountId,
			bp_kusama::AccountPublic,
			Vec<u8>,
		>,
		bp_polkadot::Balance,
	),
	#[codec(index = 5)]
	receive_messages_proof(
		bp_kusama::AccountId,
		bridge_runtime_common::messages::target::FromBridgedChainMessagesProof<bp_kusama::Hash>,
		u32,
		Weight,
	),
	#[codec(index = 6)]
	receive_messages_delivery_proof(
		bridge_runtime_common::messages::source::FromBridgedChainMessagesDeliveryProof<
			bp_kusama::Hash,
		>,
		UnrewardedRelayersState,
	),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum BridgeKusamaMessagesParameter {
	#[codec(index = 0)]
	KusamaToPolkadotConversionRate(FixedU128),
}

impl sp_runtime::traits::Dispatchable for Call {
	type Origin = ();
	type Config = ();
	type Info = ();
	type PostInfo = ();

	fn dispatch(self, _origin: Self::Origin) -> sp_runtime::DispatchResultWithInfo<Self::PostInfo> {
		unimplemented!("The Call is not expected to be dispatched.")
	}
}
