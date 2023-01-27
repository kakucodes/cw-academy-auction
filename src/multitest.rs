use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::{
    contract::{execute, instantiate, query},
    msg::{AuctionStatusResponse, BidResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    ContractError,
};

pub struct AuctionContract(Addr);

impl AuctionContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        admin: impl Into<Option<&'a Addr>>,
        label: &str,
        funds: &[Coin],
        instantiate_msg: &InstantiateMsg,
    ) -> StdResult<AuctionContract> {
        let admin = admin.into();

        app.instantiate_contract(
            code_id,
            sender.clone(),
            &instantiate_msg,
            funds,
            label,
            admin.map(Addr::to_string),
        )
        .map_err(|err| err.downcast().unwrap())
        .map(AuctionContract)
    }

    #[track_caller]
    pub fn query_auction_status(&self, app: &App) -> StdResult<AuctionStatusResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::GetAuctionStatus {})
    }

    #[track_caller]
    pub fn query_user_bid(&self, app: &App, bidder: String) -> StdResult<BidResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::GetUserBid { bidder })
    }

    #[track_caller]
    pub fn bid(
        &self,
        app: &mut App,
        sender: &Addr,
        bid_funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecuteMsg::Bid {},
            &bid_funds,
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    #[track_caller]
    pub fn retract_funds(
        &self,
        app: &mut App,
        sender: &Addr,
        withdraw_address: Option<String>,
    ) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecuteMsg::RetractFunds { withdraw_address },
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    #[track_caller]
    pub fn close_bidding(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecuteMsg::CloseBidding {},
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }
}
