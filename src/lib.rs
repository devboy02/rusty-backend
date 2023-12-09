// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Import the Stylus SDK along with alloy primitive types for use in our program.
use alloc::{string::String, vec::Vec};
use alloy_primitives::FixedBytes;
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    call::Call,
    evm, msg,
    prelude::*,
};

pub trait Erc721Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
}

sol_storage! {
    #[entrypoint]
    pub struct SimpleStory {
        mapping(address => uint256) user_storybooks_count;
        mapping(address => uint256[]) user_storybooks;
        mapping(uint256 => string) ipfs_hashes;
        address[] unique_users;
        uint256 next_token_id;

    }
}


impl SimpleStory {
    
    fn add_new_user(&mut self, new_user: Address) -> Result<(), Vec<u8>>{
        if !self.user_exists(new_user) {
            self.unique_users.push(new_user);
        }
        Ok(())
    }

    fn user_exists(&self, user: Address) -> bool {
        for i in 0..self.unique_users.len(){
            if self.unique_users.get(i) == Some(user) {
                return true;
            }
        }
        false
    }

}

#[external]
impl SimpleStory {
    pub fn create_storybook(&mut self, ipfs_hash: String) -> Result<(), Vec<u8>> {

        let sender = msg::sender();
        let token_id = self.next_token_id.get();

        let _ = self.add_new_user(sender.clone());

        self.ipfs_hashes.setter(token_id).set_str(ipfs_hash);

        self.user_storybooks.setter(sender.clone()).push(token_id);
        self.next_token_id.set(token_id + U256::from(1));
        
        let user_storybook_count = self.user_storybooks_count.get(sender.clone());
        self.user_storybooks_count.insert(sender.clone(), user_storybook_count + U256::from(1));

        Ok(())
    } 

    pub fn get_user_storybooks_count(&self, user: Address) -> Result<U256, Vec<u8>> {
        let user_storybooks_count = self.user_storybooks_count.get(user.clone());
        Ok(user_storybooks_count)
    }

    pub fn get_user_storybooks(&self, user: Address) -> Result<Vec<U256>, Vec<u8>> {
        let user_storybooks_raw = unsafe { self.user_storybooks.get(user.clone()).into_raw() };

        let mut user_storybooks = Vec::new();

        for index in 0..user_storybooks_raw.len() {
            let item = user_storybooks_raw.get(index);
            let u256_value = item.unwrap();
            user_storybooks.push(u256_value);
        }

        Ok(user_storybooks)
    }

    pub fn get_ipfs_hash(&self, token_id: U256) -> Result<String, Vec<u8>> {
        let ipfs_hash = unsafe { self.ipfs_hashes.get(token_id).into_raw().get_string() };
        Ok(ipfs_hash)
    }

}