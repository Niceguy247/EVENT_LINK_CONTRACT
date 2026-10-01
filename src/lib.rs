#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TicketStatus {
    Valid,
    Claimable,
    Used,
    ProofNFT,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub id: u64,
    pub event_id: u64,
    pub tier_name: String,
    pub original_buyer: Address,
    pub current_owner: Address,
    pub status: TicketStatus,
    pub price: i128,
    pub is_listed_resale: bool,
    pub resale_price: i128,
    pub mint_timestamp: u64,
    pub redeem_timestamp: u64,
    pub claim_secret_hash: String,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventMeta {
    pub event_id: u64,
    pub organizer: Address,
    pub name: String,
    pub total_supply: u32,
    pub minted_count: u32,
    pub royalty_bps: u32, // e.g., 500 = 5%
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractEventPayload {
    pub event_id: u64,
    pub ticket_id: Option<u64>,
    pub ticket_owner: Option<Address>,
    pub ticket_status: Option<String>,
    pub tier_name: Option<String>,
    pub ticket_price: Option<i128>,
    pub is_listed_resale: Option<bool>,
    pub resale_price: Option<i128>,
    pub mint_timestamp: Option<u64>,
    pub redeem_timestamp: Option<u64>,
    pub event_name: Option<String>,
    pub organizer: Option<Address>,
    pub total_supply: Option<u32>,
    pub minted_count: Option<u32>,
    pub royalty_bps: Option<u32>,
    pub previous_owner: Option<Address>,
    pub royalty: Option<i128>,
    pub seller_payout: Option<i128>,
}

impl ContractEventPayload {
    fn for_event(meta: &EventMeta) -> Self {
        Self {
            event_id: meta.event_id,
            ticket_id: None,
            ticket_owner: None,
            ticket_status: None,
            tier_name: None,
            ticket_price: None,
            is_listed_resale: None,
            resale_price: None,
            mint_timestamp: None,
            redeem_timestamp: None,
            event_name: Some(meta.name.clone()),
            organizer: Some(meta.organizer.clone()),
            total_supply: Some(meta.total_supply),
            minted_count: Some(meta.minted_count),
            royalty_bps: Some(meta.royalty_bps),
            previous_owner: None,
            royalty: None,
            seller_payout: None,
        }
    }

    fn for_ticket(
        env: &Env,
        ticket: &Ticket,
        previous_owner: Option<Address>,
        royalty: Option<i128>,
        seller_payout: Option<i128>,
    ) -> Self {
        let status = match ticket.status {
            TicketStatus::Valid => "Valid",
            TicketStatus::Claimable => "Claimable",
            TicketStatus::Used => "Used",
            TicketStatus::ProofNFT => "ProofNFT",
        };

        Self {
            event_id: ticket.event_id,
            ticket_id: Some(ticket.id),
            ticket_owner: Some(ticket.current_owner.clone()),
            ticket_status: Some(String::from_str(env, status)),
            tier_name: Some(ticket.tier_name.clone()),
            ticket_price: Some(ticket.price),
            is_listed_resale: Some(ticket.is_listed_resale),
            resale_price: Some(ticket.resale_price),
            mint_timestamp: Some(ticket.mint_timestamp),
            redeem_timestamp: Some(ticket.redeem_timestamp),
            event_name: None,
            organizer: None,
            total_supply: None,
            minted_count: None,
            royalty_bps: None,
            previous_owner,
            royalty,
            seller_payout,
        }
    }
}

#[contracttype]
pub enum DataKey {
    EventInfo,
    Ticket(u64),
    TicketCounter,
    ClaimLink(String),
    StorageVersion,
}

#[contract]
pub struct EventTicketContract;

const STORAGE_VERSION: u32 = 1;

#[contractimpl]
impl EventTicketContract {
    /// Initialize Event Metadata & Royalty structure
    pub fn initialize(
        env: Env,
        organizer: Address,
        name: String,
        total_supply: u32,
        royalty_bps: u32,
    ) {
        organizer.require_auth();

        if env.storage().instance().has(&DataKey::EventInfo) {
            panic!("Contract is already initialized");
        }
        if name.len() == 0 {
            panic!("Event name cannot be empty");
        }
        if total_supply == 0 {
            panic!("Event supply must be greater than zero");
        }
        if royalty_bps > 10_000 {
            panic!("Royalty rate cannot exceed 100 percent");
        }

        let event_info = EventMeta {
            event_id: 101,
            organizer,
            name,
            total_supply,
            minted_count: 0,
            royalty_bps,
        };

        env.storage()
            .instance()
            .set(&DataKey::EventInfo, &event_info);
        env.storage().instance().set(&DataKey::TicketCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_VERSION);
        env.events().publish(
            (symbol_short!("init"), event_info.event_id),
            ContractEventPayload::for_event(&event_info),
        );
    }

    /// Migrate legacy unversioned storage to the current schema version.
    pub fn migrate_storage(env: Env) -> u32 {
        let meta: EventMeta = env
            .storage()
            .instance()
            .get(&DataKey::EventInfo)
            .expect("Event is not initialized");
        meta.organizer.require_auth();

        let version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StorageVersion)
            .unwrap_or(0);
        if version == STORAGE_VERSION {
            return version;
        }
        if version != 0 {
            panic!("Unsupported storage version");
        }

        let _: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TicketCounter)
            .expect("Legacy ticket counter is missing");
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_VERSION);
        STORAGE_VERSION
    }

    /// Issue a new unique ticket digital asset / claimable balance
    pub fn mint_ticket(
        env: Env,
        buyer: Address,
        tier_name: String,
        price: i128,
        claim_secret_hash: String,
    ) -> u64 {
        let mut meta: EventMeta = env.storage().instance().get(&DataKey::EventInfo).unwrap();
        if price <= 0 {
            panic!("Ticket price must be greater than zero");
        }
        if meta.minted_count >= meta.total_supply {
            panic!("Event sold out");
        }
        if claim_secret_hash.len() > 0
            && env
                .storage()
                .persistent()
                .has(&DataKey::ClaimLink(claim_secret_hash.clone()))
        {
            panic!("Claim link is already in use");
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TicketCounter)
            .unwrap_or(0);
        if counter != u64::from(meta.minted_count) {
            panic!("Ticket counter and minted inventory are inconsistent");
        }
        counter = counter
            .checked_add(1)
            .expect("Ticket counter overflow");
        meta.minted_count = meta
            .minted_count
            .checked_add(1)
            .expect("Minted inventory overflow");

        let status = if claim_secret_hash.len() > 0 {
            TicketStatus::Claimable
        } else {
            TicketStatus::Valid
        };

        let ticket = Ticket {
            id: counter,
            event_id: meta.event_id,
            tier_name,
            original_buyer: buyer.clone(),
            current_owner: buyer,
            status,
            price,
            is_listed_resale: false,
            resale_price: 0,
            mint_timestamp: env.ledger().timestamp(),
            redeem_timestamp: 0,
            claim_secret_hash: claim_secret_hash.clone(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Ticket(counter), &ticket);
        env.storage().instance().set(&DataKey::EventInfo, &meta);
        env.storage()
            .instance()
            .set(&DataKey::TicketCounter, &counter);

        if claim_secret_hash.len() > 0 {
            env.storage()
                .persistent()
                .set(&DataKey::ClaimLink(claim_secret_hash), &counter);
        }

        env.events().publish(
            (symbol_short!("mint"), meta.event_id),
            ContractEventPayload::for_ticket(&env, &ticket, None, None, None),
        );

        counter
    }

    /// Claim ticket using unique secret link -> transfer ownership to user's wallet
    pub fn claim_ticket(env: Env, claim_secret_hash: String, new_owner: Address) -> bool {
        new_owner.require_auth();

        let ticket_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::ClaimLink(claim_secret_hash.clone()))
            .expect("Invalid or expired claim link");

        let mut ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .expect("Ticket not found");

        if ticket.status != TicketStatus::Claimable {
            panic!("Ticket already claimed or invalid status");
        }

        let previous_owner = ticket.current_owner.clone();
        ticket.current_owner = new_owner;
        ticket.status = TicketStatus::Valid;
        ticket.claim_secret_hash = String::from_str(&env, "");

        env.storage()
            .persistent()
            .set(&DataKey::Ticket(ticket_id), &ticket);
        env.storage()
            .persistent()
            .remove(&DataKey::ClaimLink(claim_secret_hash));
        env.events().publish(
            (symbol_short!("claim"), ticket.event_id),
            ContractEventPayload::for_ticket(
                &env,
                &ticket,
                Some(previous_owner),
                None,
                None,
            ),
        );

        true
    }

    /// Gatekeeper verification & Check-in: validates ticket and prevents double usage
    pub fn check_in_ticket(env: Env, organizer: Address, ticket_id: u64) -> TicketStatus {
        organizer.require_auth();

        let meta: EventMeta = env.storage().instance().get(&DataKey::EventInfo).unwrap();
        if meta.organizer != organizer {
            panic!("Unauthorized gatekeeper");
        }

        let mut ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .expect("Ticket not found");

        if ticket.status == TicketStatus::Used || ticket.status == TicketStatus::ProofNFT {
            panic!("DOUBLE USE PREVENTED: Ticket already redeemed!");
        }

        if ticket.status != TicketStatus::Valid {
            panic!("Ticket cannot be redeemed: Unclaimed or invalid");
        }

        // Convert ticket -> Proof of Attendance NFT
        ticket.status = TicketStatus::ProofNFT;
        ticket.redeem_timestamp = env.ledger().timestamp();
        ticket.is_listed_resale = false;
        ticket.resale_price = 0;

        env.storage()
            .persistent()
            .set(&DataKey::Ticket(ticket_id), &ticket);
        env.events().publish(
            (symbol_short!("checkin"), ticket.event_id),
            ContractEventPayload::for_ticket(&env, &ticket, None, None, None),
        );

        TicketStatus::ProofNFT
    }

    /// Resale listing with cap check
    pub fn list_resale(env: Env, seller: Address, ticket_id: u64, resale_price: i128) {
        seller.require_auth();

        let mut ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .expect("Ticket not found");

        if ticket.current_owner != seller {
            panic!("Not ticket owner");
        }

        if ticket.status != TicketStatus::Valid {
            panic!("Only valid tickets can be listed for resale");
        }
        if ticket.is_listed_resale {
            panic!("Ticket is already listed for resale");
        }

        if ticket.price <= 0 || resale_price <= 0 {
            panic!("Ticket and resale prices must be greater than zero");
        }

        // Anti-scalping cap: Max 150% of original price.
        let max_resale = ticket
            .price
            .checked_mul(150)
            .map(|price| price / 100)
            .unwrap_or(i128::MAX);
        if resale_price > max_resale {
            panic!("Resale price exceeds anti-scalping price cap (150%)");
        }

        ticket.is_listed_resale = true;
        ticket.resale_price = resale_price;

        env.storage()
            .persistent()
            .set(&DataKey::Ticket(ticket_id), &ticket);
        env.events().publish(
            (symbol_short!("listing"), ticket.event_id),
            ContractEventPayload::for_ticket(&env, &ticket, None, None, None),
        );
    }

    /// Transfer a listed ticket and record royalty payout values in an event
    pub fn buy_resale(env: Env, buyer: Address, ticket_id: u64) {
        buyer.require_auth();

        let mut ticket: Ticket = env
            .storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .expect("Ticket not found");

        if !ticket.is_listed_resale {
            panic!("Ticket is not listed for resale");
        }
        if ticket.status != TicketStatus::Valid {
            panic!("Only valid tickets can be purchased");
        }
        if ticket.current_owner == buyer {
            panic!("Ticket owner cannot purchase their own listing");
        }

        let meta: EventMeta = env.storage().instance().get(&DataKey::EventInfo).unwrap();

        // Calculate Royalty
        let royalty_bps = meta.royalty_bps as i128;
        let royalty = (ticket.resale_price / 10_000) * royalty_bps
            + ((ticket.resale_price % 10_000) * royalty_bps) / 10_000;
        let seller_payout = ticket.resale_price - royalty;

        let previous_owner = ticket.current_owner.clone();
        ticket.current_owner = buyer.clone();
        ticket.is_listed_resale = false;
        ticket.resale_price = 0;

        env.storage()
            .persistent()
            .set(&DataKey::Ticket(ticket_id), &ticket);
        env.events().publish(
            (symbol_short!("resale"), ticket.event_id),
            ContractEventPayload::for_ticket(
                &env,
                &ticket,
                Some(previous_owner),
                Some(royalty),
                Some(seller_payout),
            ),
        );
    }

    /// Fetch ticket details
    pub fn get_ticket(env: Env, ticket_id: u64) -> Ticket {
        env.storage()
            .persistent()
            .get(&DataKey::Ticket(ticket_id))
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Events as _}, Address, Env, TryFromVal};

    fn setup_event(env: &Env, total_supply: u32, royalty_bps: u32) -> (Address, Address) {
        env.mock_all_auths();
        let contract_id = env.register_contract(None, EventTicketContract);
        let client = EventTicketContractClient::new(env, &contract_id);
        let organizer = Address::generate(env);

        client.initialize(
            &organizer,
            &String::from_str(env, "Event"),
            &total_supply,
            &royalty_bps,
        );

        (contract_id, organizer)
    }

    #[test]
    fn migration_versions_legacy_storage_without_losing_tickets() {
        let env = Env::default();
        let (contract_id, _) = setup_event(&env, 1, 500);
        let client = EventTicketContractClient::new(&env, &contract_id);
        let buyer = Address::generate(&env);
        client.mint_ticket(
            &buyer,
            &String::from_str(&env, "General"),
            &100,
            &String::from_str(&env, ""),
        );

        env.as_contract(&contract_id, || {
            env.storage().instance().remove(&DataKey::StorageVersion);
        });

        assert_eq!(client.migrate_storage(), STORAGE_VERSION);
        assert_eq!(client.migrate_storage(), STORAGE_VERSION);
        assert_eq!(client.get_ticket(&1).current_owner, buyer);
        assert_eq!(
            env.as_contract(&contract_id, || {
                env.storage().instance().get(&DataKey::StorageVersion)
            }),
            Some(STORAGE_VERSION)
        );
    }

    #[test]
    fn buy_resale_handles_large_royalty_calculation() {
        let env = Env::default();
        let (contract_id, _) = setup_event(&env, 1, 500);
        let client = EventTicketContractClient::new(&env, &contract_id);
        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);

        client.mint_ticket(
            &seller,
            &String::from_str(&env, "General"),
            &i128::MAX,
            &String::from_str(&env, ""),
        );
        client.list_resale(&seller, &1, &i128::MAX);
        client.buy_resale(&buyer, &1);

        assert_eq!(client.get_ticket(&1).current_owner, buyer);
    }

    #[test]
    fn mint_rejects_duplicate_claim_links() {
        let env = Env::default();
        let (contract_id, _) = setup_event(&env, 2, 500);
        let client = EventTicketContractClient::new(&env, &contract_id);
        let first_buyer = Address::generate(&env);
        let second_buyer = Address::generate(&env);
        let claim_secret_hash = String::from_str(&env, "unique-claim-hash");

        client.mint_ticket(
            &first_buyer,
            &String::from_str(&env, "General"),
            &100,
            &claim_secret_hash,
        );

        assert!(client
            .try_mint_ticket(
                &second_buyer,
                &String::from_str(&env, "General"),
                &100,
                &claim_secret_hash,
            )
            .is_err());

        assert_eq!(client.get_ticket(&1).current_owner, first_buyer);
    }

    #[test]
    fn check_in_cancels_resale_listing() {
        let env = Env::default();
        let (contract_id, organizer) = setup_event(&env, 1, 500);
        let client = EventTicketContractClient::new(&env, &contract_id);
        let seller = Address::generate(&env);

        client.mint_ticket(
            &seller,
            &String::from_str(&env, "General"),
            &100,
            &String::from_str(&env, ""),
        );
        client.list_resale(&seller, &1, &100);
        client.check_in_ticket(&organizer, &1);

        let ticket = client.get_ticket(&1);
        assert_eq!(ticket.status, TicketStatus::ProofNFT);
        assert!(!ticket.is_listed_resale);
        assert_eq!(ticket.resale_price, 0);
        assert!(client.try_buy_resale(&Address::generate(&env), &1).is_err());
        assert_eq!(client.get_ticket(&1).current_owner, seller);
    }

    #[test]
    fn state_changing_calls_reject_missing_authorization() {
        let env = Env::default();
        let (contract_id, organizer) = setup_event(&env, 2, 500);
        let client = EventTicketContractClient::new(&env, &contract_id);
        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);
        let claim_recipient = Address::generate(&env);
        let claim_hash = String::from_str(&env, "auth-test-claim");

        client.mint_ticket(
            &seller,
            &String::from_str(&env, "General"),
            &100,
            &String::from_str(&env, ""),
        );
        client.mint_ticket(
            &seller,
            &String::from_str(&env, "General"),
            &100,
            &claim_hash,
        );
        env.mock_auths(&[]);

        assert!(client.try_check_in_ticket(&organizer, &1).is_err());
        assert!(client.try_list_resale(&seller, &1, &100).is_err());
        assert!(client.try_buy_resale(&buyer, &1).is_err());
        assert!(client
            .try_claim_ticket(&claim_hash, &claim_recipient)
            .is_err());
    }
}
