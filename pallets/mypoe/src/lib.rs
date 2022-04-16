#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	/// key: (owner, blockNumber)
	pub type Proofs<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber)>;

	#[pallet::event]
	///#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransferred(T::AccountId, T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ProofNotExist,
		NotProofOwner,
		SelfTransferNotAllowed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// use the claim as key, and set the owner fields and blockNumber accordingly, and store to database.
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);
			Proofs::<T>::insert(&claim, (who.clone(), frame_system::Pallet::<T>::block_number()));
			Self::deposit_event(Event::ClaimCreated(who, claim));
			Ok(())
		}

		/// try find the claim, check owner ship with owner and remove it from database.
		#[pallet::weight(0)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ProofNotExist)?;
			ensure!(owner == who, Error::<T>::NotProofOwner);
			Proofs::<T>::remove(&claim);
			Self::deposit_event(Event::ClaimRevoked(who, claim));
			Ok(())
		}

		/// try find the claim, check owner, mutate the owner filed of the data to be receiver.
		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			receiver: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ProofNotExist)?;
			ensure!(owner == who, Error::<T>::NotProofOwner);
			ensure!(who != receiver, Error::<T>::SelfTransferNotAllowed);
			Proofs::<T>::try_mutate(&claim, |proof| {
				if let Some(value) = proof {
					value.0 = receiver.clone();
					return Ok(());
				}
				Err(Error::<T>::ProofNotExist)
			})
			.map_err(|_| Error::<T>::ProofNotExist)?;
			Self::deposit_event(Event::ClaimTransferred(who, receiver, claim));
			Ok(())
		}
	}
}
