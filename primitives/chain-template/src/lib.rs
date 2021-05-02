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

#![cfg_attr(not(feature = "std"), no_std)]
// RuntimeApi generated functions
#![allow(clippy::too_many_arguments)]
// Runtime-generated DecodeLimit::decode_all_with_depth_limit
#![allow(clippy::unnecessary_mut_passed)]

use bp_messages::{LaneId, MessageNonce, UnrewardedRelayersState, Weight};
use frame_support::weights::constants::WEIGHT_PER_SECOND;
use sp_runtime::Perbill;
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

pub use bp_polkadot_core::*;

/// Template Chain
pub type Template = PolkadotLike;

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: sp_version::create_runtime_str!("node-template"),
	impl_name: sp_version::create_runtime_str!("node-template"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: sp_version::create_apis_vec![[]],
	transaction_version: 1,
};

pub type AccountSigner = sp_runtime::MultiSigner;

// We use this to get the account on Template (target) which is derived from Millau's (source)
// account.
pub fn derive_account_from_millau_id(id: bp_runtime::SourceAccount<AccountId>) -> AccountId {
	let encoded_id = bp_runtime::derive_account_id(bp_runtime::MILLAU_BRIDGE_INSTANCE, id);
	AccountIdConverter::convert(encoded_id)
}

/// Number of extra bytes (excluding size of storage value itself) of storage proof, built at
/// Template chain. This mostly depends on number of entries (and their density) in the storage trie.
/// Some reserve is reserved to account future chain growth.
pub const EXTRA_STORAGE_PROOF_SIZE: u32 = 1024;

/// Number of bytes, included in the signed Template transaction apart from the encoded call itself.
///
/// Can be computed by subtracting encoded call size from raw transaction size.
pub const TX_EXTRA_BYTES: u32 = 103;

/// Maximal size (in bytes) of encoded (using `Encode::encode()`) account id.
pub const MAXIMAL_ENCODED_ACCOUNT_ID_SIZE: u32 = 32;

/// Maximal weight of single Template block.
///
/// This represents two seconds of compute assuming a target block time of six seconds.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

/// Represents the average portion of a block's weight that will be used by an
/// `on_initialize()` runtime call.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);

/// Represents the portion of a block that will be used by Normal extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// Maximal number of unrewarded relayer entries at inbound lane.
pub const MAX_UNREWARDED_RELAYER_ENTRIES_AT_INBOUND_LANE: MessageNonce = 128;

/// Maximal number of unconfirmed messages at inbound lane.
pub const MAX_UNCONFIRMED_MESSAGES_AT_INBOUND_LANE: MessageNonce = 128;

/// Weight of single regular message delivery transaction on Template chain.
///
/// This value is a result of `pallet_bridge_messages::Pallet::receive_messages_proof_weight()` call
/// for the case when single message of `pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH` bytes is delivered.
/// The message must have dispatch weight set to zero. The result then must be rounded up to account
/// possible future runtime upgrades.
pub const DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT: Weight = 1_000_000_000;

/// Increase of delivery transaction weight on Template chain with every additional message byte.
///
/// This value is a result of `pallet_bridge_messages::WeightInfoExt::storage_proof_size_overhead(1)` call. The
/// result then must be rounded up to account possible future runtime upgrades.
pub const ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT: Weight = 25_000;

/// Maximal weight of single message delivery confirmation transaction on Template chain.
///
/// This value is a result of `pallet_bridge_messages::Pallet::receive_messages_delivery_proof` weight formula computation
/// for the case when single message is confirmed. The result then must be rounded up to account possible future
/// runtime upgrades.
pub const MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT: Weight = 2_000_000_000;

/// Name of the `TemplateFinalityApi::best_finalized` runtime method.
pub const BEST_FINALIZED_TEMPLATE_HEADER_METHOD: &str = "TemplateFinalityApi_best_finalized";
/// Name of the `TemplateFinalityApi::is_known_header` runtime method.
pub const IS_KNOWN_TEMPLATE_HEADER_METHOD: &str = "TemplateFinalityApi_is_known_header";

/// Name of the `ToTemplateOutboundLaneApi::estimate_message_delivery_and_dispatch_fee` runtime method.
pub const TO_TEMPLATE_ESTIMATE_MESSAGE_FEE_METHOD: &str =
	"ToTemplateOutboundLaneApi_estimate_message_delivery_and_dispatch_fee";
/// Name of the `ToTemplateOutboundLaneApi::messages_dispatch_weight` runtime method.
pub const TO_TEMPLATE_MESSAGES_DISPATCH_WEIGHT_METHOD: &str = "ToTemplateOutboundLaneApi_messages_dispatch_weight";
/// Name of the `ToTemplateOutboundLaneApi::latest_generated_nonce` runtime method.
pub const TO_TEMPLATE_LATEST_GENERATED_NONCE_METHOD: &str = "ToTemplateOutboundLaneApi_latest_generated_nonce";
/// Name of the `ToTemplateOutboundLaneApi::latest_received_nonce` runtime method.
pub const TO_TEMPLATE_LATEST_RECEIVED_NONCE_METHOD: &str = "ToTemplateOutboundLaneApi_latest_received_nonce";

/// Name of the `FromTemplateInboundLaneApi::latest_received_nonce` runtime method.
pub const FROM_TEMPLATE_LATEST_RECEIVED_NONCE_METHOD: &str = "FromTemplateInboundLaneApi_latest_received_nonce";
/// Name of the `FromTemplateInboundLaneApi::latest_onfirmed_nonce` runtime method.
pub const FROM_TEMPLATE_LATEST_CONFIRMED_NONCE_METHOD: &str = "FromTemplateInboundLaneApi_latest_confirmed_nonce";
/// Name of the `FromTemplateInboundLaneApi::unrewarded_relayers_state` runtime method.
pub const FROM_TEMPLATE_UNREWARDED_RELAYERS_STATE: &str = "FromTemplateInboundLaneApi_unrewarded_relayers_state";

/// The target length of a session (how often authorities change) on Template measured in of number of
/// blocks.
///
/// Note that since this is a target sessions may change before/after this time depending on network
/// conditions.
pub const SESSION_LENGTH: BlockNumber = 10 * time_units::MINUTES;

sp_api::decl_runtime_apis! {
	/// API for querying information about the finalized Template headers.
	///
	/// This API is implemented by runtimes that are bridging with the Template chain, not the
	/// Template runtime itself.
	pub trait TemplateFinalityApi {
		/// Returns number and hash of the best finalized header known to the bridge module.
		fn best_finalized() -> (BlockNumber, Hash);
		/// Returns true if the header is known to the runtime.
		fn is_known_header(hash: Hash) -> bool;
	}

	/// Outbound message lane API for messages that are sent to Template chain.
	///
	/// This API is implemented by runtimes that are sending messages to Template chain, not the
	/// Template runtime itself.
	pub trait ToTemplateOutboundLaneApi<OutboundMessageFee: Parameter, OutboundPayload: Parameter> {
		/// Estimate message delivery and dispatch fee that needs to be paid by the sender on
		/// this chain.
		///
		/// Returns `None` if message is too expensive to be sent to Template from this chain.
		///
		/// Please keep in mind that this method returns lowest message fee required for message
		/// to be accepted to the lane. It may be good idea to pay a bit over this price to account
		/// future exchange rate changes and guarantee that relayer would deliver your message
		/// to the target chain.
		fn estimate_message_delivery_and_dispatch_fee(
			lane_id: LaneId,
			payload: OutboundPayload,
		) -> Option<OutboundMessageFee>;
		/// Returns total dispatch weight and encoded payload size of all messages in given inclusive range.
		///
		/// If some (or all) messages are missing from the storage, they'll also will
		/// be missing from the resulting vector. The vector is ordered by the nonce.
		fn messages_dispatch_weight(
			lane: LaneId,
			begin: MessageNonce,
			end: MessageNonce,
		) -> Vec<(MessageNonce, Weight, u32)>;
		/// Returns nonce of the latest message, received by bridged chain.
		fn latest_received_nonce(lane: LaneId) -> MessageNonce;
		/// Returns nonce of the latest message, generated by given lane.
		fn latest_generated_nonce(lane: LaneId) -> MessageNonce;
	}

	/// Inbound message lane API for messages sent by Template chain.
	///
	/// This API is implemented by runtimes that are receiving messages from Template chain, not the
	/// Template runtime itself.
	pub trait FromTemplateInboundLaneApi {
		/// Returns nonce of the latest message, received by given lane.
		fn latest_received_nonce(lane: LaneId) -> MessageNonce;
		/// Nonce of latest message that has been confirmed to the bridged chain.
		fn latest_confirmed_nonce(lane: LaneId) -> MessageNonce;
		/// State of the unrewarded relayers set at given lane.
		fn unrewarded_relayers_state(lane: LaneId) -> UnrewardedRelayersState;
	}
}
