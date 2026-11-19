use super::*;

#[test]
fn comparison_of_identical_plans_has_no_changes() -> Result<(), Box<dyn std::error::Error>> {
    let comparison = manager()
        .compare_plans("shared", "shared")
        .ok_or("both plans exist")?;
    assert!(comparison
        .limit_changes
        .iter()
        .all(|change| change.direction == ChangeDirection::NoChange));
    assert!(matches!(comparison.price_change, PriceChange::NoChange));
    assert!(comparison.feature_changes.added.is_empty());
    assert!(comparison.feature_changes.removed.is_empty());
    Ok(())
}

#[test]
fn comparison_to_or_from_a_custom_plan_is_flagged() -> Result<(), Box<dyn std::error::Error>> {
    let to_custom = manager()
        .compare_plans("shared", "enterprise")
        .ok_or("both plans exist")?;
    assert!(matches!(to_custom.price_change, PriceChange::ToCustom));

    let from_custom = manager()
        .compare_plans("enterprise", "shared")
        .ok_or("both plans exist")?;
    assert!(matches!(from_custom.price_change, PriceChange::FromCustom));

    assert!(manager().compare_plans("shared", "ghost").is_none());
    assert!(manager().compare_plans("ghost", "shared").is_none());
    Ok(())
}
