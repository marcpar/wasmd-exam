#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{UsersResponse, ExistsResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:angel-protocol";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        users: Vec::new(),
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
       )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddUser{user} => add_user(deps, info , user),
        ExecuteMsg::RemoveUser { user } => remove_user(deps, info ,user),
        ExecuteMsg::UpdateUsers{add, remove} => update_user(deps, info ,add, remove) ,
    }
}

pub fn add_user(deps: DepsMut, info: MessageInfo , user: String )-> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError>{
        
        let address: Addr = deps.api.addr_validate(&user)?;

        if state.owner == info.sender{
        
            state.users.push(address);
            Ok(state)
        }else{
            Err(ContractError::Unauthorized {})
        }
    })?;
    Ok(Response::new().add_attribute("method", "add_user"))
}

pub fn remove_user(deps: DepsMut, info: MessageInfo , user: String )-> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError>{
        let address: Addr = deps.api.addr_validate(&user)?;
        
        if state.owner == info.sender{
           let index: usize = find_index( &state.users, address)?;
           state.users.remove(index);
           Ok(state)
        }else{
            Err(ContractError::Unauthorized {})
        }
    })?;
    Ok(Response::new().add_attribute("method", "add_user"))
}

pub fn update_user(deps: DepsMut, info: MessageInfo , add: Vec<String> , remove: Vec<String>)-> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError>{
        
        for user in add{
            let address: Addr = deps.api.addr_validate(&user).unwrap();
            state.users.push(address);
            
        }

        for user in remove{
            let address: Addr = deps.api.addr_validate(&user).unwrap();
            let index: usize = find_index(&state.users,address).unwrap();
            state.users.remove(index);
        }
        
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "add_user"))
}

// fn remove(users: Vec<Addr> , user: Addr)-> Result<Vec<Addr>, ContractError>{
//     let index: usize = find_index(users, user)?;
//     users.remove(index);
//     Ok(users)
// }

// fn add(users: Vec<Addr> , user: Addr)-> Result<Vec<Addr>, ContractError>{
//     if users.contains(&user){
//         Err(ContractError::Existing{user: user.into_string()})
   
//     }else{
//         users.push(user);
//         Ok(users)
//     }

// }

fn find_index(users: &Vec<Addr> , user: Addr) -> Result<usize, ContractError>{
    let index = users.iter().position(|x| *x == user);
    if let Some(ind) = index {
        Ok(ind)
    } else {
        Err(ContractError::NotFound{user: user.into_string()})
    }
}

// pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
//     STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//         state.count += 1;
//         Ok(state)
//     })?;

//     Ok(Response::new().add_attribute("method", "try_increment"))
// }
// pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
//     STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//         if info.sender != state.owner {
//             return Err(ContractError::Unauthorized {});
//         }
//         state.count = count;
//         Ok(state)
//     })?;
//     Ok(Response::new().add_attribute("method", "reset"))
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Users {} => to_binary(&query_users(deps)?),
        QueryMsg::User {user} => to_binary(&query_user(deps, user)?)
    }
}

fn query_users(deps: Deps) -> StdResult<UsersResponse>{
    let state = STATE.load(deps.storage)?;
    Ok(UsersResponse{users: state.users.iter().map(|x|  x.to_string()).collect()})
}

fn query_user(deps: Deps, user: String) -> StdResult<ExistsResponse>{
    let state = STATE.load(deps.storage)?;
    let address: Addr = deps.api.addr_validate(&user)?;
    
    Ok(ExistsResponse{exists: state.users.contains(&address)})
}
// fn query_count(deps: Deps) -> StdResult<CountResponse> {
//     let state = STATE.load(deps.storage)?;
//     Ok(CountResponse { count: state.count })
// }

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Users {}).unwrap();
        let value: UsersResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.users.iter().len());
    }

    #[test]
    fn AddUser() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::AddUser {user: String::from("wasm1hu8dr4t235qcpuu7ej73mr43ncsg86xdj3s6yw")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Users {}).unwrap();
        let value: UsersResponse = from_binary(&res).unwrap();
        assert_eq!(true, value.users.contains(&String::from("wasm1hu8dr4t235qcpuu7ej73mr43ncsg86xdj3s6yw")));
    }

    // #[test]
    // fn RemoveUser() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::AddUser {user: String::from("wasm1hu8dr4t235qcpuu7ej73mr43ncsg86xdj3s6yw")};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::Users {}).unwrap();
    //     let value: UsersResponse = from_binary(&res).unwrap();
    //     assert_eq!(true, value.users.contains("wasm1hu8dr4t235qcpuu7ej73mr43ncsg86xdj3s6yw");
    // }

    
    // fn UpdateUsers() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
