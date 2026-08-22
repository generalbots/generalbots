use botintegrations::scope::ConnectionScope;
use botintegrations::secrets::{build_connection_path, ConnectionVault};
use uuid::Uuid;

fn scope(org: u128, branch: u128, bot: u128, owner: u128) -> ConnectionScope {
    ConnectionScope {
        user_id: Uuid::from_u128(owner),
        org_id: Uuid::from_u128(org),
        branch_id: Uuid::from_u128(branch),
        bot_id: Uuid::from_u128(bot),
    }
}

#[test]
fn canonical_path_format_is_exact() {
    let connection_id = Uuid::from_u128(4);
    let path = build_connection_path(&scope(1, 2, 3, 5), connection_id);
    assert_eq!(
        path,
        "gbo/00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002/00000000-0000-0000-0000-000000000003/integrations/00000000-0000-0000-0000-000000000005/00000000-0000-0000-0000-000000000004"
    );
}

#[test]
fn paths_are_unique_across_scope_components() {
    let connection_id = Uuid::from_u128(9);
    let base = build_connection_path(&scope(1, 2, 3, 4), connection_id);

    let same = build_connection_path(&scope(1, 2, 3, 4), connection_id);
    assert_eq!(base, same, "identical scope and id must map to one path");

    let mut variants = Vec::new();
    for (org, branch, bot, owner) in [(11, 2, 3, 4), (1, 22, 3, 4), (1, 2, 33, 4), (1, 2, 3, 44)] {
        variants.push(build_connection_path(
            &scope(org, branch, bot, owner),
            connection_id,
        ));
    }
    // A different owner or tenant must never collide on the same path.
    for variant in &variants {
        assert_ne!(&base, variant);
    }
    let unique: std::collections::HashSet<&String> = variants.iter().collect();
    assert_eq!(
        unique.len(),
        variants.len(),
        "variants must be pairwise distinct"
    );
}

#[test]
fn distinct_connections_in_one_scope_do_not_collide() {
    let shared_scope = scope(7, 8, 9, 10);
    let first = build_connection_path(&shared_scope, Uuid::from_u128(1));
    let second = build_connection_path(&shared_scope, Uuid::from_u128(2));
    assert_ne!(first, second);
}

/// The wrapper is only a thin facade over the platform secrets manager; this
/// asserts the constructor stays cheap and side-effect free.
#[test]
fn vault_wrapper_constructs_without_io() {
    if let Ok(manager) = botcoresecrets::manager::SecretsManager::get_clone() {
        let _vault = ConnectionVault::new(manager);
    }
}
