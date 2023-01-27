use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};

use crate::contract::{execute, instantiate, query};

fn auctioning_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);

    Box::new(contract)
}

#[cfg(test)]
mod tests {
    use crate::integration_tests::auctioning_contract;
    use crate::msg::{AuctionStatusResponse, BidResponse, InstantiateMsg};
    use crate::multitest::AuctionContract;
    use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, Uint128};
    use cw_multi_test::App;

    #[test]
    fn instantiate_with_defaults() {
        let sender = Addr::unchecked("sender");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(100_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &sender,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: None,
                auction_item_title: "Test Auction".to_string(),
                commission_percentage: None,
            },
        )
        .unwrap();

        let auction_status = contract.query_auction_status(&app).unwrap();

        assert_eq!(
            auction_status,
            AuctionStatusResponse {
                owner: sender.to_string(),
                active: true,
                auction_item_title: "Test Auction".to_string(),
                highest_bid: BidResponse {
                    bidder: sender.to_string(),
                    bid: Coin::new(100_000, "ubtc")
                },
                bidders_count: 1,
                commission_percentage: Decimal::new(Uint128::new(50_000_000_000_000_000)),
            }
        );

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(100_000, "ubtc")
        );

        assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), &[])
    }

    #[test]
    fn instantiate_without_defaults() {
        let sender = Addr::unchecked("sender");
        let auction_owner = Addr::unchecked("auction_owner");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(100_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &sender,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: Some(auction_owner.to_string()),
                auction_item_title: "Test Auction 2".to_string(),
                commission_percentage: Some(Decimal::new(Uint128::new(10_000_000_000_000_000))),
            },
        )
        .unwrap();

        let auction_status = contract.query_auction_status(&app).unwrap();

        assert_eq!(
            auction_status,
            AuctionStatusResponse {
                owner: auction_owner.to_string(),
                active: true,
                auction_item_title: "Test Auction 2".to_string(),
                highest_bid: BidResponse {
                    bidder: sender.to_string(),
                    bid: Coin::new(100_000, "ubtc")
                },
                bidders_count: 1,
                commission_percentage: Decimal::new(Uint128::new(10_000_000_000_000_000)),
            }
        );

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(100_000, "ubtc")
        );

        assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), &[])
    }

    #[test]
    fn query_bids() {
        let sender = Addr::unchecked("sender");
        let bidder = Addr::unchecked("bidder");
        let non_bidder = Addr::unchecked("non_bidder");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(100_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder, coins(150_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &sender,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: None,
                auction_item_title: "Test Auction".to_string(),
                commission_percentage: None,
            },
        )
        .unwrap();

        let _ = contract.bid(&mut app, &bidder, &coins(150_000, "ubtc"));

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(250_000, "ubtc")
        );
        assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), &[]);
        assert_eq!(app.wrap().query_all_balances(&bidder).unwrap(), &[]);

        let sender_bid_query_resp = contract.query_user_bid(&app, sender.to_string()).unwrap();
        assert_eq!(
            sender_bid_query_resp,
            BidResponse {
                bidder: sender.to_string(),
                bid: coin(100_000, "ubtc")
            }
        );

        let bidder_query_resp = contract.query_user_bid(&app, bidder.to_string()).unwrap();
        assert_eq!(
            bidder_query_resp,
            BidResponse {
                bidder: bidder.to_string(),
                bid: coin(150_000, "ubtc")
            }
        );

        let nonbidder_query_resp = contract
            .query_user_bid(&app, non_bidder.to_string())
            .unwrap();
        assert_eq!(
            nonbidder_query_resp,
            BidResponse {
                bidder: non_bidder.to_string(),
                bid: coin(0, "ubtc")
            }
        );
    }

    #[test]
    fn auction_count_and_highest_bid() {
        let sender = Addr::unchecked("sender");
        let bidder = Addr::unchecked("bidder");
        let bidder_two = Addr::unchecked("bidder_two");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(100_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder, coins(150_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder_two, coins(300_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &sender,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: None,
                auction_item_title: "Test Auction".to_string(),
                commission_percentage: None,
            },
        )
        .unwrap();

        let AuctionStatusResponse {
            highest_bid: initial_highest_bid,
            bidders_count: initial_bidders_count,
            ..
        } = contract.query_auction_status(&app).unwrap();

        assert_eq!(initial_bidders_count, 1);
        assert_eq!(
            initial_highest_bid,
            BidResponse {
                bidder: sender.to_string(),
                bid: Coin::new(100_000, "ubtc")
            }
        );

        let _ = contract.bid(&mut app, &bidder, &coins(150_000, "ubtc"));

        let AuctionStatusResponse {
            highest_bid: new_highest_bid,
            bidders_count: new_bidders_count,
            ..
        } = contract.query_auction_status(&app).unwrap();

        assert_eq!(new_bidders_count, 2);
        assert_eq!(
            new_highest_bid,
            BidResponse {
                bidder: bidder.to_string(),
                bid: Coin::new(150_000, "ubtc")
            }
        );

        let _ = contract.bid(&mut app, &bidder_two, &coins(250_000, "ubtc"));

        let AuctionStatusResponse {
            highest_bid: final_highest_bid,
            bidders_count: final_bidders_count,
            ..
        } = contract.query_auction_status(&app).unwrap();

        assert_eq!(final_bidders_count, 3);
        assert_eq!(
            final_highest_bid,
            BidResponse {
                bidder: bidder_two.to_string(),
                bid: Coin::new(250_000, "ubtc")
            }
        );

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(500_000, "ubtc")
        );
        assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), &[]);
        assert_eq!(app.wrap().query_all_balances(&bidder).unwrap(), &[]);
        assert_eq!(
            app.wrap().query_all_balances(&bidder_two).unwrap(),
            coins(50_000, "ubtc")
        );
    }

    #[test]
    fn bids_must_beat_current_bid() {
        let sender = Addr::unchecked("sender");
        let bidder = Addr::unchecked("bidder");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(100_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder, coins(100_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &sender,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: None,
                auction_item_title: "Test Auction".to_string(),
                commission_percentage: None,
            },
        )
        .unwrap();

        let _ = contract.bid(&mut app, &bidder, &coins(100_000, "ubtc"));

        let AuctionStatusResponse {
            highest_bid: new_highest_bid,
            bidders_count: new_bidders_count,
            ..
        } = contract.query_auction_status(&app).unwrap();

        assert_eq!(new_bidders_count, 1);
        assert_eq!(
            new_highest_bid,
            BidResponse {
                bidder: sender.to_string(),
                bid: Coin::new(100_000, "ubtc")
            }
        );

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(100_000, "ubtc")
        );
        assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), &[]);
        assert_eq!(
            app.wrap().query_all_balances(&bidder).unwrap(),
            coins(100_000, "ubtc")
        );
    }

    #[test]
    fn only_owner_can_close_bidding() {
        let sender = Addr::unchecked("sender");
        let bidder = Addr::unchecked("bidder");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(100_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &sender,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: None,
                auction_item_title: "Test Auction".to_string(),
                commission_percentage: None,
            },
        )
        .unwrap();

        let _ = contract.close_bidding(&mut app, &bidder.clone());

        let AuctionStatusResponse {
            active: auction_active,
            ..
        } = contract.query_auction_status(&app).unwrap();
        assert_eq!(auction_active, true);

        let _ = contract.close_bidding(&mut app, &sender);

        let AuctionStatusResponse {
            active: auction_active,
            ..
        } = contract.query_auction_status(&app).unwrap();
        assert_eq!(auction_active, false);
    }

    #[test]
    fn cannot_retract_funds_while_active() {
        let owner = Addr::unchecked("sender");
        let bidder = Addr::unchecked("bidder");
        let bidder_two = Addr::unchecked("bidder_two");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &owner, coins(100_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder, coins(150_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder_two, coins(200_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &owner,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: None,
                auction_item_title: "Test Auction".to_string(),
                commission_percentage: None,
            },
        )
        .unwrap();

        let _ = contract.bid(&mut app, &bidder, &coins(150_000, "ubtc"));

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(250_000, "ubtc")
        );

        let _ = contract.retract_funds(&mut app, &owner, None);

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(250_000, "ubtc")
        );

        let _ = contract.close_bidding(&mut app, &owner);

        let _ = contract.retract_funds(&mut app, &owner, None);

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(150_000, "ubtc")
        );

        let AuctionStatusResponse {
            active: auction_active,
            ..
        } = contract.query_auction_status(&app).unwrap();
        assert_eq!(auction_active, false);
    }

    #[test]
    fn winner_cannot_retract_funds() {
        let owner = Addr::unchecked("sender");
        let bidder = Addr::unchecked("bidder");
        let bidder_two = Addr::unchecked("bidder_two");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &owner, coins(100_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder, coins(150_000, "ubtc"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &bidder_two, coins(200_000, "ubtc"))
                .unwrap();
        });

        let contract_id = app.store_code(auctioning_contract());

        let contract = AuctionContract::instantiate(
            &mut app,
            contract_id,
            &owner,
            None,
            "Test auction contract",
            &coins(100_000, "ubtc"),
            &InstantiateMsg {
                owner: None,
                auction_item_title: "Test Auction".to_string(),
                commission_percentage: None,
            },
        )
        .unwrap();

        let _ = contract.bid(&mut app, &bidder, &coins(150_000, "ubtc"));

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(250_000, "ubtc")
        );

        let _ = contract.close_bidding(&mut app, &owner);

        let _ = contract.retract_funds(&mut app, &bidder, None);

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(250_000, "ubtc")
        );

        let _ = contract.retract_funds(&mut app, &owner, None);

        assert_eq!(
            app.wrap().query_all_balances(contract.addr()).unwrap(),
            coins(150_000, "ubtc")
        );
    }
}
