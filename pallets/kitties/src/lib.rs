#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{tokens::ExistenceRequirement, Currency, Randomness, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_128;

    type KittyIndex = u32;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
    pub struct Kitty<T: Config> {
        pub dna: [u8; 16],
        // the price of the kitty, if the price is None, means this kitty is not for sales yet.
        pub price: Option<BalanceOf<T>>,
    }

    /// Configure the pallet by specifying the parameters and types it depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        // the amount of token to deposit when mint a kitty
        #[pallet::constant]
        type MintDeposit: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated(T::AccountId, KittyIndex),
        KittyTransfered(T::AccountId, T::AccountId, KittyIndex),
        PriceUpdateForSale(T::AccountId, KittyIndex, Option<BalanceOf<T>>),
        Bought(T::AccountId, T::AccountId, KittyIndex, BalanceOf<T>),
    }

    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<Kitty<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn owner)]
    pub type Owner<T: Config> =
        StorageMap<_, Blake2_128Concat, KittyIndex, Option<T::AccountId>, ValueQuery>;

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        KittiesCountOverflow,
        NotOwner,
        OwnerNotExist,
        SameParentIndex,
        InvalidKittyIndex,
        KittyNotExist,
        BuyerIsKittyOwner,
        KittyNotForSale,
        NotEnoughBalance,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let dna = Self::random_value(&who);

            let kitty_id = Self::mint(who.clone(), dna)?;

            Self::deposit_event(Event::KittyCreated(who, kitty_id));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn transfer(
            origin: OriginFor<T>,
            new_owner: T::AccountId,
            kitty_id: KittyIndex,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Some(who.clone()) == Owner::<T>::get(kitty_id),
                Error::<T>::NotOwner
            );

            Self::transfer_kitty_to(kitty_id, &new_owner)?;
            Self::deposit_event(Event::KittyTransfered(who, new_owner, kitty_id));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn breed(
            origin: OriginFor<T>,
            kitty_id_1: KittyIndex,
            kitty_id_2: KittyIndex,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

            let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

            let dna_1 = kitty1.dna;
            let dna_2 = kitty2.dna;

            let selector = Self::random_value(&who);
            let mut new_dna = [0u8; 16];

            for i in 0..dna_1.len() {
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }

            let kitty_id = Self::mint(who.clone(), new_dna)?;

            Self::deposit_event(Event::KittyCreated(who, kitty_id));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn sell_kitty(
            origin: OriginFor<T>,
            kitty_id: KittyIndex,
            price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Some(who.clone()) == Owner::<T>::get(kitty_id),
                Error::<T>::NotOwner
            );

            let mut kitty = Self::kitties(&kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            kitty.price = price.clone();
            Kitties::<T>::insert(kitty_id, Some(kitty));
            Self::deposit_event(Event::PriceUpdateForSale(who, kitty_id, price));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn buy_kitty(origin: OriginFor<T>, kitty_id: KittyIndex) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            ensure!(
                Some(buyer.clone()) != Owner::<T>::get(kitty_id),
                Error::<T>::BuyerIsKittyOwner
            );

            let kitty = Self::kitties(&kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            let sale_price = kitty.price.ok_or(Error::<T>::KittyNotForSale)?;

            ensure!(
                T::Currency::free_balance(&buyer) >= sale_price,
                Error::<T>::NotEnoughBalance
            );

            let seller = Owner::<T>::get(kitty_id).ok_or(Error::<T>::OwnerNotExist)?;

            T::Currency::transfer(&buyer, &seller, sale_price, ExistenceRequirement::KeepAlive)?;

            Self::transfer_kitty_to(kitty_id, &buyer)?;

            Self::deposit_event(Event::Bought(buyer, seller, kitty_id, sale_price));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)
        }

        fn mint(owner: T::AccountId, dna: [u8; 16]) -> Result<KittyIndex, Error<T>> {
            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    ensure!(
                        id != KittyIndex::max_value(),
                        Error::<T>::KittiesCountOverflow
                    );
                    id
                }
                None => 1,
            };
            // reserve tokens for mint any kitty.
            T::Currency::reserve(&owner, T::MintDeposit::get())
                .map_err(|_| Error::<T>::NotEnoughBalance)?;

            Kitties::<T>::insert(kitty_id, Some(Kitty { dna, price: None }));
            Owner::<T>::insert(kitty_id, Some(owner.clone()));
            KittiesCount::<T>::put(kitty_id + 1);
            Ok(kitty_id)
        }

        fn transfer_kitty_to(kitty_id: KittyIndex, to: &T::AccountId) -> Result<(), Error<T>> {
            let _kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            let _owner = Owner::<T>::get(kitty_id).ok_or(Error::<T>::OwnerNotExist)?;
            Owner::<T>::insert(kitty_id, Some(to.clone()));
            Ok(())
        }
    }
}
