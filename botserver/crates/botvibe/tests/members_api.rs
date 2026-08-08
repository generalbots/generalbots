use botvibe::members_api::{MembersResponse, RoleRequest, TransferRequest};
use serde_json::json;
use uuid::Uuid;

#[test]
fn members_response_serializes_ok_shape() {
    let response = MembersResponse {
        success: true,
        members: None,
        error: None,
    };
    let value = serde_json::to_value(&response).unwrap();
    assert_eq!(value["success"], true);
    assert!(value["members"].is_null());
    assert!(value["error"].is_null());
}

#[test]
fn members_response_serializes_error_shape() {
    let response = MembersResponse {
        success: false,
        members: None,
        error: Some("denied".to_string()),
    };
    let value = serde_json::to_value(&response).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["error"], "denied");
}

#[test]
fn role_request_deserializes_role_field() {
    let request: RoleRequest = serde_json::from_value(json!({"role": "admin"})).unwrap();
    assert_eq!(request.role, "admin");
}

#[test]
fn role_request_rejects_missing_role() {
    let result: Result<RoleRequest, _> = serde_json::from_value(json!({}));
    assert!(result.is_err());
}

#[test]
fn transfer_request_deserializes_user_id() {
    let user_id = Uuid::new_v4();
    let request: TransferRequest =
        serde_json::from_value(json!({"user_id": user_id})).unwrap();
    assert_eq!(request.user_id, user_id);
}
