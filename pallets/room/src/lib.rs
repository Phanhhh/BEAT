#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::SaturatedConversion;
use frame_support::traits::Time;
use frame_system::pallet_prelude::*;

pub type Id = u32;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[derive(Debug, Decode, Encode, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Room<T: Config> {
		creator: T::AccountId,
		id_room: Id,
		num_days: u64,
		end_date: u64,
		total_value: u128,
		is_started: bool,
		is_ended: bool,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type RoomTime: Time;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(crate) type RoomId<T> = StorageValue<_, u32, ValueQuery>;

	// #[pallet::storage]
	// pub(crate) type NumberUser<T> = StorageValue<_, u32, ValueQuery>;

	// Room index and information of room
	#[pallet::storage]
	#[pallet::getter(fn get_room)]
	pub(super) type Rooms<T: Config> = StorageMap<_, Blake2_128Concat, u32, Room<T>, OptionQuery>;

	// Users who have joined in room (id_room, balance deposit)
	#[pallet::storage]
	#[pallet::getter(fn get_user_in_room)]
	pub(super) type UserInRoom<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, Vec<T::AccountId>, OptionQuery>;

	// Value which users have deposited
	#[pallet::storage]
	#[pallet::getter(fn get_deposit)]
	pub(super) type UserDeposit<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u128, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Create new room (id_room, who)
		CreateRoom { id_room: u32, creator: T::AccountId },
		/// Who join in room (id_room, who, amount deposited)
		JoinRoom { id_room: u32, user: T::AccountId, deposit_amount: u128 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Room doesn't exist
		RoomNotExist,
		/// Maximum participants
		RoomOverload,
		/// Room started already
		RoomAlreadyStarted,
		/// Room ended already
		RoomAlreadyEnded,
		/// User has already joined
		UserAlreadyJoined,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_new_room(
			origin: OriginFor<T>,
			num_days: u64,
			deposit: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Update room id
			let mut current_id_room = <RoomId<T>>::get();
			current_id_room += 1;
			<RoomId<T>>::put(current_id_room);

			// let start = T::RoomTime::now().saturated_into::<u64>();

			// let end = start + num_days * 86_400;

			// let mut current_number = <NumberUser<T>>::get();
			// current_number += 1;
			// <NumberUser<T>>::put(current_number);

			// let mut users_joined = UserInRoom::<T>::take(&who).unwrap_or_default();
			// users_joined.push(who.clone());

			let room = Room::<T> {
				creator: who.clone(),
				id_room: current_id_room,
				num_days,
				end_date: 0,
				total_value: deposit,
				is_started: false,
				is_ended: false,
			};

			let mut vec_users = Vec::new();
			vec_users.push(who.clone());

			<UserInRoom<T>>::insert(current_id_room, vec_users);
			<UserDeposit<T>>::insert(who.clone(), deposit);
			<Rooms<T>>::insert(current_id_room, room);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn join_room(origin: OriginFor<T>, id_room: u32, deposit: u128) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if the room exists or not
			ensure!(id_room <= <RoomId<T>>::get(), Error::<T>::RoomNotExist);

			// Check if the user has not already join any room
			ensure!(!UserDeposit::<T>::contains_key(&who), Error::<T>::UserAlreadyJoined);

			let mut room = Rooms::<T>::get(id_room).unwrap();

			// Check if the room has started/ ended yet
			ensure!(!room.is_started, Error::<T>::RoomAlreadyStarted);
			ensure!(!room.is_ended, Error::<T>::RoomAlreadyEnded);

			// Update infomation
			let mut vec_users = UserInRoom::<T>::get(id_room).unwrap();
			vec_users.push(who.clone());

			<UserInRoom<T>>::insert(id_room, vec_users.clone());
			<UserDeposit<T>>::insert(who.clone(), deposit);

			// If the number of users is 4, room will start
			if (vec_users.len() as i32) == 4 {
				room.is_started = true;
				let end = T::RoomTime::now().saturated_into::<u64>() + room.num_days * 86_400_000;
				room.end_date = end;
				<Rooms<T>>::insert(id_room, room);
			}

			Ok(())
		}
	}
}