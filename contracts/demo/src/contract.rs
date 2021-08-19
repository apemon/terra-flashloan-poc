#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, CosmosMsg, WasmMsg, Uint128, from_binary, BankMsg, Coin, Attribute};
use flashloan::bank::{ExecuteMsg as BankExecuteMsg, FlashloanReceiveMsg};
use flashloan::counter::{ExecuteMsg as CounterExecuteMsg};
use terraswap::asset::{Asset, AssetInfo};

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, FlashloanHookMsg};
use crate::state::{Config, CONFIG};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        bank: deps.api.addr_canonicalize(msg.bank_addr.as_str())?,
        counter: deps.api.addr_canonicalize(msg.counter_addr.as_str())?,
        flag: true
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
        ExecuteMsg::Receive(msg) => receive_flashloan(deps,env,info,msg),
        ExecuteMsg::Borrow { } => borrow(deps,env,info),
        ExecuteMsg::SetFlag { flag } => {
            let mut config = CONFIG.load(deps.storage)?;
            config.flag = flag;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default())
        }
    }
}

pub fn receive_flashloan(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: FlashloanReceiveMsg
) -> Result<Response, ContractError> {
    match from_binary(&msg.msg) {
        Ok(FlashloanHookMsg::Process { }) => {
            let config = CONFIG.load(deps.storage)?;
            let mut messages: Vec<CosmosMsg> = vec![];
            let mut attributes: Vec<Attribute> = vec![];
            attributes.push(Attribute {
                key: "action".to_string(),
                value: "receive_flashloan".to_string()
            });
            attributes.push(Attribute {
                key: "flag".to_string(),
                value: config.flag.clone().to_string()
            });
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.counter)?.to_string(),
                msg: to_binary(&CounterExecuteMsg::Increment {
                    
                })?,
                funds: vec![]
            }));
            if config.flag {
                // calculate tax
                let asset = Asset {
                    info: AssetInfo::NativeToken {
                        denom: "uusd".to_string()
                    },
                    amount: msg.amount
                };
                let tax = asset.compute_tax(&deps.querier)?;
                let bank_addr = deps.api.addr_humanize(&config.bank)?;
                //messages.push(asset.into_msg(&deps.querier, bank_addr.clone())?);
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: bank_addr.clone().to_string(),
                    amount: vec![Coin {
                        denom: "uusd".to_string(),
                        amount: asset.amount + tax
                    }]
                }));
                attributes.push(Attribute {
                    key: "send_amount".to_string(),
                    value: (asset.amount + tax).to_string()
                });
            }
            Ok(Response::new()
                .add_messages(messages)
                .add_attributes(attributes))
        }
        Err(err) => Err(ContractError::Std(err))
    }
}

pub fn borrow(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo
) -> Result<Response, ContractError> {
    // load config
    let config = CONFIG.load(deps.storage)?;
    // flashloan

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bank)?.to_string(),
            msg: to_binary(&BankExecuteMsg::Flashloan {
                recipient: env.contract.address.clone(),
                amount: Uint128::from(100000000u128),
                msg: to_binary(&FlashloanHookMsg::Process {

                })?
            })?,
            funds: vec![]
        }))
        .add_attribute("action", "borrow")
        .add_attribute("flag", config.flag.clone().to_string()))
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
        bank: deps.api.addr_humanize(&config.bank)?,
        flag: config.flag
     })
}