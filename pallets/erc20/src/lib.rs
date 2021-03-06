#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use log::{warn};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//The pallet's runtime storage items.
	//https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type BalanceOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type TotalSupply<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_init)]
	pub type IsInit<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn allowance)]
	pub type Allowance<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		u64,
		ValueQuery
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// Token was initialized by user
		Initialized(T::AccountId),
        /// Tokens successfully transferred between users
        Transfer(T::AccountId, T::AccountId, u64), // (from, to, value)
		/// Allowance successfully created
        Approval(T::AccountId, T::AccountId, u64), // (from, to, value)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		// Attempted to initialize the token
		//after it had already been initialized.
		AlreadyInitialized,

		InsufficientFunds,

		InsufficientApprovedFunds
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}


		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn init(origin: OriginFor<T>, total_supply: u64) -> DispatchResult {

        	let sender = ensure_signed(origin)?;
        	ensure!(!Self::is_init(), <Error<T>>::AlreadyInitialized);
			warn!("Request sent by----->: {:?}", Self::is_init());
        	<TotalSupply<T>>::put(total_supply);
        	<BalanceOf<T>>::insert(&sender, total_supply);

        	<IsInit::<T>>::put(true);
        	// Emit an event
        	Self::deposit_event(Event::Initialized(sender));
        	Ok(().into())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, value: u64) -> DispatchResult {
        	let sender = ensure_signed(origin)?;

			// get the balance values
			let from_balance = Self::balance_of(&sender);
			let to_balance = Self::balance_of(&to);

			// Calculate new balances
			let updated_from_balance = from_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientFunds)?;
			let updated_to_balance = to_balance.checked_add(value).expect("Entire supply fits in u64; qed");

			// Write new balances to storage
			<BalanceOf<T>>::insert(&sender, updated_from_balance);
			<BalanceOf<T>>::insert(&to, updated_to_balance);

			Self::deposit_event(Event::Transfer(sender, to, value));

			Ok(().into())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn approve(origin: OriginFor<T>, spender: T::AccountId, value: u64) -> DispatchResult {
        	let owner = ensure_signed(origin)?;

			<Allowance<T>>::insert(&owner, &spender, value);

			Self::deposit_event(Event::Transfer(owner, spender, value));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn transfer_from(origin: OriginFor<T>, owner: T::AccountId,to: T::AccountId, value: u64) -> DispatchResult {

        	let spender = ensure_signed(origin)?;

			// get the balance values
			let owner_balance = Self::balance_of(&owner);
			let to_balance = Self::balance_of(&to);

			// get the allowance value
			let approved_balance = Self::allowance(&owner, &spender);

			// Calculate new balances
			let updated_approved_balance = approved_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientApprovedFunds)?;
			let updated_owner_balance = owner_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientFunds)?;
			let updated_to_balance = to_balance.checked_add(value).expect("Entire supply fits in u64; qed");

			// Write new balances to storage
			<BalanceOf<T>>::insert(&owner, updated_owner_balance);
			<BalanceOf<T>>::insert(&to, updated_to_balance);

			// Write new allowance to storage
			<Allowance<T>>::insert(&owner, &spender, updated_approved_balance);

			Self::deposit_event(Event::Transfer(owner, to, value));

			Ok(().into())
		}
	}
}
