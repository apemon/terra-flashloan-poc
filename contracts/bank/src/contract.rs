#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128, StdError, CosmosMsg, WasmMsg};
use flashloan::bank::{InstantiateMsg, ExecuteMsg, QueryMsg, ConfigResponse, FlashloanReceiveMsg};
use terraswap::querier::{query_balance};
use terraswap::asset::{Asset, AssetInfo};

use crate::error::ContractError;
use crate::state::{Tmp, TMP, Config, CONFIG};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Flashloan { recipient, amount, msg } => flashloan(deps, env, info, recipient, amount, msg),
        ExecuteMsg::FlashloanHook {} => {
            // check owner
            Ok(Response::default())
        }
    }
}

pub fn flashloan(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
    msg: Binary
) -> Result<Response, ContractError> {
    // query balance
    let balance = query_balance(&deps.querier, env.contract.address.clone(), "uusd".to_string())?;
    // check loan < balance
    if amount > balance {
        return Err(ContractError::Std(StdError::generic_err("not enough money")));
    }
    // save current balance to tmp
    let tmp = Tmp {
        prev_balance: balance
    };
    TMP.save(deps.storage, &tmp)?;
    // send fund to recipient
    let mut messages: Vec<CosmosMsg> = vec![];
    let asset = Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string()
        },
        amount: amount
    };
    messages.push(asset.into_msg(&deps.querier, recipient.clone())?);
    // send msg to recipient
    let flash_msg = FlashloanReceiveMsg {
        sender: env.contract.address.clone(),
        amount: amount,
        msg: msg,
    }.into_cosmos_msg(recipient.clone().to_string())?;
    messages.push(flash_msg);
    // send flashloan hook to check final position
    messages.push(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.clone().to_string(),
            msg: to_binary(&ExecuteMsg::FlashloanHook {
                
            })?,
            funds: vec![]
        })
    );
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "flashloan")
        .add_attribute("amount", amount.to_string()))
}

pub fn flashloan_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    // only this contract can call
    let sender_addr = info.sender.as_str();
    if sender_addr != env.contract.address {
        return Err(ContractError::Unauthorized {});
    };
    // load temp
    let tmp = TMP.load(deps.storage)?;
    let balance = query_balance(&deps.querier, env.contract.address.clone(), "uusd".to_string())?;
    if balance < tmp.prev_balance {
        return Err(ContractError::Std(StdError::generic_err("balance mismatch")));
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?,
     })
}