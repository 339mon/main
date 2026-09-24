#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn bytes32(env: &Env, value: u8) -> BytesN<32> {
    let mut data = [0u8; 32];
    data[31] = value;
    BytesN::from_array(env, &data)
}

#[test]
fn registers_lineage_with_bounded_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    let parent = bytes32(&env, 1);

    client.init(&admin);
    client.register_source(&actor, &bytes32(&env, 2), &bytes32(&env, 3), &parent);

    let lineage = client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [parent.clone()]),
        &bytes32(&env, 4),
        &Symbol::new(&env, "crop"),
        &bytes32(&env, 5),
        1,
    );

    assert_eq!(lineage.output_digest, bytes32(&env, 5));
    assert_eq!(lineage.depth, 1);
    assert_eq!(client.get_lineage_children_count(&parent), 1);
    let page = client.list_lineage_children(&parent, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.children.get(0).unwrap(), bytes32(&env, 5));
}

#[test]
#[should_panic(expected = "Error(Contract, #60)")]
fn rejects_excessive_fanout() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    client.init(&admin);

    let parents = soroban_sdk::Vec::from_array(
        &env,
        [
            bytes32(&env, 1),
            bytes32(&env, 2),
            bytes32(&env, 3),
            bytes32(&env, 4),
            bytes32(&env, 5),
        ],
    );

    client.register_lineage(
        &actor,
        &parents,
        &bytes32(&env, 6),
        &Symbol::new(&env, "compose"),
        &bytes32(&env, 7),
        1,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #58)")]
fn rejects_self_referential_lineage() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    client.init(&admin);

    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [bytes32(&env, 1)]),
        &bytes32(&env, 2),
        &Symbol::new(&env, "crop"),
        &bytes32(&env, 1),
        1,
    );
}

#[test]
fn paginates_lineage_children_in_registration_order() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    let parent = bytes32(&env, 10);

    client.init(&admin);
    client.register_source(&actor, &bytes32(&env, 11), &bytes32(&env, 12), &parent);

    let child_a = bytes32(&env, 20);
    let child_b = bytes32(&env, 21);
    let child_c = bytes32(&env, 22);

    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [parent.clone()]),
        &bytes32(&env, 30),
        &Symbol::new(&env, "crop"),
        &child_a,
        1,
    );
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [parent.clone()]),
        &bytes32(&env, 31),
        &Symbol::new(&env, "blur"),
        &child_b,
        1,
    );
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [parent.clone()]),
        &bytes32(&env, 32),
        &Symbol::new(&env, "redact"),
        &child_c,
        1,
    );

    assert_eq!(client.get_lineage_children_count(&parent), 3);

    let page1 = client.list_lineage_children(&parent, &0, &2);
    assert_eq!(page1.total, 3);
    assert_eq!(page1.next_offset, 2);
    assert_eq!(page1.children.len(), 2);
    assert_eq!(page1.children.get(0).unwrap(), child_a);
    assert_eq!(page1.children.get(1).unwrap(), child_b);

    let page2 = client.list_lineage_children(&parent, &page1.next_offset, &2);
    assert_eq!(page2.total, 3);
    assert_eq!(page2.next_offset, 3);
    assert_eq!(page2.children.len(), 1);
    assert_eq!(page2.children.get(0).unwrap(), child_c);

    let empty = client.list_lineage_children(&parent, &3, &2);
    assert_eq!(empty.total, 3);
    assert_eq!(empty.next_offset, 3);
    assert_eq!(empty.children.len(), 0);

    let unknown = client.list_lineage_children(&bytes32(&env, 99), &0, &10);
    assert_eq!(unknown.total, 0);
    assert_eq!(unknown.children.len(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #68)")]
fn rejects_zero_children_page_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin);

    client.list_lineage_children(&bytes32(&env, 1), &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #68)")]
fn rejects_oversized_children_page_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin);

    client.list_lineage_children(&bytes32(&env, 1), &0, &(MAX_LINEAGE_CHILDREN_PAGE + 1));
}
