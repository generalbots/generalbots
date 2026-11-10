use super::*;

pub(crate) fn record_to_objective(record: ObjectiveRecord) -> Objective {
    Objective {
        id: record.id,
        organization_id: record.org_id,
        owner_id: record.owner_id,
        owner_name: None,
        parent_id: record.parent_id,
        title: record.title,
        description: record.description.unwrap_or_default(),
        period: record.period,
        period_start: record.period_start,
        period_end: record.period_end,
        status: ObjectiveStatus::from_label(&record.status),
        progress: record.progress.to_f32().unwrap_or(0.0),
        visibility: Visibility::from_label(&record.visibility),
        weight: record.weight.to_f32().unwrap_or(1.0),
        tags: record.tags.into_iter().flatten().collect(),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}
pub(crate) fn record_to_key_result(record: KeyResultRecord) -> KeyResult {
    KeyResult {
        id: record.id,
        objective_id: record.objective_id,
        owner_id: record.owner_id,
        title: record.title,
        description: record.description,
        metric_type: MetricType::from_label(&record.metric_type),
        start_value: record.start_value.to_f64().unwrap_or(0.0),
        target_value: record.target_value.to_f64().unwrap_or(0.0),
        current_value: record.current_value.to_f64().unwrap_or(0.0),
        unit: record.unit,
        weight: record.weight.to_f32().unwrap_or(1.0),
        due_date: record.due_date,
        status: KRStatus::from_label(&record.status),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}
pub(crate) fn record_to_checkin(record: CheckInRecord) -> CheckIn {
    CheckIn {
        id: record.id,
        key_result_id: record.key_result_id,
        user_id: record.user_id,
        previous_value: record.previous_value.and_then(|v| v.to_f64()).unwrap_or(0.0),
        new_value: record.new_value.to_f64().unwrap_or(0.0),
        note: record.note.unwrap_or_default(),
        confidence: Confidence::from_label(record.confidence.as_deref().unwrap_or("medium")),
        blockers: record.blockers,
        created_at: record.created_at,
    }
}

#[cfg(test)]
mod projection_tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn objective_record() -> ObjectiveRecord {
        ObjectiveRecord {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            owner_id: Uuid::nil(),
            parent_id: None,
            title: "Grow ARR".to_string(),
            description: Some("Reach 1M".to_string()),
            period: "q4-2026".to_string(),
            period_start: NaiveDate::from_ymd_opt(2026, 10, 1),
            period_end: NaiveDate::from_ymd_opt(2026, 12, 31),
            status: "active".to_string(),
            progress: BigDecimal::from(45),
            visibility: "team".to_string(),
            weight: BigDecimal::from(2),
            tags: vec![Some("growth".to_string()), None],
            created_at: now(),
            updated_at: now(),
            branch_id: Uuid::nil(),
        }
    }

    #[test]
    fn an_objective_maps_every_field() {
        let record = objective_record();
        let id = record.id;
        let objective = record_to_objective(record);

        assert_eq!(objective.id, id);
        assert_eq!(objective.title, "Grow ARR");
        assert_eq!(objective.description, "Reach 1M");
        assert_eq!(objective.period, "q4-2026");
        assert_eq!(objective.progress, 45.0);
        assert_eq!(objective.weight, 2.0);
        // Empty tag slots are dropped.
        assert_eq!(objective.tags, vec!["growth"]);
        assert!(objective.parent_id.is_none());
    }

    #[test]
    fn a_missing_description_becomes_an_empty_string() {
        let mut record = objective_record();
        record.description = None;
        assert_eq!(record_to_objective(record).description, "");
    }

    #[test]
    fn a_key_result_maps_its_metric_and_values() {
        let record = KeyResultRecord {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            objective_id: Uuid::nil(),
            owner_id: Uuid::nil(),
            title: "New customers".to_string(),
            description: None,
            metric_type: "number".to_string(),
            start_value: BigDecimal::from(0),
            target_value: BigDecimal::from(100),
            current_value: BigDecimal::from(25),
            unit: Some("count".to_string()),
            weight: BigDecimal::from(1),
            status: "on_track".to_string(),
            due_date: NaiveDate::from_ymd_opt(2026, 12, 31),
            scoring_type: "linear".to_string(),
            created_at: now(),
            updated_at: now(),
            branch_id: Uuid::nil(),
        };

        let kr = record_to_key_result(record);
        assert_eq!(kr.title, "New customers");
        assert_eq!(kr.start_value, 0.0);
        assert_eq!(kr.target_value, 100.0);
        assert_eq!(kr.current_value, 25.0);
        assert_eq!(kr.unit.as_deref(), Some("count"));
        assert_eq!(kr.weight, 1.0);
        assert!(kr.due_date.is_some());
    }

    #[test]
    fn a_checkin_defaults_missing_values() {
        let record = CheckInRecord {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            key_result_id: Uuid::nil(),
            user_id: Uuid::nil(),
            previous_value: None,
            new_value: BigDecimal::from(42),
            note: None,
            confidence: None,
            blockers: None,
            created_at: now(),
            branch_id: Uuid::nil(),
        };

        let checkin = record_to_checkin(record);
        assert_eq!(checkin.previous_value, 0.0);
        assert_eq!(checkin.new_value, 42.0);
        assert_eq!(checkin.note, "");
        // An absent confidence falls back to the documented default.
        assert_eq!(checkin.confidence, Confidence::Medium);
    }
}
