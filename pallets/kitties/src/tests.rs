use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_a_kitty_works_when_have_enough_balance() {
    new_test_ext().execute_with(|| {
        // count 2 balance 20, which s/b enough
        assert_ok!(KittiesModule::create_kitty(Origin::signed(2)),);
    });
}

#[test]
fn create_a_kitty_fails_when_balance_not_enough() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            KittiesModule::create_kitty(Origin::signed(1)),
            Error::<Test>::NotEnoughBalance
        );
    });
}

#[test]
fn breed_a_kitty_works_when_balance_enough() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(8)),);
        assert_ok!(KittiesModule::create_kitty(Origin::signed(8)),);

        assert_ok!(KittiesModule::breed(Origin::signed(8), 1, 2),);
    });
}

#[test]
fn breed_a_kitty_fails_when_balance_not_enough() {
    new_test_ext().execute_with(|| {
        // count 4 have enough balance to create 2 kitty, but not enough to bread a new one after then.
        assert_ok!(KittiesModule::create_kitty(Origin::signed(4)),);
        assert_ok!(KittiesModule::create_kitty(Origin::signed(4)),);

        assert_noop!(
            KittiesModule::breed(Origin::signed(4), 1, 2),
            Error::<Test>::NotEnoughBalance
        );
    });
}

#[test]
fn breed_a_kitty_fails_when_parent_invalid() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(4)),);
        assert_ok!(KittiesModule::create_kitty(Origin::signed(4)),);

        assert_noop!(
            KittiesModule::breed(Origin::signed(4), 1, 22),
            Error::<Test>::InvalidKittyIndex
        );
    });
}

#[test]
fn breed_a_kitty_fails_when_parents_are_same() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(4)),);
        assert_noop!(
            KittiesModule::breed(Origin::signed(4), 1, 1),
            Error::<Test>::SameParentIndex
        );
    });
}

#[test]
fn transfer_a_kitty_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_ok!(KittiesModule::transfer(Origin::signed(3), 1, 1));
        assert_eq!(Owner::<Test>::get(1), Some(1));
    });
}

#[test]
fn transfer_a_kitty_fails_when_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_noop!(
            KittiesModule::transfer(Origin::signed(2), 1, 1),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn sell_a_kitty_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_ok!(KittiesModule::sell_kitty(Origin::signed(3), 1, Some(20)));
    });
}

#[test]
fn sell_a_kitty_fails_when_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_noop!(
            KittiesModule::sell_kitty(Origin::signed(1), 1, Some(20)),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn sell_a_kitty_fails_when_kitty_index_invalid() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_noop!(
            KittiesModule::sell_kitty(Origin::signed(3), 121, Some(20)),
            Error::<Test>::InvalidKittyIndex
        );
    });
}

#[test]
fn buy_a_kitty_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_ok!(KittiesModule::sell_kitty(Origin::signed(3), 1, Some(9)));
        assert_ok!(KittiesModule::buy_kitty(Origin::signed(1), 1));
        assert_eq!(Owner::<Test>::get(1), Some(1));
    });
}

#[test]
fn buy_a_kitty_fails_when_buyer_is_kitty_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_ok!(KittiesModule::sell_kitty(Origin::signed(3), 1, Some(9)));
        assert_noop!(
            KittiesModule::buy_kitty(Origin::signed(3), 1),
            Error::<Test>::BuyerIsKittyOwner
        );
    });
}

#[test]
fn buy_a_kitty_fails_when_not_enough_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_ok!(KittiesModule::sell_kitty(Origin::signed(3), 1, Some(19)));
        assert_noop!(
            KittiesModule::buy_kitty(Origin::signed(1), 1),
            Error::<Test>::NotEnoughBalance
        );
    });
}

#[test]
fn buy_a_kitty_fails_when_not_for_sell() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(3)),);
        assert_noop!(
            KittiesModule::buy_kitty(Origin::signed(1), 1),
            Error::<Test>::KittyNotForSale
        );
    });
}

#[test]
fn buy_a_kitty_fails_when_kitty_not_exist() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            KittiesModule::buy_kitty(Origin::signed(1), 1),
            Error::<Test>::InvalidKittyIndex
        );
    });
}
