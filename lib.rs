#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod unit_test_bug {

    #[derive(Debug, PartialEq, Eq, Copy, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FlipError {
       // A flip error to cause revert
       FlipError,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct UnitTestBug {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl UnitTestBug {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip_with_error(&mut self) -> Result<(), FlipError>{
            self.value = !self.value;
            // Revert should occur and self.value remains unchanged
            Err(FlipError::FlipError)
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {

        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let unit_test_bug = UnitTestBug::default();
            assert_eq!(unit_test_bug.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut unit_test_bug = UnitTestBug::new(false);
            assert_eq!(unit_test_bug.get(), false);
            // Error is returned, revert should occur, and value should remain as false
            assert_eq!(unit_test_bug.flip_with_error(), Err(FlipError::FlipError));
            // This test is going to FAIL because the revert did not occur
            assert_eq!(unit_test_bug.get(), false);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = UnitTestBugRef::default();

            // When
            let contract_account_id = client
                .instantiate("unit_test_bug", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<UnitTestBugRef>(contract_account_id.clone())
                .call(|unit_test_bug| unit_test_bug.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = UnitTestBugRef::new(false);
            let contract_account_id = client
                .instantiate("unit_test_bug", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<UnitTestBugRef>(contract_account_id.clone())
                .call(|unit_test_bug| unit_test_bug.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<UnitTestBugRef>(contract_account_id.clone())
                .call(|unit_test_bug| unit_test_bug.flip_with_error());

            // Call flip. Result should still be false as error is returned
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await;

            // Then
            let get = build_message::<UnitTestBugRef>(contract_account_id.clone())
                .call(|unit_test_bug| unit_test_bug.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            // This test does pass properly in e2e tests
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }
    }
}
