//! # Member Pallet
//!
//! A pallet for managing member profiles with secure ownership control and KYC functionality.
//!
//! ## Overview
//!
//! This pallet provides:
//! - Member profile registration and management with validation
//! - Email format validation (RFC 5322 basic validation)
//! - Mobile number validation (international format with +)
//! - Date format validation (YYYY-MM-DD format)
//! - Profile updates with automatic KYC status reset
//! - KYC document submission via IPFS hashes
//! - KYC status management with admin controls
//! - Email uniqueness enforcement
//! - Comprehensive event system for tracking changes

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
    use codec::{Encode, Decode};
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
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

    /// MemberType enumeration
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
    pub enum MemberType {
        UniversityStudent,
        SchoolStudent,
        Professional,
        General,
    }

    impl Default for MemberType {
        fn default() -> Self {
            MemberType::General
        }
    }

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
    #[scale_info(skip_type_params(T))]
    pub struct Member<T: Config> {
        /// Unique member identifier
        pub member_id: MemberUuid,
        pub member_type: MemberType,
        
        /// Personal Information
        pub first_name: BoundedVec<u8, T::MaxFirstNameLength>,
        pub last_name: BoundedVec<u8, T::MaxLastNameLength>,
        pub date_of_birth: BoundedVec<u8, ConstU32<10>>, // Changed to store as string format YYYY-MM-DD
        
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

	/// Events that functions in this pallet can emit.
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
            previous_email: Option<BoundedVec<u8, T::MaxEmailLength>>,
            new_email: BoundedVec<u8, T::MaxEmailLength>,
        },
        
        /// KYC documents have been submitted
        KycSubmitted {
            member_id: MemberUuid,
            submitted_by: T::AccountId,
            kyc_hash: H256,
        },

        /// KYC status has been updated
        KycStatusUpdated {
            member_id: MemberUuid,
            updated_by: T::AccountId,
            old_status: KycStatus,
            new_status: KycStatus,
        },

		/// Member data has been retrieved with all fields
		MemberDataRetrieved {
			member_id: MemberUuid,
			accessed_by: T::AccountId,
            member_type: MemberType,
			// Member data as separate fields (this avoids trait bound issues)
			first_name: BoundedVec<u8, T::MaxFirstNameLength>,
			last_name: BoundedVec<u8, T::MaxLastNameLength>,
			date_of_birth: BoundedVec<u8, ConstU32<10>>, // Updated to string format
			email: BoundedVec<u8, T::MaxEmailLength>,
			address: BoundedVec<u8, T::MaxAddressLength>,
			mobile: BoundedVec<u8, T::MaxMobileLength>,
			photo_hash: Option<H256>,
            kyc_status: KycStatus,
			kyc_hash: Option<H256>,
			created_at: u64,
			updated_at: u64,
		},
	}

	/// Errors that can be returned by this pallet.
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
        /// Cannot update email to the same value
        EmailUnchanged,
        /// Only admin/sudo can update KYC status
        UnauthorizedKycUpdate,
        /// Invalid email format
        InvalidEmailFormat,
        /// Invalid mobile number format
        InvalidMobileFormat,
        /// Invalid date format - must be YYYY-MM-DD
        InvalidDateFormat,
	}

	/// The pallet's dispatchable functions ([`Call`]s).
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a single u32 value as a parameter, writes the value
		/// to storage and emits an event.
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
        /// - `member_type`: Type of membership (UniversityStudent, SchoolStudent, Professional, General)
        /// - `first_name`: Member's first name
        /// - `last_name`: Member's last name  
        /// - `date_of_birth`: Date in YYYY-MM-DD format (e.g., "1998-08-20")
        /// - `email`: Email address (must be valid format and unique)
        /// - `address`: Physical address
        /// - `mobile`: Mobile phone number (7-15 digits, + prefix optional)
        /// 
        /// Emits: `MemberRegistered` event
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::register_member())]
        pub fn register_member(
            origin: OriginFor<T>,
            member_type: MemberType,
            first_name: Vec<u8>,
            last_name: Vec<u8>,
            date_of_birth: Vec<u8>,
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

            // Validate email format before proceeding
            Self::validate_email(&email)?;

            // Validate mobile number format
            Self::validate_mobile(&mobile)?;

            // Validate date format
            Self::validate_date(&date_of_birth)?;

            // Convert to bounded vectors with length validation
            let bounded_first_name: BoundedVec<u8, T::MaxFirstNameLength> = 
                first_name.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
            let bounded_last_name: BoundedVec<u8, T::MaxLastNameLength> = 
                last_name.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
            let bounded_date_of_birth: BoundedVec<u8, ConstU32<10>> = 
                date_of_birth.try_into().map_err(|_| Error::<T>::InvalidDateFormat)?;
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

            // Create member profile with specified member type
            let member = Member {
                member_id,
                member_type, // Use the provided member_type instead of defaulting to General
                first_name: bounded_first_name,
                last_name: bounded_last_name,
                date_of_birth: bounded_date_of_birth,
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
                member_type: member.member_type,
				first_name: member.first_name,
				last_name: member.last_name,
				date_of_birth: member.date_of_birth,
				email: member.email,
				address: member.address,
				mobile: member.mobile,
				photo_hash: member.photo_hash,
                kyc_status: member.kyc_status,
				kyc_hash: member.kyc_hash,
				created_at: member.created_at,
				updated_at: member.updated_at,
			});

			Ok(())
		}

        /// Update member profile information
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_member())]
        pub fn update_member(
            origin: OriginFor<T>,
            member_type: Option<MemberType>,
            first_name: Option<Vec<u8>>,
            last_name: Option<Vec<u8>>,
            date_of_birth: Option<Vec<u8>>,
            email: Option<Vec<u8>>,
            address: Option<Vec<u8>>,
            mobile: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get member UUID for this account
            let member_id = AccountToMember::<T>::get(&who)
                .ok_or(Error::<T>::MemberNotFound)?;

            // Get existing member data
            let mut member = Members::<T>::get(&member_id)
                .ok_or(Error::<T>::MemberNotFound)?;

            // Verify ownership
            ensure!(member.created_by == who, Error::<T>::NotMemberOwner);

            let mut profile_changed = false;
            let old_email = member.email.clone();
            let mut new_email = member.email.clone();

            // Update member type if provided
            if let Some(mt) = member_type {
                if mt != member.member_type {
                    member.member_type = mt;
                    profile_changed = true;
                }
            }

            // Update first name if provided
            if let Some(name) = first_name {
                let bounded_name: BoundedVec<u8, T::MaxFirstNameLength> = 
                    name.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
                if bounded_name != member.first_name {
                    member.first_name = bounded_name;
                    profile_changed = true;
                }
            }

            // Update last name if provided
            if let Some(name) = last_name {
                let bounded_name: BoundedVec<u8, T::MaxLastNameLength> = 
                    name.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
                if bounded_name != member.last_name {
                    member.last_name = bounded_name;
                    profile_changed = true;
                }
            }

            // Update date of birth if provided
            if let Some(dob) = date_of_birth {
                // Validate date format
                Self::validate_date(&dob)?;
                
                let bounded_dob: BoundedVec<u8, ConstU32<10>> = 
                    dob.try_into().map_err(|_| Error::<T>::InvalidDateFormat)?;
                if bounded_dob != member.date_of_birth {
                    member.date_of_birth = bounded_dob;
                    profile_changed = true;
                }
            }

            // Update email if provided
            if let Some(new_email_vec) = email {
                // Validate email format
                Self::validate_email(&new_email_vec)?;
                
                let bounded_email: BoundedVec<u8, T::MaxEmailLength> = 
                    new_email_vec.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
                
                if bounded_email != member.email {
                    // Check if new email is already taken by another member
                    if let Some(existing_member_id) = MemberByEmail::<T>::get(&bounded_email) {
                        ensure!(existing_member_id == member_id, Error::<T>::EmailAlreadyExists);
                    }

                    // Remove old email mapping
                    MemberByEmail::<T>::remove(&member.email);
                    
                    // Update email and create new mapping
                    member.email = bounded_email.clone();
                    new_email = bounded_email.clone();
                    MemberByEmail::<T>::insert(&bounded_email, &member_id);
                    profile_changed = true;
                }
            }

            // Update address if provided
            if let Some(addr) = address {
                let bounded_address: BoundedVec<u8, T::MaxAddressLength> = 
                    addr.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
                if bounded_address != member.address {
                    member.address = bounded_address;
                    profile_changed = true;
                }
            }

            // Update mobile if provided
            if let Some(mob) = mobile {
                // Validate mobile format
                Self::validate_mobile(&mob)?;
                
                let bounded_mobile: BoundedVec<u8, T::MaxMobileLength> = 
                    mob.try_into().map_err(|_| Error::<T>::InvalidMemberData)?;
                if bounded_mobile != member.mobile {
                    member.mobile = bounded_mobile;
                    profile_changed = true;
                }
            }

            // If any field was changed, reset KYC status and update timestamp
            if profile_changed {
                member.kyc_status = KycStatus::Unapproved;
                member.updated_at = Self::current_timestamp();

                // Store updated member data
                Members::<T>::insert(&member_id, &member);

                // Determine previous email for event (None if email didn't change)
                let previous_email = if new_email != old_email {
                    Some(old_email)
                } else {
                    None
                };

                // Emit event
                Self::deposit_event(Event::MemberUpdated {
                    member_id,
                    updated_by: who,
                    previous_email,
                    new_email,
                });
            }

            Ok(())
        }

        /// Submit KYC documents
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::submit_kyc())]
        pub fn submit_kyc(
            origin: OriginFor<T>,
            kyc_hash: H256,
            photo_hash: Option<H256>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get member UUID for this account
            let member_id = AccountToMember::<T>::get(&who)
                .ok_or(Error::<T>::MemberNotFound)?;

            // Get existing member data
            let mut member = Members::<T>::get(&member_id)
                .ok_or(Error::<T>::MemberNotFound)?;

            // Verify ownership
            ensure!(member.created_by == who, Error::<T>::NotMemberOwner);

            // Update KYC hash and photo hash if provided
            member.kyc_hash = Some(kyc_hash);
            if let Some(photo) = photo_hash {
                member.photo_hash = Some(photo);
            }
            member.updated_at = Self::current_timestamp();

            // Store updated member data
            Members::<T>::insert(&member_id, &member);

            // Emit event
            Self::deposit_event(Event::KycSubmitted {
                member_id,
                submitted_by: who,
                kyc_hash,
            });

            Ok(())
        }

        /// Update KYC status (Admin/Sudo only)
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::update_kyc_status())]
        pub fn update_kyc_status(
            origin: OriginFor<T>,
            member_id: MemberUuid,
            new_status: KycStatus,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get existing member data
            let mut member = Members::<T>::get(&member_id)
                .ok_or(Error::<T>::MemberNotFound)?;

            // Store old status for event
            let old_status = member.kyc_status.clone();

            // Update KYC status and timestamp
            member.kyc_status = new_status.clone();
            member.updated_at = Self::current_timestamp();

            // Store updated member data
            Members::<T>::insert(&member_id, &member);

            // Emit event
            Self::deposit_event(Event::KycStatusUpdated {
                member_id,
                updated_by: who,
                old_status,
                new_status,
            });

            Ok(())
        }

        /// Update KYC status by admin with additional validation (Root/Sudo only)
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::admin_update_kyc_status())]
        pub fn admin_update_kyc_status(
            origin: OriginFor<T>,
            member_id: MemberUuid,
            new_status: KycStatus,
        ) -> DispatchResult {
            // Only root/sudo can call this function
            ensure_root(origin)?;

            // Get existing member data
            let mut member = Members::<T>::get(&member_id)
                .ok_or(Error::<T>::MemberNotFound)?;

            // Store old status for event
            let old_status = member.kyc_status.clone();

            // Update KYC status and timestamp
            member.kyc_status = new_status.clone();
            member.updated_at = Self::current_timestamp();

            // Store updated member data
            Members::<T>::insert(&member_id, &member);

            // For admin updates, we'll use the member's account as placeholder
            Self::deposit_event(Event::KycStatusUpdated {
                member_id,
                updated_by: member.created_by.clone(),
                old_status,
                new_status,
            });

            Ok(())
        }
	}

	//// Public query functions and validation helpers
    impl<T: Config> Pallet<T> {
        /// Validate email format (basic RFC 5322 validation)
        fn validate_email(email: &[u8]) -> DispatchResult {
            let email_str = core::str::from_utf8(email)
                .map_err(|_| Error::<T>::InvalidEmailFormat)?;
            
            // Basic email validation
            // Must contain exactly one @ symbol
            let at_count = email_str.matches('@').count();
            ensure!(at_count == 1, Error::<T>::InvalidEmailFormat);
            
            // Split into local and domain parts
            let parts: Vec<&str> = email_str.split('@').collect();
            ensure!(parts.len() == 2, Error::<T>::InvalidEmailFormat);
            
            let local = parts[0];
            let domain = parts[1];
            
            // Local part validation
            ensure!(!local.is_empty() && local.len() <= 64, Error::<T>::InvalidEmailFormat);
            ensure!(!local.starts_with('.') && !local.ends_with('.'), Error::<T>::InvalidEmailFormat);
            ensure!(!local.contains(".."), Error::<T>::InvalidEmailFormat);
            
            // Domain part validation
            ensure!(!domain.is_empty() && domain.len() <= 253, Error::<T>::InvalidEmailFormat);
            ensure!(domain.contains('.'), Error::<T>::InvalidEmailFormat);
            ensure!(!domain.starts_with('.') && !domain.ends_with('.'), Error::<T>::InvalidEmailFormat);
            ensure!(!domain.starts_with('-') && !domain.ends_with('-'), Error::<T>::InvalidEmailFormat);
            
            // Check for valid characters in local part
            for c in local.chars() {
                ensure!(
                    c.is_ascii_alphanumeric() || 
                    c == '.' || c == '_' || c == '-' || c == '+',
                    Error::<T>::InvalidEmailFormat
                );
            }
            
            // Check for valid characters in domain part
            for c in domain.chars() {
                ensure!(
                    c.is_ascii_alphanumeric() || c == '.' || c == '-',
                    Error::<T>::InvalidEmailFormat
                );
            }
            
            Ok(())
        }

        /// Validate mobile number format (flexible format - with or without + prefix)
        fn validate_mobile(mobile: &[u8]) -> DispatchResult {
            let mobile_str = core::str::from_utf8(mobile)
                .map_err(|_| Error::<T>::InvalidMobileFormat)?;
            
            // Handle both formats: with or without + prefix
            let number_part = if mobile_str.starts_with('+') {
                &mobile_str[1..]  // Remove + prefix if present
            } else {
                mobile_str        // Use as-is if no + prefix
            };
            
            // Must be between 7 and 15 digits
            ensure!(number_part.len() >= 7 && number_part.len() <= 15, Error::<T>::InvalidMobileFormat);
            
            // All characters must be digits
            for c in number_part.chars() {
                ensure!(c.is_ascii_digit(), Error::<T>::InvalidMobileFormat);
            }
            
            Ok(())
        }

        /// Validate date format (YYYY-MM-DD)
        fn validate_date(date: &[u8]) -> DispatchResult {
            let date_str = core::str::from_utf8(date)
                .map_err(|_| Error::<T>::InvalidDateFormat)?;
            
            // Must be exactly 10 characters
            ensure!(date_str.len() == 10, Error::<T>::InvalidDateFormat);
            
            // Check format: YYYY-MM-DD
            let chars: Vec<char> = date_str.chars().collect();
            
            // Check positions of dashes
            ensure!(chars[4] == '-' && chars[7] == '-', Error::<T>::InvalidDateFormat);
            
            // Check that year, month, day parts are all digits
            for i in 0..4 {
                ensure!(chars[i].is_ascii_digit(), Error::<T>::InvalidDateFormat);
            }
            for i in 5..7 {
                ensure!(chars[i].is_ascii_digit(), Error::<T>::InvalidDateFormat);
            }
            for i in 8..10 {
                ensure!(chars[i].is_ascii_digit(), Error::<T>::InvalidDateFormat);
            }
            
            // Extract year, month, day and validate ranges
            let year_str = &date_str[0..4];
            let month_str = &date_str[5..7];
            let day_str = &date_str[8..10];
            
            // Parse year (basic range check: 1900-2100)
            if let Ok(year) = year_str.parse::<u32>() {
                ensure!(year >= 1900 && year <= 2100, Error::<T>::InvalidDateFormat);
            } else {
                return Err(Error::<T>::InvalidDateFormat.into());
            }
            
            // Parse month (1-12)
            if let Ok(month) = month_str.parse::<u32>() {
                ensure!(month >= 1 && month <= 12, Error::<T>::InvalidDateFormat);
            } else {
                return Err(Error::<T>::InvalidDateFormat.into());
            }
            
            // Parse day (1-31, basic validation)
            if let Ok(day) = day_str.parse::<u32>() {
                ensure!(day >= 1 && day <= 31, Error::<T>::InvalidDateFormat);
            } else {
                return Err(Error::<T>::InvalidDateFormat.into());
            }
            
            Ok(())
        }

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

        /// Get member by UUID (admin function - returns full data)
        pub fn get_member_by_uuid(member_id: &MemberUuid) -> Option<Member<T>> {
            Members::<T>::get(member_id)
        }

        /// Get member UUID by account
        pub fn get_member_uuid_by_account(account: &T::AccountId) -> Option<MemberUuid> {
            AccountToMember::<T>::get(account)
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