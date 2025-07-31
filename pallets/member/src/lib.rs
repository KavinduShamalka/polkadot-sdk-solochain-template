//! # Template Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single `u32` value
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! Learn more about FRAME macros [here](https://docs.substrate.io/reference/frame-macros/).
//!
//! ### Pallet Sections
//!
//! The pallet sections in this template are:
//!
//! - A **configuration trait** that defines the types and parameters which the pallet depends on
//!   (denoted by the `#[pallet::config]` attribute). See: [`Config`].
//! - A **means to store pallet-specific data** (denoted by the `#[pallet::storage]` attribute).
//!   See: [`storage_types`].
//! - A **declaration of the events** this pallet emits (denoted by the `#[pallet::event]`
//!   attribute). See: [`Event`].
//! - A **declaration of the errors** that this pallet can throw (denoted by the `#[pallet::error]`
//!   attribute). See: [`Error`].
//! - A **set of dispatchable functions** that define the pallet's functionality (denoted by the
//!   `#[pallet::call]` attribute). See: [`dispatchables`].
//!
//! Run `cargo doc --package pallet-template --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::Get,
    };
    use frame_system::pallet_prelude::*;
    use codec::Encode;
    use frame_support::sp_runtime::SaturatedConversion;
    use scale_info::prelude::vec::Vec;
	use sp_core::H256;

	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;

		/// Maximum length allowed for first name
        #[pallet::constant]
        type MaxFirstNameLength: Get<u32>;

		/// Maximum length allowed for last name
        #[pallet::constant]
        type MaxLastNameLength: Get<u32>;
        
        /// Maximum length allowed for email
        #[pallet::constant]
        type MaxEmailLength: Get<u32>;
        
        /// Maximum length allowed for address
        #[pallet::constant]
        type MaxAddressLength: Get<u32>;
        
        /// Maximum length allowed for mobile number
        #[pallet::constant]
        type MaxMobileLength: Get<u32>;
	}

	/// Member UUID type - using H256 for 32-byte unique identifier
    pub type MemberUuid = H256;

    /// KYC Status enumeration
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum KycStatus {
        Unapproved,
        Approved,
        Rejected,
    }

    impl Default for KycStatus {
        fn default() -> Self {
            KycStatus::Unapproved
        }
    }

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Member<T: Config> {
        /// Unique member identifier
        pub member_id: MemberUuid,
        
        /// Personal Information
        pub first_name: BoundedVec<u8, T::MaxFirstNameLength>,
        pub last_name: BoundedVec<u8, T::MaxLastNameLength>,
        pub date_of_birth: u64, // Unix timestamp
        
        /// Contact Information
        pub email: BoundedVec<u8, T::MaxEmailLength>,
        pub address: BoundedVec<u8, T::MaxAddressLength>,
        pub mobile: BoundedVec<u8, T::MaxMobileLength>,
        
        /// KYC & Verification
        pub kyc_status: KycStatus,
        
        /// File References (IPFS hashes)
        pub photo_hash: Option<H256>,
        pub kyc_hash: Option<H256>,
        
        /// Metadata
        pub created_at: u64, // Block timestamp
        pub updated_at: u64, // Block timestamp
        pub created_by: T::AccountId, // Account that created this member
    }

	/// A storage item for this pallet.
	///
	/// In this template, we are declaring a storage item called `Something` that stores a single
	/// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
	#[pallet::storage]
	pub type Something<T> = StorageValue<_, u32>;

	/// Main storage for member profiles
    /// Key: MemberUuid → Value: Member profile data
    #[pallet::storage]
    pub type Members<T: Config> = StorageMap<
        _, Blake2_128Concat, MemberUuid, Member<T>, OptionQuery
    >;

    /// Maps account addresses to their owned member UUIDs
    /// Key: AccountId → Value: MemberUuid
    #[pallet::storage]
    pub type AccountToMember<T: Config> = StorageMap<
        _, Blake2_128Concat, T::AccountId, MemberUuid, OptionQuery
    >;

    /// Email uniqueness index
    /// Key: Email → Value: MemberUuid
    #[pallet::storage]
    pub type MemberByEmail<T: Config> = StorageMap<
        _, Blake2_128Concat, BoundedVec<u8, T::MaxEmailLength>, MemberUuid, OptionQuery
    >;

    /// Total count of registered members
    #[pallet::storage]
    pub type MemberCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Member index for iteration (registration order)
    /// Key: Index → Value: MemberUuid
    #[pallet::storage]
    pub type MemberByIndex<T: Config> = StorageMap<
        _, Blake2_128Concat, u32, MemberUuid, OptionQuery
    >;


	// First, create a return type for Member data (without the created_by field for privacy)
	// Add this BEFORE your Event enum:

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct MemberInfo<T: Config> {
		pub member_id: MemberUuid,
		pub first_name: BoundedVec<u8, T::MaxFirstNameLength>,
		pub last_name: BoundedVec<u8, T::MaxLastNameLength>,
		pub date_of_birth: u64,
		pub email: BoundedVec<u8, T::MaxEmailLength>,
		pub address: BoundedVec<u8, T::MaxAddressLength>,
		pub mobile: BoundedVec<u8, T::MaxMobileLength>,
		pub kyc_status: KycStatus,
		pub photo_hash: Option<H256>,
		pub kyc_hash: Option<H256>,
		pub created_at: u64,
		pub updated_at: u64,
	}
	/// Events that functions in this pallet can emit.
	///
	/// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
	/// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
	/// documentation for each event field and its parameters is added to a node's metadata so it
	/// can be used by external interfaces or tools.
	///
	///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
	/// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
	/// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has successfully set a new value.
		SomethingStored {
			/// The new value set.
			something: u32,
			/// The account who set the new value.
			who: T::AccountId,
		},

		/// A new member has been registered
        MemberRegistered {
            member_id: MemberUuid,
            account: T::AccountId,
            email: BoundedVec<u8, T::MaxEmailLength>,
        },
        
        /// Member information has been updated
        MemberUpdated {
            member_id: MemberUuid,
            updated_by: T::AccountId,
        },
        
        /// KYC documents have been submitted
        KycSubmitted {
            member_id: MemberUuid,
            submitted_by: T::AccountId,
        },

		// Add this event to your Event enum:
		/// Member data has been retrieved
		/// Member data has been retrieved with all fields
		MemberDataRetrieved {
			member_id: MemberUuid,
			accessed_by: T::AccountId,
			// Member data as separate fields (this avoids trait bound issues)
			first_name: BoundedVec<u8, T::MaxFirstNameLength>,
			last_name: BoundedVec<u8, T::MaxLastNameLength>,
			date_of_birth: u64,
			email: BoundedVec<u8, T::MaxEmailLength>,
			address: BoundedVec<u8, T::MaxAddressLength>,
			mobile: BoundedVec<u8, T::MaxMobileLength>,
			photo_hash: Option<H256>,
			kyc_hash: Option<H256>,
			created_at: u64,
			updated_at: u64,
		},
	}

	/// Errors that can be returned by this pallet.
	///
	/// Errors tell users that something went wrong so it's important that their naming is
	/// informative. Similar to events, error documentation is added to a node's metadata so it's
	/// equally important that they have helpful documentation associated with them.
	///
	/// This type of runtime error can be up to 4 bytes in size should you want to return additional
	/// information.
	#[pallet::error]
	pub enum Error<T> {
		/// The value retrieved was `None` as no value was previously set.
		NoneValue,
		/// There was an attempt to increment the value in storage over `u32::MAX`.
		StorageOverflow,
		/// Member profile not found
        MemberNotFound,
        /// Account already has a member profile
        MemberAlreadyExists,
        /// Email address is already registered
        EmailAlreadyExists,
        /// Account does not own this member profile
        NotMemberOwner,
        /// Invalid member data provided
        InvalidMemberData,
        /// Member profile access denied
        AccessDenied,
        /// KYC documents not found
        KycNotFound,
        /// Invalid KYC status transition
        InvalidKycStatusTransition,
	}

	/// The pallet's dispatchable functions ([`Call`]s).
	///
	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// They must always return a `DispatchResult` and be annotated with a weight and call index.
	///
	/// The [`call_index`] macro is used to explicitly
	/// define an index for calls in the [`Call`] enum. This is useful for pallets that may
	/// introduce new dispatchables over time. If the order of a dispatchable changes, its index
	/// will also change which will break backwards compatibility.
	///
	/// The [`weight`] macro is used to assign a weight to each call.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a single u32 value as a parameter, writes the value
		/// to storage and emits an event.
		///
		/// It checks that the _origin_ for this call is _Signed_ and returns a dispatch
		/// error if it isn't. Learn more about origins here: <https://docs.substrate.io/build/origins/>
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::<T>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });

			// Return a successful `DispatchResult`
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		///
		/// It checks that the caller is a signed origin and reads the current value from the
		/// `Something` storage item. If a current value exists, it is incremented by 1 and then
		/// written back to storage.
		///
		/// ## Errors
		///
		/// The function will return an error under the following conditions:
		///
		/// - If no value has been set ([`Error::NoneValue`])
		/// - If incrementing the value in storage causes an arithmetic overflow
		///   ([`Error::StorageOverflow`])
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::<T>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage. This will cause an error in the event
					// of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::<T>::put(new);
					Ok(())
				},
			}
		}

		/// Register a new member profile
        /// 
        /// This function creates a new member profile owned by the calling account.
        /// Each account can only have one member profile.
        /// 
        /// Parameters:
        /// - `first_name`: Member's first name
        /// - `last_name`: Member's last name  
        /// - `date_of_birth`: Unix timestamp of birth date
        /// - `email`: Email address (must be unique)
        /// - `address`: Physical address
        /// - `mobile`: Mobile phone number
        /// 
        /// Emits: `MemberRegistered` event
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::register_member())]
        pub fn register_member(
            origin: OriginFor<T>,
            first_name: Vec<u8>,
            last_name: Vec<u8>,
            date_of_birth: u64,
            email: Vec<u8>,
            address: Vec<u8>,
            mobile: Vec<u8>,
        ) -> DispatchResult {
            // Verify the extrinsic is signed and get the signer's account
            let who = ensure_signed(origin)?;

            // Check if account already has a member profile
            ensure!(
                !AccountToMember::<T>::contains_key(&who),
                Error::<T>::MemberAlreadyExists
            );

            // Convert to bounded vectors with length validation
            let bounded_first_name: BoundedVec<u8, T::MaxFirstNameLength> = 
                first_name.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
            let bounded_last_name: BoundedVec<u8, T::MaxLastNameLength> = 
                last_name.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
            let bounded_email: BoundedVec<u8, T::MaxEmailLength> = 
                email.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
            let bounded_address: BoundedVec<u8, T::MaxAddressLength> = 
                address.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
            let bounded_mobile: BoundedVec<u8, T::MaxMobileLength> = 
                mobile.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;

            // Check email uniqueness
            ensure!(
                !MemberByEmail::<T>::contains_key(&bounded_email),
                Error::<T>::EmailAlreadyExists
            );

            // Generate unique member UUID using account and current timestamp
            let current_time = Self::current_timestamp();
            let member_id = Self::generate_member_uuid(&who, current_time);

            // Create member profile
            let member = Member {
                member_id,
                first_name: bounded_first_name,
                last_name: bounded_last_name,
                date_of_birth,
                email: bounded_email.clone(),
                address: bounded_address,
                mobile: bounded_mobile,
                kyc_status: KycStatus::Unapproved,
                photo_hash: None,
                kyc_hash: None,
                created_at: current_time,
                updated_at: current_time,
                created_by: who.clone(),
            };

            // Get current member count for indexing
            let member_index = MemberCount::<T>::get();

            // Store member data
            Members::<T>::insert(&member_id, &member);
            AccountToMember::<T>::insert(&who, &member_id);
            MemberByEmail::<T>::insert(&bounded_email, &member_id);
            MemberByIndex::<T>::insert(member_index, &member_id);
            
            // Increment member count
            MemberCount::<T>::put(member_index.saturating_add(1));

            // Emit event
            Self::deposit_event(Event::MemberRegistered {
                member_id,
                account: who,
                email: bounded_email,
            });

            Ok(())
        }

		/// Get member profile information with full data
		/// 
		/// Returns the complete member profile data in the event fields.
		/// Only the owner can access their own member data.
		/// 
		/// Returns: Success confirmation
		/// Data: Available in MemberDataRetrieved event fields
		/// 
		/// Emits: `MemberDataRetrieved` event with all member fields
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::get_member())]
		pub fn get_member(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get member UUID for this account
			let member_id = AccountToMember::<T>::get(&who)
				.ok_or(Error::<T>::MemberNotFound)?;

			// Get member data
			let member = Members::<T>::get(&member_id)
				.ok_or(Error::<T>::MemberNotFound)?;

			// Verify ownership - only allow access if the account owns the profile
			ensure!(member.created_by == who, Error::<T>::NotMemberOwner);

			// Emit event with all member data as separate fields
			Self::deposit_event(Event::MemberDataRetrieved {
				member_id,
				accessed_by: who,
				first_name: member.first_name,
				last_name: member.last_name,
				date_of_birth: member.date_of_birth,
				email: member.email,
				address: member.address,
				mobile: member.mobile,
				photo_hash: member.photo_hash,
				kyc_hash: member.kyc_hash,
				created_at: member.created_at,
				updated_at: member.updated_at,
			});

			Ok(())
		}

	}


	/// Public query functions (not extrinsics)
    impl<T: Config> Pallet<T> {
        /// Get member profile by account (only returns data if caller owns the profile)
        pub fn get_member_by_account(account: &T::AccountId) -> Option<Member<T>> {
            // Get member UUID for this account
            let member_id = AccountToMember::<T>::get(account)?;
            
            // Get member data
            let member = Members::<T>::get(&member_id)?;
            
            // Verify ownership - only return data if the account owns the profile
            if *account == member.created_by {
                Some(member)
            } else {
                None
            }
        }

        /// Check if an account has a member profile
        pub fn has_member_profile(account: &T::AccountId) -> bool {
            AccountToMember::<T>::contains_key(account)
        }

        /// Check if an email is already registered
        pub fn is_email_registered(email: &BoundedVec<u8, T::MaxEmailLength>) -> bool {
            MemberByEmail::<T>::contains_key(email)
        }

        /// Get total number of registered members
        pub fn total_members() -> u32 {
            MemberCount::<T>::get()
        }

        /// Helper function to generate unique member UUID
        fn generate_member_uuid(account: &T::AccountId, timestamp: u64) -> MemberUuid {
            use sp_runtime::traits::{BlakeTwo256, Hash};
            
            let mut data = Vec::new();
            data.extend_from_slice(&account.encode());
            data.extend_from_slice(&timestamp.to_le_bytes());
            
            BlakeTwo256::hash(&data)
        }

        /// Helper function to get current timestamp
        fn current_timestamp() -> u64 {
            // In a real implementation, you would get this from pallet_timestamp
            // For now, using block number as a simple timestamp
            <frame_system::Pallet<T>>::block_number().saturated_into::<u64>()
        }
    }
}
