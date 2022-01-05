use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_create_a_kitty() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_noop!(KittiesModule::create_kitty(Origin::signed(1)), Error::<Test>::NotEnoughBalance);
        // Read pallet storage and assert an expected result.
    });
}


