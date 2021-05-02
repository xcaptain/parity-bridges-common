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

//! Everything required to serve Millau <-> Template messages.

use crate::Runtime;

use bp_messages::{
	source_chain::TargetHeaderChain,
	target_chain::{ProvedMessages, SourceHeaderChain},
	InboundLaneData, LaneId, Message, MessageNonce, Parameter as MessagesParameter,
};
use bp_runtime::{InstanceId, TEMPLATE_BRIDGE_INSTANCE};
use bridge_runtime_common::messages::{self, MessageBridge, MessageTransaction};
use codec::{Decode, Encode};
use frame_support::{
	parameter_types,
	weights::{DispatchClass, Weight},
	RuntimeDebug,
};
use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128};
use sp_std::{convert::TryFrom, ops::RangeInclusive};

/// Initial value of `TemplateToMillauConversionRate` parameter.
pub const INITIAL_TEMPLATE_TO_MILLAU_CONVERSION_RATE: FixedU128 = FixedU128::from_inner(FixedU128::DIV);

parameter_types! {
	/// Template to Millau conversion rate. Initially we treat both tokens as equal.
	pub storage TemplateToMillauConversionRate: FixedU128 = INITIAL_TEMPLATE_TO_MILLAU_CONVERSION_RATE;
}

/// Message payload for Millau -> Template messages.
pub type ToTemplateMessagePayload = messages::source::FromThisChainMessagePayload<WithTemplateMessageBridge>;

/// Message verifier for Millau -> Template messages.
pub type ToTemplateMessageVerifier = messages::source::FromThisChainMessageVerifier<WithTemplateMessageBridge>;

/// Message payload for Template -> Millau messages.
pub type FromTemplateMessagePayload = messages::target::FromBridgedChainMessagePayload<WithTemplateMessageBridge>;

/// Encoded Millau Call as it comes from Template.
pub type FromTemplateEncodedCall = messages::target::FromBridgedChainEncodedMessageCall<WithTemplateMessageBridge>;

/// Messages proof for Template -> Millau messages.
type FromTemplateMessagesProof = messages::target::FromBridgedChainMessagesProof<bp_template::Hash>;

/// Messages delivery proof for Millau -> Template messages.
type ToTemplateMessagesDeliveryProof = messages::source::FromBridgedChainMessagesDeliveryProof<bp_template::Hash>;

/// Call-dispatch based message dispatch for Template -> Millau messages.
pub type FromTemplateMessageDispatch = messages::target::FromBridgedChainMessageDispatch<
	WithTemplateMessageBridge,
	crate::Runtime,
	crate::TemplateDispatchInstance,
>;

/// Millau <-> Template message bridge.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct WithTemplateMessageBridge;

impl MessageBridge for WithTemplateMessageBridge {
	const INSTANCE: InstanceId = TEMPLATE_BRIDGE_INSTANCE;

	const RELAYER_FEE_PERCENT: u32 = 10;

	type ThisChain = Millau;
	type BridgedChain = Template;

	fn bridged_balance_to_this_balance(bridged_balance: bp_template::Balance) -> bp_millau::Balance {
		bp_millau::Balance::try_from(TemplateToMillauConversionRate::get().saturating_mul_int(bridged_balance))
			.unwrap_or(bp_millau::Balance::MAX)
	}
}

/// Millau chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct Millau;

impl messages::ChainWithMessages for Millau {
	type Hash = bp_millau::Hash;
	type AccountId = bp_millau::AccountId;
	type Signer = bp_millau::AccountSigner;
	type Signature = bp_millau::Signature;
	type Weight = Weight;
	type Balance = bp_millau::Balance;

	type MessagesInstance = crate::WithTemplateMessagesInstance;
}

impl messages::ThisChainWithMessages for Millau {
	type Call = crate::Call;

	fn is_outbound_lane_enabled(lane: &LaneId) -> bool {
		*lane == [0, 0, 0, 0] || *lane == [0, 0, 0, 1]
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}

	fn estimate_delivery_confirmation_transaction() -> MessageTransaction<Weight> {
		let inbound_data_size =
			InboundLaneData::<bp_millau::AccountId>::encoded_size_hint(bp_millau::MAXIMAL_ENCODED_ACCOUNT_ID_SIZE, 1)
				.unwrap_or(u32::MAX);

		MessageTransaction {
			dispatch_weight: bp_millau::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
			size: inbound_data_size
				.saturating_add(bp_template::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_millau::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> bp_millau::Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			bp_millau::BlockWeights::get().get(DispatchClass::Normal).base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}

/// Template chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct Template;

impl messages::ChainWithMessages for Template {
	type Hash = bp_template::Hash;
	type AccountId = bp_template::AccountId;
	type Signer = bp_template::AccountSigner;
	type Signature = bp_template::Signature;
	type Weight = Weight;
	type Balance = bp_template::Balance;

	type MessagesInstance = crate::WithTemplateMessagesInstance;
}

impl messages::BridgedChainWithMessages for Template {
	fn maximal_extrinsic_size() -> u32 {
		bp_template::max_extrinsic_size()
	}

	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Weight> {
		// we don't want to relay too large messages + keep reserve for future upgrades
		let upper_limit =
			messages::target::maximal_incoming_message_dispatch_weight(bp_template::max_extrinsic_weight());

		// we're charging for payload bytes in `WithTemplateMessageBridge::transaction_payment` function
		//
		// this bridge may be used to deliver all kind of messages, so we're not making any assumptions about
		// minimal dispatch weight here

		0..=upper_limit
	}

	fn estimate_delivery_transaction(
		message_payload: &[u8],
		message_dispatch_weight: Weight,
	) -> MessageTransaction<Weight> {
		let message_payload_len = u32::try_from(message_payload.len()).unwrap_or(u32::MAX);
		let extra_bytes_in_payload = Weight::from(message_payload_len)
			.saturating_sub(pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH.into());

		MessageTransaction {
			dispatch_weight: extra_bytes_in_payload
				.saturating_mul(bp_template::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT)
				.saturating_add(bp_template::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT)
				.saturating_add(message_dispatch_weight),
			size: message_payload_len
				.saturating_add(bp_millau::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_template::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> bp_template::Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			bp_template::BlockWeights::get()
				.get(DispatchClass::Normal)
				.base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}

impl TargetHeaderChain<ToTemplateMessagePayload, bp_template::AccountId> for Template {
	type Error = &'static str;
	// The proof is:
	// - hash of the header this proof has been created with;
	// - the storage proof or one or several keys;
	// - id of the lane we prove state of.
	type MessagesDeliveryProof = ToTemplateMessagesDeliveryProof;

	fn verify_message(payload: &ToTemplateMessagePayload) -> Result<(), Self::Error> {
		messages::source::verify_chain_message::<WithTemplateMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_millau::AccountId>), Self::Error> {
		messages::source::verify_messages_delivery_proof::<
			WithTemplateMessageBridge,
			Runtime,
			crate::TemplateGrandpaInstance,
		>(proof)
	}
}

impl SourceHeaderChain<bp_template::Balance> for Template {
	type Error = &'static str;
	// The proof is:
	// - hash of the header this proof has been created with;
	// - the storage proof or one or several keys;
	// - id of the lane we prove messages for;
	// - inclusive range of messages nonces that are proved.
	type MessagesProof = FromTemplateMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<bp_template::Balance>>, Self::Error> {
		messages::target::verify_messages_proof::<WithTemplateMessageBridge, Runtime, crate::TemplateGrandpaInstance>(
			proof,
			messages_count,
		)
	}
}

/// Millau -> Template message lane pallet parameters.
#[derive(RuntimeDebug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum MillauToTemplateMessagesParameter {
	/// The conversion formula we use is: `MillauTokens = TemplateTokens * conversion_rate`.
	TemplateToMillauConversionRate(FixedU128),
}

impl MessagesParameter for MillauToTemplateMessagesParameter {
	fn save(&self) {
		match *self {
			MillauToTemplateMessagesParameter::TemplateToMillauConversionRate(ref conversion_rate) => {
				TemplateToMillauConversionRate::set(conversion_rate)
			}
		}
	}
}
