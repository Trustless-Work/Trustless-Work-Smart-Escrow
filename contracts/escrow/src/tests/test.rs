#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    vec, Address, Env, IntoVal, String,
};
use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use crate::storage::types::Flags;
use crate::storage::types::Roles;
use crate::storage::types::Trustline;
use crate::storage::types::{Escrow, Milestone};
use crate::token::token::{Token, TokenClient};

fn create_usdc_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = TokenClient::new(e, &e.register_contract(None, Token {}));
    token.initialize(admin, &7, &"USDC".into_val(e), &"USDC".into_val(e));
    token
}

#[test]
fn test_initialize_excrow() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);
    let platform_fee = 3;
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones,
        flags,
        trustline,
        receiver_memo: 0,
    };

    let initialized_escrow = escrow_approver.initialize_escrow(&escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.engagement_id, initialized_escrow.engagement_id);
    assert_eq!(escrow.roles.approver, escrow_properties.roles.approver);
    assert_eq!(escrow.roles.service_provider, escrow_properties.roles.service_provider);
    assert_eq!(escrow.roles.platform_address, escrow_properties.roles.platform_address);
    assert_eq!(escrow.amount, amount);
    assert_eq!(escrow.platform_fee, platform_fee);
    assert_eq!(escrow.milestones, escrow_properties.milestones);
    assert_eq!(escrow.roles.release_signer, escrow_properties.roles.release_signer);
    assert_eq!(escrow.roles.dispute_resolver, escrow_properties.roles.dispute_resolver);
    assert_eq!(escrow.roles.receiver, escrow_properties.roles.receiver);
    assert_eq!(escrow.receiver_memo, escrow_properties.receiver_memo);

    let result = escrow_approver.try_initialize_escrow(&escrow_properties);
    assert!(result.is_err());
}

#[test]
fn test_change_escrow_properties() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let amount: i128 = 100_000_000;
    let platform_fee = (0.3 * 10i128.pow(18) as f64) as i128;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let engagement_id = String::from_str(&env, "test_escrow_2");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Create a new updated escrow properties
    let new_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Updated"),
        description: String::from_str(&env, "Test Escrow Description Updated"),
        roles,
        amount: amount * 2,
        platform_fee: platform_fee * 2,
        milestones: new_milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    // Update escrow properties
    let _updated_escrow = escrow_approver.change_escrow_properties(
        &platform_address,
        &updated_escrow_properties,
    );

    // Verify updated escrow properties
    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.title, updated_escrow_properties.title);
    assert_eq!(escrow.description, updated_escrow_properties.description);
    assert_eq!(escrow.amount, updated_escrow_properties.amount);
    assert_eq!(escrow.platform_fee, updated_escrow_properties.platform_fee);
    assert_eq!(escrow.milestones, updated_escrow_properties.milestones);
    assert_eq!(
        escrow.roles.release_signer,
        updated_escrow_properties.roles.release_signer
    );
    assert_eq!(
        escrow.roles.dispute_resolver,
        updated_escrow_properties.roles.dispute_resolver
    );
    assert_eq!(escrow.roles.receiver, updated_escrow_properties.roles.receiver);
    assert_eq!(escrow.receiver_memo, updated_escrow_properties.receiver_memo);

    // Try to update escrow properties without platform address (should fail)
    let non_platform_address = Address::generate(&env);
    let result = escrow_approver.try_change_escrow_properties(
        &non_platform_address,
        &updated_escrow_properties,
    );
    assert!(result.is_err());
}

#[test]
fn test_change_milestone_status_and_approved_flag() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = (0.3 * 10i128.pow(18) as f64) as i128;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    // Change milestone status (valid case)
    let new_status = String::from_str(&env, "completed");
    let new_evidence = Some(String::from_str(&env, "New evidence"));
    escrow_approver.change_milestone_status(
        &(0 as i128), 
        &new_status,
        &new_evidence,
        &service_provider_address,
    );

    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, new_status);
    assert_eq!(updated_escrow.milestones.get(0).unwrap().evidence, String::from_str(&env, "New evidence"));

    // Change milestone approved_flag (valid case)
    escrow_approver.change_milestone_flag(&(0 as i128), &true, &approver_address);

    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().approved_flag);

    let invalid_index = 10 as i128;
    let new_status = String::from_str(&env, "completed");
    let new_evidence = Some(String::from_str(&env, "New evidence"));

    let result = escrow_approver.try_change_milestone_status(
        &invalid_index,
        &new_status,
        &new_evidence,
        &service_provider_address,
    );
    assert!(result.is_err());

    let result =
        escrow_approver.try_change_milestone_flag(&invalid_index, &true, &approver_address);
    assert!(result.is_err());

    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let result = escrow_approver.try_change_milestone_status(
        &(0 as i128),
        &new_status,
        &new_evidence,
        &unauthorized_address,
    );
    assert!(result.is_err());

    // Test for `change_approved_flag` by invalid approver
    let result =
        escrow_approver.try_change_milestone_flag(&(0 as i128), &true, &unauthorized_address);
    assert!(result.is_err());

    let new_escrow_contract_address = env.register_contract(None, EscrowContract);
    let new_escrow_approver = EscrowContractClient::new(&env, &new_escrow_contract_address);

    //Escrow Test with no milestone
    let escrow_properties_v2: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Updated Escrow"),
        description: String::from_str(&env, "Updated Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: vec![&env],
        flags,
        trustline,
        receiver_memo: 0,
    };

    new_escrow_approver.initialize_escrow(&escrow_properties_v2);
    
    let result = new_escrow_approver.try_change_milestone_status(
        &(0 as i128),
        &new_status,
        &new_evidence,
        &service_provider_address,
    );
    assert!(result.is_err());

    let result = new_escrow_approver.try_change_milestone_flag(&(0 as i128), &true, &approver_address);
    assert!(result.is_err());
}

#[test]
fn test_distribute_escrow_earnings_successful_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.mint(&approver_address, &(amount as i128));

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: true,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: true,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_1");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&escrow_contract_address, &(amount as i128));

    escrow_approver
        .distribute_escrow_earnings(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let receiver_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&platform_address),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&_receiver_address),
        receiver_amount,
        "Receiver received incorrect amount"
    );

    assert_eq!(
        usdc_token.balance(&service_provider_address),
        0,
        "Service Provider should have zero balance when using separate receiver"
    );

    assert_eq!(
        usdc_token.balance(&escrow_contract_address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

//test claim escrow earnings in failure scenarios
// Scenario 1: Escrow with no milestones:
#[test]
fn test_distribute_escrow_earnings_no_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id_no_milestones = String::from_str(&env, "test_no_milestones");
    let amount: i128 = 100_000_000;
    let platform_fee = 30;

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_no_milestones.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: vec![&env],
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    // Try to claim earnings with no milestones (should fail)
    let result = escrow_approver
        .try_distribute_escrow_earnings(&release_signer_address, &platform_address);
    assert!(result.is_err());
}

// Scenario 2: Milestones incomplete
#[test]
fn test_distribute_escrow_earnings_milestones_incomplete() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id_incomplete_milestones =
        String::from_str(&env, "test_incomplete_milestones");
    let amount: i128 = 100_000_000;
    let platform_fee = 30;

    // Define milestones with one not approved
    let incomplete_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: true,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false, // Not approved yet
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_incomplete_milestones.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: incomplete_milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&escrow_contract_address, &(amount as i128));

    // Try to distribute earnings with incomplete milestones (should fail)
    let result = escrow_approver
        .try_distribute_escrow_earnings(&release_signer_address, &platform_address);
    assert!(result.is_err());
}

#[test]
fn test_distribute_escrow_earnings_same_receiver_as_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    // Use service_provider_address as receiver to test same-address case
    let _receiver_address = service_provider_address.clone();

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.mint(&approver_address, &(amount as i128));

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: true,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(), // Set to service_provider to test same-address case
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_same_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&escrow_contract_address, &(amount as i128));

    escrow_approver
        .distribute_escrow_earnings(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let service_provider_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&platform_address),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&service_provider_address),
        service_provider_amount,
        "Service Provider should receive funds when receiver is set to same address"
    );

    assert_eq!(
        usdc_token.balance(&escrow_contract_address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

#[test]
fn test_distribute_escrow_earnings_invalid_receiver_fallback() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    
    // Create a valid but separate receiver address
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.mint(&approver_address, &(amount as i128));

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: true,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(), // Different receiver address than service provider
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&escrow_contract_address, &(amount as i128));

    escrow_approver
        .distribute_escrow_earnings(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let receiver_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&platform_address),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    // Funds should go to the receiver (not service provider)
    assert_eq!(
        usdc_token.balance(&_receiver_address),
        receiver_amount,
        "Receiver should receive funds when set to a different address than service provider"
    );

    // The service provider should not receive funds when a different receiver is set
    assert_eq!(
        usdc_token.balance(&service_provider_address),
        0,
        "Service provider should not receive funds when a different receiver is set"
    );

    assert_eq!(
        usdc_token.balance(&escrow_contract_address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

#[test]
fn test_dispute_flag_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_dispute_flag");
    let amount: i128 = 100_000_000;
    let platform_fee = 30;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert!(!escrow.flags.dispute_flag);

    escrow_approver.change_dispute_flag();

    let escrow_after_change = escrow_approver.get_escrow();
    assert!(escrow_after_change.flags.dispute_flag);

    // Test block on funding during dispute
    usdc_token.mint(&approver_address, &(amount as i128));
    let result = escrow_approver.try_fund_escrow(&approver_address, &(amount as i128));
    assert!(result.is_err());

    // Test block on distributing earnings during dispute
    let result = escrow_approver
        .try_distribute_escrow_earnings(&release_signer_address, &platform_address);
    assert!(result.is_err());

    let _ = escrow_approver.try_change_dispute_flag();

    let escrow_after_second_change = escrow_approver.get_escrow();
    assert!(escrow_after_second_change.flags.dispute_flag);
}

#[test]
fn test_dispute_resolution_process() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.mint(&approver_address, &(amount as i128));

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: true,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_dispute_resolution");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);
    
    usdc_token.transfer(&approver_address, &escrow_contract_address, &amount);

    escrow_approver.change_dispute_flag();

    let escrow_with_dispute = escrow_approver.get_escrow();
    assert!(escrow_with_dispute.flags.dispute_flag);

    // Try to resolve dispute with incorrect dispute resolver (should fail)
    let result = escrow_approver.try_resolving_disputes(
        &approver_address,
        &(50_000_000 as i128),
        &(50_000_000 as i128),
        &trustless_work_address,
    );
    assert!(result.is_err());

    // Resolve dispute with correct dispute resolver (50/50 split)
    let approver_funds: i128 = 50_000_000;
    let service_provider_funds: i128 = 50_000_000;

    escrow_approver.resolving_disputes(
        &dispute_resolver_address,
        &approver_funds,
        &service_provider_funds,
        &trustless_work_address,
    );

    // Verify dispute was resolved
    let escrow_after_resolution = escrow_approver.get_escrow();
    assert!(!escrow_after_resolution.flags.dispute_flag);
    assert!(escrow_after_resolution.flags.resolved_flag);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let remaining_amount = total_amount - (trustless_work_commission + platform_commission);

    let platform_amount = platform_commission;
    let trustless_amount = trustless_work_commission;
    let service_provider_amount = (remaining_amount * service_provider_funds) / total_amount;
    let approver_amount = (remaining_amount * approver_funds) / total_amount;

    // Check balances
    assert_eq!(
        usdc_token.balance(&trustless_work_address),
        trustless_amount,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&platform_address),
        platform_amount,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&service_provider_address),
        service_provider_amount,
        "Service provider amount is incorrect"
    );

    assert_eq!(
        usdc_token.balance(&approver_address),
        approver_amount,
        "Approver amount is incorrect"
    );
}

#[test]
fn test_fund_escrow_successful_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.mint(&approver_address, &amount);

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_fund");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    // Check initial balances
    assert_eq!(usdc_token.balance(&approver_address), amount);
    assert_eq!(usdc_token.balance(&escrow_contract_address), 0);

    let deposit_amount = amount / 2;
    escrow_approver.fund_escrow(&approver_address, &deposit_amount);

    // Check balances after deposit
    assert_eq!(
        usdc_token.balance(&approver_address),
        amount - deposit_amount
    );
    assert_eq!(
        usdc_token.balance(&escrow_contract_address),
        deposit_amount
    );

    // Deposit remaining amount
    escrow_approver.fund_escrow(&approver_address, &deposit_amount);

    assert_eq!(usdc_token.balance(&approver_address), 0);
    assert_eq!(usdc_token.balance(&escrow_contract_address), amount);
}

#[test]
fn test_fund_escrow_fully_funded_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.mint(&approver_address, &(amount * 2));

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_fully_funded");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&escrow_contract_address, &amount);

    assert_eq!(usdc_token.balance(&escrow_contract_address), amount);

    let result = escrow_approver.try_fund_escrow(&approver_address, &(10_000_000 as i128));
    assert!(result.is_err());
}

#[test]
fn test_fund_escrow_signer_insufficient_funds_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    // Only mint a small amount to the approver
    let small_amount: i128 = 1_000_000;
    usdc_token.mint(&approver_address, &small_amount);

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_insufficient_funds");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    // Check initial balance
    assert_eq!(usdc_token.balance(&approver_address), small_amount);

    // Try to deposit more than the approver has (should fail)
    let result = escrow_approver.try_fund_escrow(&approver_address, &amount);
    assert!(result.is_err());

    // Verify balances didn't change
    assert_eq!(usdc_token.balance(&approver_address), small_amount);
    assert_eq!(usdc_token.balance(&escrow_contract_address), 0);
}

#[test]
fn test_fund_escrow_dispute_flag_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.mint(&approver_address, &amount);

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved_flag: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(),
    };

    let flags: Flags = Flags {
        dispute_flag: true,
        release_flag: false,
        resolved_flag: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.address.clone(),
        decimals: 10_000_000,
    };

    let escrow_contract_address = env.register_contract(None, EscrowContract);
    let escrow_approver = EscrowContractClient::new(&env, &escrow_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_dispute_error");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    escrow_approver.initialize_escrow(&escrow_properties);

    let result = escrow_approver.try_fund_escrow(&approver_address, &(10_000_000 as i128));
    assert!(result.is_err());

    assert_eq!(usdc_token.balance(&escrow_contract_address), 0);
}
