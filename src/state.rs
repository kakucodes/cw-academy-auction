use cosmwasm_std::{Coin, Decimal};
use cw_storage_plus::{Item, Map};

pub const OWNER: Item<String> = Item::new("owner");
pub const AUCTION_ITEM_TITLE: Item<String> = Item::new("auction_item_title");
pub const COMMISSION_PERCENTAGE: Item<Decimal> = Item::new("commission_percentage");
pub const ACTIVE: Item<bool> = Item::new("active");

pub const BIDS: Map<String, Coin> = Map::new("bids");
