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

//! Template chain specification for CLI.

use crate::cli::{
	bridge,
	encode_call::{self, CliEncodeCall},
	encode_message, CliChain,
};
use codec::Decode;
use frame_support::weights::Weight;
use relay_template_client::Template;
use sp_version::RuntimeVersion;

impl CliChain for Template {
	const RUNTIME_VERSION: RuntimeVersion = bp_template::VERSION;

	type KeyPair = sp_core::sr25519::Pair;
	type MessagePayload = ();

	fn ss58_format() -> u16 {
		22
	}

	fn max_extrinsic_weight() -> Weight {
		0
	}

	fn encode_message(_message: encode_message::MessagePayload) -> Result<Self::MessagePayload, String> {
		Err("Sending messages from Template is not yet supported.".into())
	}
}

impl CliEncodeCall for Template {
	fn max_extrinsic_size() -> u32 {
		bp_template::max_extrinsic_size()
	}

	fn encode_call(call: &encode_call::Call) -> anyhow::Result<Self::Call> {
		use encode_call::Call;

		Ok(match call {
			Call::Raw { data } => Decode::decode(&mut &*data.0)?,
			Call::Remark { remark_payload, .. } => template_runtime::Call::System(
				template_runtime::SystemCall::remark(remark_payload.as_ref().map(|x| x.0.clone()).unwrap_or_default()),
			),
			Call::Transfer { recipient, amount } => template_runtime::Call::Balances(
				template_runtime::BalancesCall::transfer(recipient.raw_id().into(), amount.0),
			),
			Call::BridgeSendMessage {
				lane,
				payload,
				fee,
				bridge_instance_index,
			} => match *bridge_instance_index {
				bridge::TEMPLATE_TO_MILLAU_INDEX => {
					let payload = Decode::decode(&mut &*payload.0)?;
					template_runtime::Call::BridgeMillauMessages(template_runtime::MessagesCall::send_message(
						lane.0, payload, fee.0,
					))
				}
				_ => anyhow::bail!(
					"Unsupported target bridge pallet with instance index: {}",
					bridge_instance_index
				),
			},
		})
	}
}
