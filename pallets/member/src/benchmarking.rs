//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn do_something() {
		let value = 100u32;
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		do_something(RawOrigin::Signed(caller), value);

		assert_eq!(Something::<T>::get(), Some(value));
	}

	#[benchmark]
	fn cause_error() {
		Something::<T>::put(100u32);
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		cause_error(RawOrigin::Signed(caller));

		assert_eq!(Something::<T>::get(), Some(101u32));
	}

	    #[benchmark]
    fn register_member() {
        let caller: T::AccountId = whitelisted_caller();
        let first_name = b"John".to_vec();
        let last_name = b"Doe".to_vec();
        let date_of_birth = 946684800u64;
        let email = b"john.doe@example.com".to_vec();
        let address = b"123 Main St, Anytown, USA".to_vec();
        let mobile = b"+1234567890".to_vec();

        #[extrinsic_call]
        register_member(
            RawOrigin::Signed(caller.clone()),
            first_name,
            last_name,
            date_of_birth,
            email.clone(),
            address,
            mobile,
        );

        // Verify member was registered
        assert!(Member::<T>::has_member_profile(&caller));
    }

    #[benchmark]
    fn get_member() {
        let caller: T::AccountId = whitelisted_caller();
        
        // Setup: Register a member first
        let _ = Member::<T>::register_member(
            RawOrigin::Signed(caller.clone()).into(),
            b"John".to_vec(),
            b"Doe".to_vec(),
            946684800u64,
            b"john.doe@example.com".to_vec(),
            b"123 Main St".to_vec(),
            b"+1234567890".to_vec(),
        );

        #[extrinsic_call]
        get_member(RawOrigin::Signed(caller.clone()));

        // Verify member data was accessed (event should be emitted)
        // The actual verification would check the event in a real scenario
    }

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
