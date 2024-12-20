use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg};
use crate::state::{Pet, PASSWORD, PET, PET_OWNER};

// Instantiate the contract with a new pet
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let pet_owner: cosmwasm_std::Addr = msg.pet_owner.unwrap_or(info.sender);
    let pet_owner_canonical = deps.api.addr_canonicalize(pet_owner.as_str())?;

    PET_OWNER.save(deps.storage, &pet_owner_canonical)?;

    if msg.pet_name.is_empty() {
        return Err(StdError::generic_err("Pet name must not be empty"));
    }

    // Create a new pet with default values
    let pet = Pet {
        name: msg.pet_name,
        hunger_level: 5,
        happiness_level: 5,
        energy_level: 5,
        last_action_block: env.block.height,
    };

    PET.save(deps.storage, &pet)?;

    Ok(Response::default())
}

// Execute the contract's actions
#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SetPassword { password } => try_set_password(deps, info, password),
        ExecuteMsg::Feed { amount } => try_feed(deps, env, info, amount),
        ExecuteMsg::Play { amount } => try_play(deps, env, info, amount),
        ExecuteMsg::Rest { amount } => try_rest(deps, env, info, amount),
        ExecuteMsg::Transfer { new_owner } => try_transfer(deps, info, new_owner),
    }
}

// Set a new password for the pet
pub fn try_set_password(deps: DepsMut, info: MessageInfo, password: String) -> StdResult<Response> {
    let pet_owner = deps.api.addr_humanize(&PET_OWNER.load(deps.storage)?)?;
    if info.sender != pet_owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    PASSWORD.save(deps.storage, &password)?;

    Ok(Response::default())
}

// Check if the pet is hungry (hunger level >= 7)
pub fn try_is_hungry(deps: Deps, password: String, env: Env) -> StdResult<Binary> {
    check_password(deps, password)?;

    let pet = PET.load(deps.storage)?;
    let hunger_level = pet.hunger_level;

    let blocks_passed = (env.block.height - pet.last_action_block) / 10;
    let updated_hunger = (hunger_level + blocks_passed as u8).min(10);

    let is_hungry = updated_hunger >= 7;

    Ok(to_binary(&QueryAnswer::IsHungry { is_hungry })?)
}

// Feed the pet and decrease the hunger level of the pet
pub fn try_feed(deps: DepsMut, env: Env, info: MessageInfo, amount: u8) -> StdResult<Response> {
    let pet_owner = deps.api.addr_humanize(&PET_OWNER.load(deps.storage)?)?;
    if info.sender != pet_owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    check_amount(amount, "Feed")?;

    let mut pet = update_state(deps.storage, env).unwrap();

    pet.hunger_level = (pet.hunger_level - amount).max(0);
    PET.save(deps.storage, &pet)?;

    Ok(Response::default())
}

// Play with the pet and increase happiness level of the pet and minus the energy level
pub fn try_play(deps: DepsMut, env: Env, info: MessageInfo, amount: u8) -> StdResult<Response> {
    let pet_owner = deps.api.addr_humanize(&PET_OWNER.load(deps.storage)?)?;
    if info.sender != pet_owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    check_amount(amount, "Play")?;

    let mut pet = update_state(deps.storage, env).unwrap();

    if pet.energy_level < 1 {
        return Err(StdError::generic_err("Pet is too tired to play"));
    }

    pet.happiness_level = (pet.happiness_level + amount).min(10);
    pet.energy_level -= 1;

    PET.save(deps.storage, &pet)?;

    Ok(Response::default())
}

// Rest the pet and increase the energy level of the pet
pub fn try_rest(deps: DepsMut, env: Env, info: MessageInfo, amount: u8) -> StdResult<Response> {
    let pet_owner = deps.api.addr_humanize(&PET_OWNER.load(deps.storage)?)?;
    if info.sender != pet_owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    check_amount(amount, "Rest")?;

    let mut pet = update_state(deps.storage, env).unwrap();
    pet.energy_level = (pet.energy_level + amount).min(10);
    PET.save(deps.storage, &pet)?;

    Ok(Response::default())
}

// Get the current status of the pet
pub fn try_get_status(deps: Deps, password: String, env: Env) -> StdResult<Binary> {
    check_password(deps, password)?;

    let pet = PET.load(deps.storage)?;
    let blocks_passed = (env.block.height - pet.last_action_block) / 10;

    Ok(to_binary(&QueryAnswer::GetStatus {
        pet_name: pet.name,
        hunger_level: (pet.hunger_level + blocks_passed as u8).min(10),
        happiness_level: (pet.happiness_level - blocks_passed as u8).max(0),
        energy_level: pet.energy_level,
    })?)
}

// Transfer ownership of the pet
pub fn try_transfer(deps: DepsMut, info: MessageInfo, new_owner: String) -> StdResult<Response> {
    let pet_owner = deps.api.addr_humanize(&PET_OWNER.load(deps.storage)?)?;
    if info.sender != pet_owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let new_owner = deps.api.addr_validate(&new_owner)?;
    let new_owner_canonical = deps.api.addr_canonicalize(new_owner.as_str())?;

    PET_OWNER.save(deps.storage, &new_owner_canonical)?;

    Ok(Response::default())
}

// Query the contract to check if the pet is hungry
#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsHungry { password } => try_is_hungry(deps, password, env),
        QueryMsg::GetStatus { password } => try_get_status(deps, password, env),
    }
}

// Helper function to check if the password is correct
fn check_password(deps: Deps, password: String) -> StdResult<()> {
    let stored_password = PASSWORD.load(deps.storage)?;

    if stored_password != password {
        return Err(StdError::generic_err("Wrong password"));
    }
    Ok(())
}

// Helper function to check if the amount is within the allowed range
fn check_amount(amount: u8, value: &str) -> StdResult<()> {
    if !(0..=10).contains(&amount) {
        return Err(StdError::generic_err(
            value.to_string() + " amount must be between 0 and 10",
        ));
    }
    Ok(())
}

// Helper function to calculate and update the pet's state based on number of blocks passed since last_action_block
fn update_state(storage: &mut dyn Storage, env: Env) -> StdResult<Pet> {
    let mut pet = PET.load(storage)?;

    let blocks_passed = ((env.block.height - pet.last_action_block) / 10).min(10);

    pet.hunger_level = (pet.hunger_level + blocks_passed as u8).min(10);

    // Set happiness level to 0 if it's negative
    pet.happiness_level = if pet.happiness_level >= blocks_passed as u8 {
        pet.happiness_level - blocks_passed as u8
    } else {
        0
    };

    // Cap energy level at 10
    pet.energy_level = pet.energy_level.min(10);

    pet.last_action_block = env.block.height;
    PET.save(storage, &pet)?;

    Ok(pet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Api, DepsMut};

    // Helper function to set up a pet
    fn create_pet(deps: DepsMut) {
        let msg = InstantiateMsg {
            pet_name: "Buddy".to_string(),
            pet_owner: None,
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        let res = instantiate(deps, env, info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
    }

    #[test]
    fn test_play_increases_happiness_and_decreases_energy() {
        // Setup dependencies and create initial pet state
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let info = mock_info("creator", &[]);
        let env = mock_env();

        // Execute play action and verify that happiness increases up to the max level
        let play_msg = ExecuteMsg::Play { amount: 5 };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), play_msg).unwrap();

        // Assert that happiness level is capped at 10 and energy decreases as expected
        let pet = PET.load(deps.as_ref().storage).unwrap();
        assert_eq!(pet.happiness_level, 10); // Should max out at 10
        assert_eq!(pet.energy_level, 4); // Decreased by 1
    }

    #[test]
    fn test_rest_increases_energy_to_max() {
        // Setup dependencies and create initial pet state
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let info = mock_info("creator", &[]);
        let env = mock_env();

        // Execute rest action to restore energy level and validate outcome
        let rest_msg = ExecuteMsg::Rest { amount: 5 };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), rest_msg).unwrap();

        // Ensure energy is capped at the maximum level
        let pet = PET.load(deps.as_ref().storage).unwrap();
        assert_eq!(pet.energy_level, 10); // Should max out at 10
    }

    #[test]
    fn test_get_status_with_valid_password() {
        // Setup dependencies, create pet, and set password
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let info = mock_info("creator", &[]);
        let env = mock_env();

        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::SetPassword {
                password: "secret".to_string(),
            },
        )
        .unwrap();

        // Query pet status using the correct password
        let query_msg = QueryMsg::GetStatus {
            password: "secret".to_string(),
        };
        let res = query(deps.as_ref(), env, query_msg).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();

        // Check if pet attributes match expected default values
        match value {
            QueryAnswer::GetStatus {
                pet_name,
                hunger_level,
                happiness_level,
                energy_level,
            } => {
                assert_eq!(pet_name, "Buddy".to_string());
                assert_eq!(hunger_level, 5);
                assert_eq!(happiness_level, 5);
                assert_eq!(energy_level, 5);
            }
            _ => panic!("Unexpected query result"),
        }
    }

    #[test]
    fn test_action_denied_to_non_owner() {
        // Attempt to execute an action from a non-owner and expect failure
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let wrong_info = mock_info("not_owner", &[]);
        let env = mock_env();

        // Non-owner attempts to feed the pet
        let res = execute(
            deps.as_mut(),
            env,
            wrong_info,
            ExecuteMsg::Feed { amount: 5 },
        );
        assert!(res.is_err()); // Expect error due to lack of ownership
    }

    #[test]
    fn test_query_denied_with_incorrect_password() {
        // Test that an incorrect password results in denied access
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let info = mock_info("creator", &[]);
        let env = mock_env();

        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::SetPassword {
                password: "correct_password".to_string(),
            },
        )
        .unwrap();

        // Attempt to query status using the wrong password
        let query_msg = QueryMsg::GetStatus {
            password: "wrong_password".to_string(),
        };
        let res = query(deps.as_ref(), env, query_msg);

        assert!(res.is_err()); // Expect error due to incorrect password
    }

    #[test]
    fn test_transfer_ownership() {
        // Test successful transfer of pet ownership
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let info = mock_info("creator", &[]);
        let env = mock_env();

        // Transfer ownership to a new owner and verify change
        let transfer_msg = ExecuteMsg::Transfer {
            new_owner: "new_owner".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info, transfer_msg).unwrap();

        // Confirm that new owner is correctly set
        let owner = PET_OWNER.load(deps.as_ref().storage).unwrap();
        let new_owner_canonical = deps.api.addr_canonicalize("new_owner").unwrap();
        assert_eq!(owner, new_owner_canonical);
    }

    #[test]
    fn test_limit_exceeds_maximum_value() {
        // Test that exceeding the max allowable values causes an error
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let info = mock_info("creator", &[]);
        let env = mock_env();

        // Attempt to play with a value exceeding the allowed max
        let play_msg = ExecuteMsg::Play { amount: 15 };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), play_msg);
        assert!(res.is_err()); // Expect error for exceeding max happiness

        // Attempt to feed with a value exceeding the allowed max
        let feed_msg = ExecuteMsg::Feed { amount: 15 };
        let res = execute(deps.as_mut(), env, info, feed_msg);
        assert!(res.is_err()); // Expect error for exceeding max hunger
    }

    #[test]
    fn test_limit_allows_minimum_value() {
        // Test that the minimum allowable values execute successfully
        let mut deps = mock_dependencies();
        create_pet(deps.as_mut());

        let info = mock_info("creator", &[]);
        let env = mock_env();

        // Play with minimum allowed value (0)
        let play_msg = ExecuteMsg::Play { amount: 0 };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), play_msg).unwrap();

        // Feed with minimum allowed value (0)
        let feed_msg = ExecuteMsg::Feed { amount: 0 };
        let _res = execute(deps.as_mut(), env, info, feed_msg).unwrap();
    }
}
