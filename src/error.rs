use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized. Action only permitted for {owner}")]
    Unauthorized { owner: String },

    #[error("Auction is not active")]
    AuctionInactive {},

    #[error("Bid too low. Minimum bid is {minimum_bid_amount} {bid_denom}. Your current bid is {current_bid_amount}")]
    BidTooLow {
        minimum_bid_amount: u128,
        bid_denom: String,
        current_bid_amount: u128,
    },

    #[error("Invalid bid amount")]
    InvalidBidAmount {},

    #[error("Nothing to withdraw")]
    NothingToWithdraw {},

    #[error("Cannot perform action while auction is active")]
    AuctionActive {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
