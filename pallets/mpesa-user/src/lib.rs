#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// specifying the parameters and types on which it depends.
#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn getLProviderIdentity)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	type TotalLiquidityPoolAmount<T> = StorageValue<_, u128, ValueQuery>;
	type TotalLiquidtyProviders<T> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;
	pub(super) type LProviderIdentity<T>: map hasher(blake2_128_concat) Vec<u8> => StorageValue<Option<T::AccountId>>;
	// ( identity, LPshare_key ) => LPshare_value
	pub(super) type LPShareAmount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u8>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::new_LProvider(pub(super) fn get_newLProvider)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NewLiquidityProvider(u32, T::AccountId),
		ContributedLPShares(u32, T::AccountId),
		TransferLPTokens(u32, T::AccountId, T::AccountId),
		ReceiveLPTokens(u32, T::AccountId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		LProviderIdentityAlreadyExists,
		LProviderIdentityDoesNotExist,
		MinimumLPShareAmountNotMet,
		NotEnoughLiquidity,
		LProviderNotAuthorised,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_LProviderIdentity(origin: OriginFor<T>, newmember: u32) -> DispatchResult {
			// add LProvider in Vec<u8> storage and return the LProvider's identity
			let who = ensure_signed(origin)?;
			let identity = Self::get_LProviderIdentity(&who);
			ensure!(identity.is_none(), Error::<T>::LProviderIdentityAlreadyExists);

			//update storage
			TotalLiquidtyProviders::put(newmember)

			// emit event of the created LProvider accountID
			Self::get_newLProvider(Event::NewLiquidityProvider(newmember, who));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn contribute_LPShares(origin: OriginFor<T>, amount: u128) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let identity = Self::get_LProviderIdentity(&who);
			ensure!(identity.is_some(), Error::<T>::LProviderIdentityDoesNotExist);

			//update storage
			TotalLiquidityPoolAmount::put(amount)

			// emit event of the created LProvider accountID
			Self::get_newLProvider(Event::ContributedLPShares(amount, who));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_LPShares(origin: OriginFor<T>, to: T::AccountId, amount: u128) -> DispatchResult {
			// add LProvider in Vec<u8> storage and return the LProvider's identity
			let who = ensure_signed(origin)?;
			let identity = Self::get_LProviderIdentity(&who);
			ensure!(identity.is_some(), Error::<T>::LProviderIdentityDoesNotExist);

			//update storage
			TotalLiquidityPoolAmount::put(amount)

			// emit event of the created LProvider accountID
			Self::get_newLProvider(Event::TransferLPTokens(amount, who, to));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn receive_LPShares(origin: OriginFor<T>, from: T::AccountId, amount: u128) -> DispatchResult {
			// add LProvider in Vec<u8> storage and return the LProvider's identity
			let who = ensure_signed(origin)?;
			let identity = Self::get_LProviderIdentity(&who);
			ensure!(identity.is_some(), Error::<T>::LProviderIdentityDoesNotExist);

			//update storage
			TotalLiquidityPoolAmount::put(amount)

			// emit event of the created LProvider accountID
			Self::get_newLProvider(Event::ReceiveLPTokens(amount, who, from));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delete_LProviderIdentity(origin: OriginFor<T>) -> DispatchResult {
			// add LProvider in Vec<u8> storage and return the LProvider's identity
			let who = ensure_signed(origin)?;
			let identity = Self::get_LProviderIdentity(&who);
			ensure!(identity.is_some(), Error::<T>::LProviderIdentityDoesNotExist);

			//update storage
			TotalLiquidtyProviders::put(who)

			// emit event of the created LProvider accountID
			Self::get_newLProvider(Event::NewLiquidityProvider(who));

			Ok(())
		}
	}
}