use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub auction_item_title: String,
    pub commission_percentage: Option<Decimal>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bid {},
    // Withdraw { withdraw_address: Option<String> },
    RetractFunds { withdraw_address: Option<String> },
    CloseBidding {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(AuctionStatusResponse)]
    GetAuctionStatus {},

    #[returns(BidResponse)]
    GetUserBid { bidder: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct AuctionStatusResponse {
    pub owner: String,
    pub active: bool,
    pub auction_item_title: String,
    pub highest_bid: BidResponse,
    pub bidders_count: usize,
    pub commission_percentage: Decimal,
}

#[cw_serde]
pub struct BidResponse {
    pub bidder: String,
    pub bid: Coin,
}
