use crate::{EventTicketContract, EventTicketContractClient, TicketStatus};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn initializes_and_mints_ticket_for_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);
    let organizer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let event_name = String::from_str(&env, "Test event");

    client.initialize(&organizer, &event_name, &1, &500);
    let ticket_id = client.mint_ticket(
        &buyer,
        &String::from_str(&env, "General"),
        &2500,
        &String::from_str(&env, ""),
    );

    let ticket = client.get_ticket(&ticket_id);
    assert_eq!(ticket.event_id, 101);
    assert_eq!(ticket.current_owner, buyer);
    assert_eq!(ticket.status, TicketStatus::Valid);
}

#[test]
#[should_panic]
fn initialize_rejects_empty_event_name() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);

    client.initialize(
        &Address::generate(&env),
        &String::from_str(&env, ""),
        &1,
        &500,
    );
}

#[test]
#[should_panic]
fn initialize_rejects_zero_supply() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);

    client.initialize(
        &Address::generate(&env),
        &String::from_str(&env, "Test event"),
        &0,
        &500,
    );
}

#[test]
#[should_panic]
fn initialize_rejects_royalty_above_one_hundred_percent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);

    client.initialize(
        &Address::generate(&env),
        &String::from_str(&env, "Test event"),
        &1,
        &10_001,
    );
}

#[test]
fn initialize_accepts_one_hundred_percent_royalty() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);

    client.initialize(
        &Address::generate(&env),
        &String::from_str(&env, "Test event"),
        &1,
        &10_000,
    );
    let ticket_id = client.mint_ticket(
        &Address::generate(&env),
        &String::from_str(&env, "General"),
        &100,
        &String::from_str(&env, ""),
    );

    assert_eq!(client.get_ticket(&ticket_id).price, 100);
}

#[test]
#[should_panic]
fn initialize_cannot_be_called_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);
    let organizer = Address::generate(&env);

    client.initialize(&organizer, &String::from_str(&env, "First"), &1, &500);
    client.initialize(&organizer, &String::from_str(&env, "Second"), &1, &500);
}

#[test]
#[should_panic]
fn mint_rejects_zero_price() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);
    client.initialize(
        &Address::generate(&env),
        &String::from_str(&env, "Test event"),
        &1,
        &500,
    );

    client.mint_ticket(
        &Address::generate(&env),
        &String::from_str(&env, "General"),
        &0,
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic]
fn mint_rejects_negative_price() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);
    client.initialize(
        &Address::generate(&env),
        &String::from_str(&env, "Test event"),
        &1,
        &500,
    );

    client.mint_ticket(
        &Address::generate(&env),
        &String::from_str(&env, "General"),
        &-1,
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic]
fn mint_rejects_tickets_after_inventory_is_sold_out() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EventTicketContract);
    let client = EventTicketContractClient::new(&env, &contract_id);
    client.initialize(
        &Address::generate(&env),
        &String::from_str(&env, "Test event"),
        &1,
        &500,
    );
    let tier_name = String::from_str(&env, "General");
    let claim_hash = String::from_str(&env, "");

    client.mint_ticket(&Address::generate(&env), &tier_name, &100, &claim_hash);
    client.mint_ticket(&Address::generate(&env), &tier_name, &100, &claim_hash);
}
