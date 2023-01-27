#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ACTIVE, AUCTION_ITEM_TITLE, BIDS, COMMISSION_PERCENTAGE, OWNER};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-academy-auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const BID_DENOM: &str = "ubtc";

pub fn calc_highest_bid(deps: Deps) -> (String, Coin) {
    BIDS.range(deps.storage, None, None, Order::Ascending)
        .map(|v| v.unwrap_or((String::new(), Coin::new(0, "utoken"))))
        .reduce(
            |(highest_bidder_address, highest_bid), (bidder_address, bidder_amount)| {
                if highest_bid.amount > bidder_amount.amount {
                    (highest_bidder_address, highest_bid)
                } else {
                    (bidder_address, bidder_amount)
                }
            },
        )
        .unwrap()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner_addr = msg.owner.unwrap_or(info.sender.to_string());
    OWNER.save(deps.storage, &owner_addr)?;

    let commission = msg
        .commission_percentage
        .unwrap_or(Decimal::new(Uint128::new(50_000_000_000_000_000)));
    COMMISSION_PERCENTAGE.save(deps.storage, &commission)?;

    AUCTION_ITEM_TITLE.save(deps.storage, &msg.auction_item_title)?;
    ACTIVE.save(deps.storage, &true)?;

    let zero_coin = Coin::new(0, "ubtc");

    let new_bid_funds = info
        .funds
        .iter()
        .find(|coin| coin.denom == BID_DENOM)
        .unwrap_or(&zero_coin);

    BIDS.save(deps.storage, info.sender.to_string(), new_bid_funds)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bid {} => execute::bid(deps, info),
        ExecuteMsg::CloseBidding {} => execute::close(deps, info),
        ExecuteMsg::RetractFunds { withdraw_address } => {
            execute::retract(deps, info, withdraw_address)
        }
    }
}

pub mod execute {
    use cosmwasm_std::{BankMsg, Coin, StakingMsg};

    use crate::state::BIDS;

    use super::*;

    pub fn bid(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let auction_active = ACTIVE.load(deps.storage)?;
        if !auction_active {
            return Err(ContractError::AuctionInactive {});
        }

        let zero_coin = Coin::new(0, "ubtc");

        let new_bid_funds = info
            .funds
            .iter()
            .find(|coin| coin.denom == BID_DENOM)
            .unwrap_or(&zero_coin);

        if new_bid_funds.amount.le(&Uint128::zero()) {
            return Err(ContractError::InvalidBidAmount {});
        }

        let (highest_bid_address, highest_bid) = calc_highest_bid(deps.as_ref());

        let previous_bid = if highest_bid_address == info.sender.to_string() {
            highest_bid.clone()
        } else {
            BIDS.may_load(deps.storage, info.sender.to_string())?
                .unwrap_or(zero_coin.clone())
        };

        let new_bid = Coin {
            denom: "ubtc".to_string(),
            amount: previous_bid.amount + new_bid_funds.amount,
        };

        if highest_bid.amount >= new_bid.amount {
            return Err(ContractError::BidTooLow {
                minimum_bid_amount: highest_bid.amount.u128(),
                bid_denom: "ubtc".to_string(),
                current_bid_amount: previous_bid.amount.u128(),
            });
        }

        BIDS.save(deps.storage, info.sender.to_string(), &new_bid)?;

        Ok(Response::new()
            .add_attribute("action", "bid")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("bid_amount", new_bid.to_string()))
    }

    pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized { owner: owner });
        }

        let auction_active = ACTIVE.load(deps.storage)?;
        if !auction_active {
            return Err(ContractError::AuctionInactive {});
        }

        ACTIVE.save(deps.storage, &false)?;

        Ok(Response::new()
            .add_attribute("action", "close_bidding")
            .add_attribute("sender", info.sender.to_string()))
    }

    pub fn retract(
        deps: DepsMut,
        info: MessageInfo,
        withdraw_address: Option<String>,
    ) -> Result<Response, ContractError> {
        let auction_active = ACTIVE.load(deps.storage)?;
        if auction_active {
            return Err(ContractError::AuctionActive {});
        }

        let (highest_bidder_address, _) = calc_highest_bid(deps.as_ref());

        if info.sender.to_string() == highest_bidder_address {
            return Err(ContractError::NothingToWithdraw {});
        }

        let withdrawl = match BIDS.may_load(deps.storage, info.sender.to_string()) {
            Ok(Some(bid)) if bid.amount != Uint128::new(0) => bid,
            _ => return Err(ContractError::NothingToWithdraw {}),
        };

        let to_address = withdraw_address.unwrap_or(info.sender.to_string());

        BIDS.save(
            deps.storage,
            info.sender.to_string(),
            &Coin::new(0, BID_DENOM),
        )?;

        StakingMsg::Delegate { validator: (), amount: () }

        Ok(Response::new()
            .add_message(BankMsg::Send {
                to_address,
                amount: vec![withdrawl],
            })
            .add_attribute("action", "retract_funds")
            .add_attribute("sender", info.sender))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAuctionStatus {} => to_binary(&query::status(deps)?),
        QueryMsg::GetUserBid { bidder } => to_binary(&query::get_user_bid(deps, bidder)?),
    }
}

pub mod query {
    use cosmwasm_std::{coin, Coin, Order};

    use crate::{
        msg::{AuctionStatusResponse, BidResponse},
        state::BIDS,
    };

    use super::*;

    pub fn status(deps: Deps) -> StdResult<AuctionStatusResponse> {
        let owner = OWNER.load(deps.storage)?;
        let active = ACTIVE.load(deps.storage)?;
        let auction_item_title = AUCTION_ITEM_TITLE.load(deps.storage)?;
        let commission_percentage = COMMISSION_PERCENTAGE.load(deps.storage)?;

        let all_bids: Vec<(String, Coin)> = BIDS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|v| v.unwrap_or((String::new(), Coin::new(0, "utoken"))))
            .collect();

        let (bidder, bid) = all_bids
            .iter()
            .map(|(addr, bid)| (addr, bid))
            .reduce(
                |(highest_bidder_address, highest_bid), (bidder_address, bidder_amount)| {
                    if highest_bid.amount > bidder_amount.amount {
                        (highest_bidder_address, highest_bid)
                    } else {
                        (bidder_address, bidder_amount)
                    }
                },
            )
            .unwrap();

        Ok(AuctionStatusResponse {
            owner,
            active,
            auction_item_title,
            highest_bid: BidResponse {
                bidder: bidder.to_string(),
                bid: bid.clone(),
            },
            bidders_count: all_bids.len(),
            commission_percentage,
        })
    }

    pub fn get_user_bid(deps: Deps, bidder: String) -> StdResult<BidResponse> {
        let bid = BIDS
            .may_load(deps.storage, bidder.to_string())?
            .unwrap_or(coin(0, "ubtc"));
        Ok(BidResponse { bidder, bid })
    }
}
