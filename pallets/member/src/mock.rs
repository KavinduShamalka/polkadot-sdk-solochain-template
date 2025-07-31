
use crate as pallet_member;
use frame_support::{
    derive_impl, parameter_types,
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
	// The main runtime
	#[runtime::runtime]
	// Runtime Types to be generated
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system::Pallet<Test>;

	#[runtime::pallet_index(1)]
	pub type Member = pallet_member::Pallet<Test>;

}

// Define Parameter types
parameter_types! {
    pub const MaxFirstNameLength: u32 = 50;
    pub const MaxLastNameLength: u32 = 50;
    pub const MaxEmailLength: u32 = 100;
    pub const MaxAddressLength: u32 = 200;
    pub const MaxMobileLength: u32 = 20;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

impl pallet_member::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
    type MaxFirstNameLength = MaxFirstNameLength;
    type MaxLastNameLength = MaxLastNameLength;
    type MaxEmailLength = MaxEmailLength;
    type MaxAddressLength = MaxAddressLength;
    type MaxMobileLength = MaxMobileLength;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
