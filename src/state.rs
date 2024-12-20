use cosmwasm_std::CanonicalAddr;
use schemars::JsonSchema;
use secret_toolkit::storage::Item;
use serde::{Deserialize, Serialize};

pub static PET_KEY: &[u8] = b"pet";
pub static PET: Item<Pet> = Item::new(PET_KEY);

pub static PET_OWNER_KEY: &[u8] = b"pet_owner";
pub static PET_OWNER: Item<CanonicalAddr> = Item::new(PET_OWNER_KEY);

pub static PASWORD_KEY: &[u8] = b"password";
pub static PASSWORD: Item<String> = Item::new(PASWORD_KEY);

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq, JsonSchema)]
pub struct Pet {
    pub name: String,
    pub hunger_level: u8,
    pub happiness_level: u8,
    pub energy_level: u8,
    pub last_action_block: u64,
}
