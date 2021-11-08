use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	log::{error, trace},
	traits::fungibles::{Inspect, InspectMetadata, Transfer},
};
use pallet_assets::WeightInfo;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
pub struct Psp22BalanceOfInput<AssetId, AccountId> {
	pub asset_id: AssetId,
	pub owner: AccountId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
pub struct Psp22TransferInput<AssetId, AccountId, Balance> {
	pub asset_id: AssetId,
	pub to: AccountId,
	pub value: Balance,
}

pub struct Psp22Extension;

impl<T> ChainExtension<T> for Psp22Extension
where
	T: SysConfig + pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// PSP22 Metadata interfaces

			// PSP22Metadata::token_name
			0x3d261bd4 => {
				let mut env = env.buf_in_buf_out();
				let asset_id = env.read_as()?;

				let name =
					<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::name(&asset_id)
						.encode();
				trace!(
					target: "runtime",
					"[ChainExtension] <PSP22Metadata::token_name"
				);
				env.write(&name, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call token_name")
				})?;
			},

			// PSP22Metadata::token_symbol
			0x34205be5 => {
				let mut env = env.buf_in_buf_out();
				let asset_id = env.read_as()?;

				let symbol =
					<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::symbol(&asset_id)
						.encode();
				trace!(
					target: "runtime",
					"[ChainExtension] PSP22Metadata::token_symbol"
				);
				env.write(&symbol, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call token_symbol")
				})?;
			},

			// PSP22Metadata::token_decimals
			0x7271b782 => {
				let mut env = env.buf_in_buf_out();
				let asset_id = env.read_as()?;

				let decimals =
					<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::decimals(
						&asset_id,
					)
					.encode();
				trace!(
					target: "runtime",
					"[ChainExtension] PSP22Metadata::token_decimals"
				);
				env.write(&decimals, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call token_decimals")
				})?;
			},

			// Note: We use the PSP22 interface selectors as function IDs,
			// there is no need but it makes sense from a convention perspective.

			// PSP22 interfaces

			// PSP22::total_supply
			0x162df8c2 => {
				let mut env = env.buf_in_buf_out();
				let asset_id = env.read_as()?;

				let total_supply =
					<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::total_issuance(asset_id);
				let result = total_supply.encode();
				trace!(
					target: "runtime",
					"[ChainExtension] PSP22::total_supply"
				);
				env.write(&result, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call total_supply")
				})?;
			},

			// PSP22::balance_of
			0x6568382f => {
				let mut env = env.buf_in_buf_out();
				let input: Psp22BalanceOfInput<T::AssetId, T::AccountId> = env.read_as()?;

				let balance = <pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(
					input.asset_id,
					&input.owner,
				);
				let result = balance.encode();
				trace!(
					target: "runtime",
					"[ChainExtension] PSP22::balance_of"
				);
				env.write(&result, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call balance_of")
				})?;
			},

			// PSP22::approve
			// 0xb20f1bbd => {
			//     let mut env = env.buf_in_buf_out();

			//     let mut bytes_owner: [u8; 32] = [0; 32];
			//     let addr_vec_owner: Vec<u8> = env.ext().caller().encode();
			//     for i in 0..32 {
			//         bytes_owner[i] = addr_vec_owner[i];
			//     }
			//     let owner = AccountId32::from(bytes_owner);
			//     debug!("owner: {:?}", owner);

			//     let asset_id = env.read_as()?;
			//     debug!("asset_id: {:?}", asset_id);

			//     // let mut bytes_spender: [u8; 32] = [0; 32];
			//     // let addr_vec_spender: Vec<u8> = env.ext().caller().encode();
			//     // for i in 0..32 {
			//     //     bytes_spender[i] = addr_vec_spender[i];
			//     // }
			//     // let spender = AccountId32::from(bytes_spender);
			//     let spender: AccountId32 =  env.read_as()?;
			//     debug!("spender: {:?}", spender);

			//     let value = env.read_as()?;
			//     debug!("value: {:?}", value);

			//     let result = pallet_assets::Pallet::<Runtime>::approve_transfer(
			//         RawOrigin::from(Some(owner)).into(), asset_id, spender, value);
			//     trace!(
			//         target: "runtime",
			//         "[ChainExtension]|call|func_id:{:}",
			//         func_id
			//     );
			//     result.map_err(|_| {
			//         DispatchError::Other("ChainExtension failed to call approve")
			//     })?;
			// }

			// P2P22:transfer
			0xdb20f9f5 => {
				let mut env = env.buf_in_buf_out();

				let transfer_fee = <T as pallet_assets::Config>::WeightInfo::transfer();
				let charged_amount =
					env.charge_weight(transfer_fee.saturating_add(transfer_fee / 10))?;
				trace!(
					target: "runtime",
					"[ChainExtension]|call|transfer / charge_weight:{:?}",
					charged_amount
				);

				let input: Psp22TransferInput<T::AssetId, T::AccountId, T::Balance> =
					env.read_as()?;
				let sender = env.ext().caller();

				let result = <pallet_assets::Pallet<T> as Transfer<T::AccountId>>::transfer(
					input.asset_id,
					sender,
					&input.to,
					input.value,
					true,
				);
				trace!(
					target: "runtime",
					"[ChainExtension]|call|transfer"
				);
				result.map_err(|err| {
					trace!(
						target: "runtime",
						"PSP22 Transfer failed:{:?}",
						err
					);
					DispatchError::Other("ChainExtension failed to call transfer")
				})?;

				// env.adjust_weight(charged, actual_weight)
			},
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"))
			},
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
