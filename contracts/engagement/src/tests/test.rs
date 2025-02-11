#![cfg(test)]

extern crate std;

use crate::storage::types::{ Escrow, Milestone };
use crate::token::token::{ Token, TokenClient };
use crate::contract::EngagementContract;
use crate::contract::EngagementContractClient;
use soroban_sdk::Vec;
use soroban_sdk::{ testutils::Address as _, vec, Address, Env, IntoVal, String };

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
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3;
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address,
        service_provider: service_provider_address,
        platform_address: platform_address,
        platform_fee: platform_fee,
        milestones: milestones,
        release_signer: release_signer_address,
        dispute_resolver: dispute_resolver_address,
        trustline: usdc_token.address,
    };

    let initialized_escrow = engagement_approver.initialize_escrow(&escrow_properties);

    let escrow = engagement_approver.get_escrow();
    assert_eq!(escrow.engagement_id, initialized_escrow.engagement_id);
    assert_eq!(escrow.approver, escrow_properties.approver);
    assert_eq!(escrow.service_provider, escrow_properties.service_provider);
    assert_eq!(escrow.platform_address, escrow_properties.platform_address);
    assert_eq!(escrow.platform_fee, platform_fee);
    assert_eq!(escrow.milestones, escrow_properties.milestones);
    assert_eq!(escrow.release_signer, escrow_properties.release_signer);
    assert_eq!(escrow.dispute_resolver, escrow_properties.dispute_resolver);

    let result = engagement_approver.try_initialize_escrow(&escrow_properties);
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

    let platform_fee = (0.3 * ((10i128).pow(18) as f64)) as i128;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address,
        service_provider: service_provider_address,
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: initial_milestones,
        release_signer: release_signer_address,
        dispute_resolver: dispute_resolver_address,
        trustline: usdc_token.address.clone(),
    };

    let initialized_escrow = engagement_approver.initialize_escrow(&escrow_properties);

    // Verify escrow was initialized
    let escrow = engagement_approver.get_escrow();
    assert_eq!(escrow.engagement_id, initialized_escrow.engagement_id);

    // Create new values for updating the escrow
    let new_approver_address = Address::generate(&env);
    let new_service_provider = Address::generate(&env);
    let new_release_signer = Address::generate(&env);
    let new_dispute_resolver = Address::generate(&env);
    let new_platform_fee = (0.5 * ((10i128).pow(18) as f64)) as i128;

    let new_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Updated first milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Updated second milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "New third milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    // Test unauthorized access (should fail)
    let unauthorized_address = Address::generate(&env);
    env.mock_all_auths();

    let escrow_properties_v2: Escrow = Escrow {
        engagement_id: initialized_escrow.engagement_id.clone(),
        title: String::from_str(&env, "Updated Escrow"),
        description: String::from_str(&env, "Updated Escrow Description"),
        approver: new_approver_address.clone(),
        service_provider: new_service_provider.clone(),
        platform_address: unauthorized_address.clone(),
        platform_fee: new_platform_fee,
        milestones: new_milestones.clone(),
        release_signer: new_release_signer.clone(),
        dispute_resolver: new_dispute_resolver.clone(),
        trustline: usdc_token.address.clone(),
    };

    let result = engagement_approver.try_change_escrow_properties(&unauthorized_address, &escrow_properties_v2);
    assert!(result.is_err());
    // Update escrow with authorized platform_address
    env.mock_all_auths();

    let escrow_properties_v3: Escrow = Escrow {
        engagement_id: initialized_escrow.engagement_id.clone(),
        title: String::from_str(&env, "Updated Escrow"),
        description: String::from_str(&env, "Updated Escrow Description"),
        approver: new_approver_address.clone(),
        service_provider: new_service_provider.clone(),
        platform_address: platform_address.clone(),
        platform_fee: new_platform_fee,
        milestones: new_milestones.clone(),
        release_signer: new_release_signer.clone(),
        dispute_resolver: new_dispute_resolver.clone(),
        trustline: usdc_token.address,
    };

    engagement_approver.change_escrow_properties(&platform_address,&escrow_properties_v3);

    // Verify updated escrow properties
    let updated_escrow = engagement_approver.get_escrow();
    assert_eq!(updated_escrow.engagement_id, engagement_id);
    assert_eq!(updated_escrow.approver, escrow_properties_v3.approver);
    assert_eq!(updated_escrow.service_provider, escrow_properties_v3.service_provider);
    assert_eq!(updated_escrow.platform_address, escrow_properties_v3.platform_address);
    assert_eq!(updated_escrow.platform_fee, new_platform_fee);
    assert_eq!(updated_escrow.milestones, escrow_properties_v3.milestones);
    assert_eq!(updated_escrow.release_signer, escrow_properties_v3.release_signer);
    assert_eq!(updated_escrow.dispute_resolver, escrow_properties_v3.dispute_resolver);
}

#[test]
fn test_change_milestone_status_and_approved_flag() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = (0.3 * ((10i128).pow(18) as f64)) as i128;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "test_engagement");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    // Change milestone status (valid case)
    let new_status = String::from_str(&env, "completed");
    engagement_approver.change_milestone_status(
        &(0 as u32), // Milestone index
        &new_status,
        &service_provider_address
    );

    // Verify milestone status change
    let updated_escrow = engagement_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, new_status);

    // Change milestone approved_flag (valid case)
    engagement_approver.change_milestone_approved_flag(&(0), &true, &approver_address);

    // Verify milestone approved_flag change
    let final_escrow = engagement_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().approved_flag);

    // Invalid index test
    let invalid_index = 10;
    let new_status = String::from_str(&env, "completed");

    // Test for `change_status` with invalid index
    let result = engagement_approver.try_change_milestone_status(
        &invalid_index,
        &new_status,
        &service_provider_address
    );
    assert!(result.is_err());

    // Test for `change_approved_flag` with invalid index
    let result = engagement_approver.try_change_milestone_approved_flag(
        &invalid_index,
        &true,
        &approver_address
    );
    assert!(result.is_err());

    // Test only authorized party can perform the function
    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let result = engagement_approver.try_change_milestone_status(
        &(0),
        &new_status,
        &unauthorized_address
    );
    assert!(result.is_err());

    // Test for `change_approved_flag` by invalid approver
    let result = engagement_approver.try_change_milestone_approved_flag(
        &(0),
        &true,
        &unauthorized_address
    );
    assert!(result.is_err());

    //Escrow Test with no milestone
    let escrow_properties_v2: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: vec![&env],
        release_signer: release_signer_address,
        dispute_resolver: dispute_resolver_address,
        trustline: usdc_token.address,
    };

    engagement_approver.change_escrow_properties(&platform_address, &escrow_properties_v2);
    // Test for `change_status` on escrow with no milestones
    let result = engagement_approver.try_change_milestone_status(
        &(0),
        &new_status,
        &service_provider_address
    );
    assert!(result.is_err());

    // Test for `change_approved_flag` on escrow with no milestones
    let result = engagement_approver.try_change_milestone_approved_flag(&(0 ), &true, &approver_address);
    assert!(result.is_err());
}

#[test]
fn test_release_milestone_payment_successful() {
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
            approved_flag: true,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Completed"),
            approved_flag: true,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);

    let engagement_id = String::from_str(&env, "test_escrow_1");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&engagement_contract_address, &(amount as i128));

    let initial_contract_balance = usdc_token.balance(&engagement_contract_address);
    
    engagement_approver.release_milestone_payment(
        &release_signer_address,
        &trustless_work_address,
        &(0)
    );

    let total_amount = milestones.get(0).unwrap().amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * (platform_fee as i128)) / (10000 as i128);
    let service_provider_amount = (total_amount -
        (trustless_work_commission + platform_commission)) as i128;

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
        "Service Provider received incorrect amount"
    );

    assert_eq!(
        usdc_token.balance(&engagement_contract_address),
        initial_contract_balance - total_amount,
        "Contract balance is incorrect after claiming earnings"
    );
}

//test claim escrow earnings in failure scenarios
// Scenario 1: Escrow with no milestones:

#[test]
fn test_release_milestone_payment_no_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);

    let engagement_id_no_milestones = String::from_str(&env, "test_no_milestones");
    let platform_fee = 30;

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_no_milestones.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: vec![&env],
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    // Try to claim earnings with no milestones (should fail)
    let result = engagement_approver.try_release_milestone_payment(
        &release_signer_address,
        &platform_address,
        &(0)
    );
    assert!(
        result.is_err(),
        "Should fail when no milestones are defined"
    );
    assert!(result.is_err(), "Should fail when no milestones are defined");
}

// Scenario 2: Milestones incomplete
#[test]
fn test_release_milestone_payment_milestones_incomplete() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);

    let engagement_id_incomplete = String::from_str(&env, "test_incomplete_milestones");
    let milestones_incomplete = vec![&env, Milestone {
        description: String::from_str(&env, "Incomplete milestone"),
        status: String::from_str(&env, "Pending"),
        approved_flag: false,
        amount: 100_000,
        dispute_flag: false,
        release_flag: false,
        resolved_flag: false,
    }];

    let platform_fee = 30;

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_incomplete.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: milestones_incomplete.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    // Try to claim earnings with incomplete milestones (should fail)
    let result = engagement_approver.try_release_milestone_payment(
        &release_signer_address,
        &platform_address,
        &(0)
    );
    assert!(
        result.is_err(),
        "Should fail when milestones are not completed"
    );
    assert!(result.is_err(), "Should fail when milestones are not completed");
}


#[test]
fn test_dispute_resolution_process() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let amount: i128 = 100_000_000;
    let platform_fee = 30;

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    // Create milestone
    let milestone = Milestone {
        description: String::from_str(&env, "Test Milestone"),
        status: String::from_str(&env, "in_progress"),
        approved_flag: false,
        amount: amount,
        dispute_flag: false, 
        release_flag: false,
        resolved_flag: false,
    };

    let mut milestones = Vec::new(&env);
    milestones.push_back(milestone);

    let engagement_id = String::from_str(&env, "test_resolution");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&admin, &(amount as i128));
    usdc_token.transfer(&admin, &engagement_contract_address, &(amount as i128));

    // Verify initial state
    let escrow_balance = usdc_token.balance(&engagement_contract_address);
    assert_eq!(escrow_balance, amount as i128);

    // Change milestone dispute flag
    engagement_approver.change_milestone_dispute_flag(&(0 as i128));

    // Verify milestone dispute flag changed
    let disputed_escrow = engagement_approver.get_escrow();
    let disputed_milestone = disputed_escrow.milestones.get(0).unwrap();
    assert_eq!(disputed_milestone.dispute_flag, true);

    // Resolve dispute
    let approver_amount: i128 = 40_000_000;
    let provider_amount: i128 = 60_000_000;
    let total_amount = approver_amount + provider_amount;

    engagement_approver.resolving_milestone_disputes(
        &dispute_resolver_address,
        &0, // milestone_index
        &approver_amount,
        &provider_amount,
        &trustless_work_address
    );

    let expected_tw_fee = (total_amount * 30) / 10000; // 0.3%
    let expected_platform_fee = (total_amount * platform_fee) / 10000;

    let expected_approver = approver_amount - (approver_amount * (expected_tw_fee + expected_platform_fee)) / total_amount;
    let expected_provider = provider_amount - (provider_amount * (expected_tw_fee + expected_platform_fee)) / total_amount;

    assert_eq!(usdc_token.balance(&engagement_contract_address), 0);
    assert_eq!(usdc_token.balance(&trustless_work_address), expected_tw_fee);
    assert_eq!(usdc_token.balance(&platform_address), expected_platform_fee);
    assert_eq!(usdc_token.balance(&approver_address), expected_approver);
    assert_eq!(usdc_token.balance(&service_provider_address), expected_provider);

    let final_escrow = engagement_approver.get_escrow();
    let resolved_milestone = final_escrow.milestones.get(0).unwrap();
    assert_eq!(resolved_milestone.status, String::from_str(&env, "resolved"));
}

#[test]
fn test_fund_escrow_successful_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 30;
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "12345");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&engagement_contract_address, &(amount as i128));
    usdc_token.mint(&release_signer_address, &(amount as i128));

    let amount_to_deposit: i128 = 100_000;

    engagement_approver.fund_escrow(&release_signer_address, &amount_to_deposit);

    let expected_result_amount: i128 = 100_100_000;

    assert_eq!(
        usdc_token.balance(&engagement_contract_address),
        expected_result_amount,
        "Escrow balance is incorrect"
    );
}

#[test]
fn test_fund_escrow_signer_insufficient_funds_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 30;
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "12345");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    usdc_token.mint(&engagement_contract_address, &(amount as i128));

    let signer_funds: i128 = 100_000;
    usdc_token.mint(&release_signer_address, &(signer_funds as i128));

    let amount_to_deposit: i128 = 180_000;

    let result = engagement_approver.try_fund_escrow(&release_signer_address, &amount_to_deposit);

    assert!(result.is_err(), "Should fail when the signer has insufficient funds");
}

#[test]
fn test_fund_escrow_dispute_flag_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 30;
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 100_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "12321");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(), 
        platform_address: platform_address,
        platform_fee: platform_fee,
        milestones: milestones,
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);
    engagement_approver.change_milestone_dispute_flag(&(0 as i128));

    let amount_to_deposit: i128 = 80_000;

    let result = engagement_approver.try_fund_escrow(&release_signer_address, &amount_to_deposit);

    assert!(result.is_err(), "Should fail when the dispute approved_flag is true");
}

#[test]
fn test_change_milestone_dispute_flag() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let platform_fee = 30;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 50_000_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            approved_flag: false,
            amount: 50_000_000,
            dispute_flag: false,
            release_flag: false,
            resolved_flag: false,
        }
    ];

    let engagement_contract_address = env.register_contract(None, EngagementContract);
    let engagement_approver = EngagementContractClient::new(&env, &engagement_contract_address);
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "test_milestone_dispute");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        trustline: usdc_token.address.clone(),
    };

    engagement_approver.initialize_escrow(&escrow_properties);

    engagement_approver.change_milestone_dispute_flag(&(0 as i128));
    
    let escrow = engagement_approver.get_escrow();
    let milestone = escrow.milestones.get(0).unwrap();
    assert!(milestone.dispute_flag, "First milestone dispute flag should be true");
    
    let milestone2 = escrow.milestones.get(1).unwrap();
    assert!(!milestone2.dispute_flag, "Second milestone dispute flag should remain false");

    let result = engagement_approver.try_change_milestone_dispute_flag(&(5 as i128));
    assert!(result.is_err(), "Should fail with invalid milestone index");

    let result = engagement_approver.try_change_milestone_dispute_flag(&(0 as i128));
    assert!(result.is_err(), "Should fail when milestone is already in dispute");
}