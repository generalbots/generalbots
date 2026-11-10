use super::*;

pub fn to_array(dynamic: &Dynamic) -> Vec<Dynamic> {
    if let Some(array) = dynamic.clone().try_cast::<Vec<Dynamic>>() {
        array
    } else if let Some(map) = dynamic.clone().try_cast::<rhai::Map>() {
        map.into_values().collect()
    } else {
        vec![dynamic.clone()]
    }
}
pub fn json_value_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::from(n.to_string())
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => {
            let vec: Vec<Dynamic> = arr.iter().map(json_value_to_dynamic).collect();
            Dynamic::from(vec)
        }
        Value::Object(map) => {
            let mut rhai_map = rhai::Map::new();
            for (k, v) in map {
                rhai_map.insert(k.clone().into(), json_value_to_dynamic(v));
            }
            Dynamic::from(rhai_map)
        }
    }
}
pub fn dynamic_to_json(value: &Dynamic) -> Value {
    if value.is_unit() {
        Value::Null
    } else if let Ok(b) = value.as_bool() {
        Value::Bool(b)
    } else if let Ok(i) = value.as_int() {
        Value::from(i)
    } else if let Ok(f) = value.as_float() {
        Value::from(f)
    } else if let Some(s) = value.clone().try_cast::<String>() {
        Value::String(s)
    } else if let Some(arr) = value.clone().try_cast::<Vec<Dynamic>>() {
        Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if let Some(map) = value.clone().try_cast::<rhai::Map>() {
        let mut json_map = serde_json::Map::new();
        for (k, v) in map {
            json_map.insert(k.to_string(), dynamic_to_json(&v));
        }
        Value::Object(json_map)
    } else {
        Value::String(value.to_string())
    }
}
pub fn convert_date_to_iso_format(date_str: &str) -> String {
    let trimmed = date_str.trim();
    // Already ISO format (yyyy-MM-dd or yyyy-MM-dd HH:mm:ss)
    if trimmed.len() >= 10 && trimmed.as_bytes()[4] == b'-' && trimmed.as_bytes()[7] == b'-' {
        return trimmed.to_string();
    }
    // Brazilian format: dd/mm/aaaa or dd/mm/aaaa HH:mm:ss
    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.splitn(3, '/').collect();
        if parts.len() == 3 {
            let day = parts[0];
            let month = parts[1];
            let rest = parts[2];
            let year = if rest.contains(' ') {
                rest.split(' ').next().unwrap_or(rest)
            } else {
                rest
            };
            // Validate numbers
            if day.chars().all(|c| c.is_ascii_digit()) && month.chars().all(|c| c.is_ascii_digit()) {
                let d: i32 = day.parse().unwrap_or(0);
                let m: i32 = month.parse().unwrap_or(0);
                if (1..=31).contains(&d) && (1..=12).contains(&m) {
                    let time_part = if rest.contains(' ') {
                        " ".to_string() + rest.split(' ').nth(1).unwrap_or("")
                    } else {
                        String::new()
                    };
                    return format!("{:04}-{:02}-{:02}{}", year, m, d, time_part);
                }
            }
        }
    }
    date_str.to_string()
}
pub fn get_work_path() -> String {
    std::env::var("GBO_WORK_PATH").unwrap_or_else(|_| "/opt/gbo/work".to_string())
}
/// Returns the current org ID for bot file path isolation.
/// Always returns Uuid::nil() until real multi-tenant auth is implemented.
pub fn current_org_id() -> uuid::Uuid {
    uuid::Uuid::nil()
}
/// Build a relative bot path with org isolation (.gborg wrapping).
/// Returns: "{org_id}.gborg/{bot_bucket}.gbai/{sub_path}"
pub fn build_bot_path(org_id: impl std::fmt::Display, bot_bucket: &str, sub_path: &str) -> String {
    format!("{org_id}.gborg/{bot_bucket}.gbai/{sub_path}")
}

#[cfg(test)]
mod to_array_tests {
    use super::*;

    #[test]
    fn an_array_is_returned_as_is() {
        let value = Dynamic::from(vec![Dynamic::from(1_i64), Dynamic::from(2_i64)]);
        assert_eq!(to_array(&value).len(), 2);
    }

    #[test]
    fn a_map_yields_its_values() {
        let mut map = rhai::Map::new();
        map.insert("a".into(), Dynamic::from(1_i64));
        map.insert("b".into(), Dynamic::from(2_i64));
        assert_eq!(to_array(&Dynamic::from(map)).len(), 2);
    }

    #[test]
    fn a_scalar_is_wrapped_in_a_single_element_array() {
        let wrapped = to_array(&Dynamic::from(7_i64));
        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0].as_int().unwrap(), 7);
        assert_eq!(to_array(&Dynamic::from("x")).len(), 1);
    }

    #[test]
    fn json_scalars_map_to_the_matching_rhai_type() {
        assert!(json_value_to_dynamic(&Value::Null).is_unit());
        assert_eq!(json_value_to_dynamic(&Value::Bool(true)).as_bool().unwrap(), true);
        assert_eq!(json_value_to_dynamic(&serde_json::json!(42)).as_int().unwrap(), 42);
        assert_eq!(json_value_to_dynamic(&serde_json::json!(1.5)).as_float().unwrap(), 1.5);
        assert_eq!(
            json_value_to_dynamic(&serde_json::json!("hi")).into_string().unwrap(),
            "hi"
        );
    }

    #[test]
    fn json_collections_round_trip_through_rhai() {
        let original = serde_json::json!({"a": [1, 2], "b": {"c": true}, "d": null});
        let dynamic = json_value_to_dynamic(&original);
        assert_eq!(dynamic_to_json(&dynamic), original);
    }

    #[test]
    fn dynamic_to_json_covers_scalars_and_containers() {
        assert_eq!(dynamic_to_json(&Dynamic::UNIT), Value::Null);
        assert_eq!(dynamic_to_json(&Dynamic::from(true)), Value::Bool(true));
        assert_eq!(dynamic_to_json(&Dynamic::from(3_i64)), serde_json::json!(3));
        assert_eq!(dynamic_to_json(&Dynamic::from(3.5_f64)), serde_json::json!(3.5));
        assert_eq!(dynamic_to_json(&Dynamic::from("s")), serde_json::json!("s"));
        assert_eq!(
            dynamic_to_json(&Dynamic::from(vec![Dynamic::from(1_i64)])),
            serde_json::json!([1])
        );
    }

    #[test]
    fn iso_dates_pass_through_unchanged() {
        assert_eq!(convert_date_to_iso_format("2026-11-10"), "2026-11-10");
        assert_eq!(
            convert_date_to_iso_format("2026-11-10 14:30:00"),
            "2026-11-10 14:30:00"
        );
        assert_eq!(convert_date_to_iso_format("  2026-11-10  "), "2026-11-10");
    }

    #[test]
    fn brazilian_dates_convert_to_iso() {
        assert_eq!(convert_date_to_iso_format("10/11/2026"), "2026-11-10");
        assert_eq!(
            convert_date_to_iso_format("10/11/2026 14:30:00"),
            "2026-11-10 14:30:00"
        );
        assert_eq!(convert_date_to_iso_format("01/02/2026"), "2026-02-01");
    }

    #[test]
    fn invalid_or_unrecognised_dates_are_returned_verbatim() {
        assert_eq!(convert_date_to_iso_format("not a date"), "not a date");
        assert_eq!(convert_date_to_iso_format("32/01/2026"), "32/01/2026");
        assert_eq!(convert_date_to_iso_format("10/13/2026"), "10/13/2026");
        assert_eq!(convert_date_to_iso_format("10/ab/2026"), "10/ab/2026");
        assert_eq!(convert_date_to_iso_format(""), "");
    }

    #[test]
    fn bot_paths_wrap_the_bucket_in_the_org() {
        assert_eq!(
            build_bot_path(uuid::Uuid::nil(), "acme", "acme.gbdialog/x.bas"),
            "00000000-0000-0000-0000-000000000000.gborg/acme.gbai/acme.gbdialog/x.bas"
        );
    }
}
