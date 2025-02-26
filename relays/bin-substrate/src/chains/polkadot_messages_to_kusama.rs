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

//! Polkadot-to-Kusama messages sync entrypoint.

use std::{ops::RangeInclusive, time::Duration};

use codec::Encode;
use sp_core::{Bytes, Pair};

use bp_messages::MessageNonce;
use bridge_runtime_common::messages::target::FromBridgedChainMessagesProof;
use frame_support::weights::Weight;
use messages_relay::message_lane::MessageLane;
use relay_kusama_client::{
	HeaderId as KusamaHeaderId, Kusama, SigningParams as KusamaSigningParams,
};
use relay_polkadot_client::{
	HeaderId as PolkadotHeaderId, Polkadot, SigningParams as PolkadotSigningParams,
};
use relay_substrate_client::{Chain, Client, TransactionSignScheme, UnsignedTransaction};
use relay_utils::metrics::MetricsParams;
use sp_runtime::{FixedPointNumber, FixedU128};
use substrate_relay_helper::{
	messages_lane::{
		select_delivery_transaction_limits, MessagesRelayParams, StandaloneMessagesMetrics,
		SubstrateMessageLane, SubstrateMessageLaneToSubstrate,
	},
	messages_source::SubstrateMessagesSource,
	messages_target::SubstrateMessagesTarget,
};

/// Polkadot-to-Kusama message lane.
pub type MessageLanePolkadotMessagesToKusama =
	SubstrateMessageLaneToSubstrate<Polkadot, PolkadotSigningParams, Kusama, KusamaSigningParams>;

#[derive(Clone)]
pub struct PolkadotMessagesToKusama {
	message_lane: MessageLanePolkadotMessagesToKusama,
}

impl SubstrateMessageLane for PolkadotMessagesToKusama {
	type MessageLane = MessageLanePolkadotMessagesToKusama;
	const OUTBOUND_LANE_MESSAGE_DETAILS_METHOD: &'static str =
		bp_kusama::TO_KUSAMA_MESSAGE_DETAILS_METHOD;
	const OUTBOUND_LANE_LATEST_GENERATED_NONCE_METHOD: &'static str =
		bp_kusama::TO_KUSAMA_LATEST_GENERATED_NONCE_METHOD;
	const OUTBOUND_LANE_LATEST_RECEIVED_NONCE_METHOD: &'static str =
		bp_kusama::TO_KUSAMA_LATEST_RECEIVED_NONCE_METHOD;

	const INBOUND_LANE_LATEST_RECEIVED_NONCE_METHOD: &'static str =
		bp_polkadot::FROM_POLKADOT_LATEST_RECEIVED_NONCE_METHOD;
	const INBOUND_LANE_LATEST_CONFIRMED_NONCE_METHOD: &'static str =
		bp_polkadot::FROM_POLKADOT_LATEST_CONFIRMED_NONCE_METHOD;
	const INBOUND_LANE_UNREWARDED_RELAYERS_STATE: &'static str =
		bp_polkadot::FROM_POLKADOT_UNREWARDED_RELAYERS_STATE;

	const BEST_FINALIZED_SOURCE_HEADER_ID_AT_TARGET: &'static str =
		bp_polkadot::BEST_FINALIZED_POLKADOT_HEADER_METHOD;
	const BEST_FINALIZED_TARGET_HEADER_ID_AT_SOURCE: &'static str =
		bp_kusama::BEST_FINALIZED_KUSAMA_HEADER_METHOD;

	const MESSAGE_PALLET_NAME_AT_SOURCE: &'static str =
		bp_polkadot::WITH_KUSAMA_MESSAGES_PALLET_NAME;
	const MESSAGE_PALLET_NAME_AT_TARGET: &'static str =
		bp_kusama::WITH_POLKADOT_MESSAGES_PALLET_NAME;

	const PAY_INBOUND_DISPATCH_FEE_WEIGHT_AT_TARGET_CHAIN: Weight =
		bp_kusama::PAY_INBOUND_DISPATCH_FEE_WEIGHT;

	type SourceChain = Polkadot;
	type TargetChain = Kusama;

	fn source_transactions_author(&self) -> bp_polkadot::AccountId {
		(*self.message_lane.source_sign.public().as_array_ref()).into()
	}

	fn make_messages_receiving_proof_transaction(
		&self,
		transaction_nonce: bp_runtime::IndexOf<Polkadot>,
		_generated_at_block: KusamaHeaderId,
		proof: <Self::MessageLane as MessageLane>::MessagesReceivingProof,
	) -> Bytes {
		let (relayers_state, proof) = proof;
		let call = relay_polkadot_client::runtime::Call::BridgeKusamaMessages(
			relay_polkadot_client::runtime::BridgeKusamaMessagesCall::receive_messages_delivery_proof(
				proof,
				relayers_state,
			),
		);
		let genesis_hash = *self.message_lane.source_client.genesis_hash();
		let transaction = Polkadot::sign_transaction(
			genesis_hash,
			&self.message_lane.source_sign,
			relay_substrate_client::TransactionEra::immortal(),
			UnsignedTransaction::new(call, transaction_nonce),
		);
		log::trace!(
			target: "bridge",
			"Prepared Kusama -> Polkadot confirmation transaction. Weight: <unknown>/{}, size: {}/{}",
			bp_polkadot::max_extrinsic_weight(),
			transaction.encode().len(),
			bp_polkadot::max_extrinsic_size(),
		);
		Bytes(transaction.encode())
	}

	fn target_transactions_author(&self) -> bp_kusama::AccountId {
		(*self.message_lane.target_sign.public().as_array_ref()).into()
	}

	fn make_messages_delivery_transaction(
		&self,
		transaction_nonce: bp_runtime::IndexOf<Kusama>,
		_generated_at_header: PolkadotHeaderId,
		_nonces: RangeInclusive<MessageNonce>,
		proof: <Self::MessageLane as MessageLane>::MessagesProof,
	) -> Bytes {
		let (dispatch_weight, proof) = proof;
		let FromBridgedChainMessagesProof { ref nonces_start, ref nonces_end, .. } = proof;
		let messages_count = nonces_end - nonces_start + 1;

		let call = relay_kusama_client::runtime::Call::BridgePolkadotMessages(
			relay_kusama_client::runtime::BridgePolkadotMessagesCall::receive_messages_proof(
				self.message_lane.relayer_id_at_source.clone(),
				proof,
				messages_count as _,
				dispatch_weight,
			),
		);
		let genesis_hash = *self.message_lane.target_client.genesis_hash();
		let transaction = Kusama::sign_transaction(
			genesis_hash,
			&self.message_lane.target_sign,
			relay_substrate_client::TransactionEra::immortal(),
			UnsignedTransaction::new(call, transaction_nonce),
		);
		log::trace!(
			target: "bridge",
			"Prepared Polkadot -> Kusama delivery transaction. Weight: <unknown>/{}, size: {}/{}",
			bp_kusama::max_extrinsic_weight(),
			transaction.encode().len(),
			bp_kusama::max_extrinsic_size(),
		);
		Bytes(transaction.encode())
	}
}

/// Polkadot node as messages source.
type PolkadotSourceClient = SubstrateMessagesSource<PolkadotMessagesToKusama>;

/// Kusama node as messages target.
type KusamaTargetClient = SubstrateMessagesTarget<PolkadotMessagesToKusama>;

/// Run Polkadot-to-Kusama messages sync.
pub async fn run(
	params: MessagesRelayParams<Polkadot, PolkadotSigningParams, Kusama, KusamaSigningParams>,
) -> anyhow::Result<()> {
	let stall_timeout = Duration::from_secs(5 * 60);
	let relayer_id_at_polkadot = (*params.source_sign.public().as_array_ref()).into();

	let lane_id = params.lane_id;
	let source_client = params.source_client;
	let lane = PolkadotMessagesToKusama {
		message_lane: SubstrateMessageLaneToSubstrate {
			source_client: source_client.clone(),
			source_sign: params.source_sign,
			target_client: params.target_client.clone(),
			target_sign: params.target_sign,
			relayer_id_at_source: relayer_id_at_polkadot,
		},
	};

	// 2/3 is reserved for proofs and tx overhead
	let max_messages_size_in_single_batch = bp_kusama::max_extrinsic_size() / 3;
	// we don't know exact weights of the Kusama runtime. So to guess weights we'll be using
	// weights from Rialto and then simply dividing it by x2.
	let (max_messages_in_single_batch, max_messages_weight_in_single_batch) =
		select_delivery_transaction_limits::<
			pallet_bridge_messages::weights::RialtoWeight<rialto_runtime::Runtime>,
		>(
			bp_kusama::max_extrinsic_weight(),
			bp_kusama::MAX_UNREWARDED_RELAYER_ENTRIES_AT_INBOUND_LANE,
		);
	let (max_messages_in_single_batch, max_messages_weight_in_single_batch) =
		(max_messages_in_single_batch / 2, max_messages_weight_in_single_batch / 2);

	log::info!(
		target: "bridge",
		"Starting Polkadot -> Kusama messages relay.\n\t\
			Polkadot relayer account id: {:?}\n\t\
			Max messages in single transaction: {}\n\t\
			Max messages size in single transaction: {}\n\t\
			Max messages weight in single transaction: {}\n\t\
			Relayer mode: {:?}",
		lane.message_lane.relayer_id_at_source,
		max_messages_in_single_batch,
		max_messages_size_in_single_batch,
		max_messages_weight_in_single_batch,
		params.relayer_mode,
	);

	let (metrics_params, metrics_values) = add_standalone_metrics(
		Some(messages_relay::message_lane_loop::metrics_prefix::<
			<PolkadotMessagesToKusama as SubstrateMessageLane>::MessageLane,
		>(&lane_id)),
		params.metrics_params,
		source_client.clone(),
	)?;
	messages_relay::message_lane_loop::run(
		messages_relay::message_lane_loop::Params {
			lane: lane_id,
			source_tick: Polkadot::AVERAGE_BLOCK_INTERVAL,
			target_tick: Kusama::AVERAGE_BLOCK_INTERVAL,
			reconnect_delay: relay_utils::relay_loop::RECONNECT_DELAY,
			stall_timeout,
			delivery_params: messages_relay::message_lane_loop::MessageDeliveryParams {
				max_unrewarded_relayer_entries_at_target:
					bp_kusama::MAX_UNREWARDED_RELAYER_ENTRIES_AT_INBOUND_LANE,
				max_unconfirmed_nonces_at_target:
					bp_kusama::MAX_UNCONFIRMED_MESSAGES_AT_INBOUND_LANE,
				max_messages_in_single_batch,
				max_messages_weight_in_single_batch,
				max_messages_size_in_single_batch,
				relayer_mode: params.relayer_mode,
			},
		},
		PolkadotSourceClient::new(
			source_client.clone(),
			lane.clone(),
			lane_id,
			params.target_to_source_headers_relay,
		),
		KusamaTargetClient::new(
			params.target_client,
			lane,
			lane_id,
			metrics_values,
			params.source_to_target_headers_relay,
		),
		metrics_params,
		futures::future::pending(),
	)
	.await
}

/// Add standalone metrics for the Polkadot -> Kusama messages loop.
pub(crate) fn add_standalone_metrics(
	metrics_prefix: Option<String>,
	metrics_params: MetricsParams,
	source_client: Client<Polkadot>,
) -> anyhow::Result<(MetricsParams, StandaloneMessagesMetrics)> {
	let kusama_to_polkadot_conversion_rate_key = bp_runtime::storage_parameter_key(
		bp_polkadot::KUSAMA_TO_POLKADOT_CONVERSION_RATE_PARAMETER_NAME,
	)
	.0;

	substrate_relay_helper::messages_lane::add_standalone_metrics::<PolkadotMessagesToKusama>(
		metrics_prefix,
		metrics_params,
		source_client,
		Some(crate::chains::kusama::TOKEN_ID),
		Some(crate::chains::polkadot::TOKEN_ID),
		Some((
			sp_core::storage::StorageKey(kusama_to_polkadot_conversion_rate_key),
			// starting relay before this parameter will be set to some value may cause troubles
			FixedU128::from_inner(FixedU128::DIV),
		)),
	)
}

/// Update Kusama -> Polkadot conversion rate, stored in Polkadot runtime storage.
pub(crate) async fn update_kusama_to_polkadot_conversion_rate(
	client: Client<Polkadot>,
	signer: <Polkadot as TransactionSignScheme>::AccountKeyPair,
	updated_rate: f64,
) -> anyhow::Result<()> {
	let genesis_hash = *client.genesis_hash();
	let signer_id = (*signer.public().as_array_ref()).into();
	client
		.submit_signed_extrinsic(signer_id, move |_, transaction_nonce| {
			Bytes(
				Polkadot::sign_transaction(
					genesis_hash,
					&signer,
					relay_substrate_client::TransactionEra::immortal(),
					UnsignedTransaction::new(
						relay_polkadot_client::runtime::Call::BridgeKusamaMessages(
							relay_polkadot_client::runtime::BridgeKusamaMessagesCall::update_pallet_parameter(
								relay_polkadot_client::runtime::BridgeKusamaMessagesParameter::KusamaToPolkadotConversionRate(
									sp_runtime::FixedU128::from_float(updated_rate),
								)
							)
						),
						transaction_nonce,
					),
				)
				.encode(),
			)
		})
		.await
		.map(drop)
		.map_err(|err| anyhow::format_err!("{:?}", err))
}
