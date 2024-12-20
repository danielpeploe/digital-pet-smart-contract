use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // Name of the pet
    pub pet_name: String,

    // Owner of the pet
    pub pet_owner: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Set a new password for the pet
    SetPassword { password: String },

    // Feed the pet a specified amount
    Feed { amount: u8 },

    // Play with the pet for a specified amount
    Play { amount: u8 },

    // Allow the pet to rest for a specified amount
    Rest { amount: u8 },

    // Transfer ownership of the pet
    Transfer { new_owner: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Check if the pet is hungry
    IsHungry { password: String },

    // Get the current status of the pet
    GetStatus { password: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    // Response for checking if the pet is hungry
    IsHungry {
        is_hungry: bool,
    },

    // Response for retrieving the pet's status
    GetStatus {
        pet_name: String,
        hunger_level: u8,
        happiness_level: u8,
        energy_level: u8,
    },
}
