mod tests {
    use super::*;

    fn create_test_manager() -> JwtManager {
        JwtManager::from_secret("this-is-a-very-long-secret-key-for-testing-purposes-only")
            .expect("Failed to create manager")
    }

    #[test]
    fn test_generate_token_pair() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
    }

    #[test]
    fn test_validate_access_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");
        let claims = manager
            .validate_access_token(&pair.access_token)
            .expect("Validation failed");

        assert_eq!(claims.user_id().expect("Invalid user ID"), user_id);
        assert!(claims.is_access_token());
    }

    #[test]
    fn test_validate_refresh_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");
        let claims = manager
            .validate_refresh_token(&pair.refresh_token)
            .expect("Validation failed");

        assert_eq!(claims.user_id().expect("Invalid user ID"), user_id);
        assert!(claims.is_refresh_token());
    }

    #[test]
    fn test_token_with_claims() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager
            .generate_token_pair_with_claims(
                user_id,
                Some("test@example.com".into()),
                Some("testuser".into()),
                Some(vec!["admin".into(), "user".into()]),
                Some("session-123".into()),
            )
            .expect("Failed to generate");

        let claims = manager
            .validate_access_token(&pair.access_token)
            .expect("Validation failed");

        assert_eq!(claims.email, Some("test@example.com".into()));
        assert_eq!(claims.username, Some("testuser".into()));
        assert_eq!(claims.roles, Some(vec!["admin".into(), "user".into()]));
        assert_eq!(claims.session_id, Some("session-123".into()));
    }

    #[test]
    fn test_invalid_token() {
        let manager = create_test_manager();
        let result = manager.validate_token("invalid.token.here");

        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_token_type() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");

        let result = manager.validate_refresh_token(&pair.access_token);
        assert!(result.is_err());

        let result = manager.validate_access_token(&pair.refresh_token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");

        let token_data = manager
            .validate_token(&pair.access_token)
            .expect("Validation failed");

        manager
            .revoke_token(&token_data.claims.jti)
            .await
            .expect("Revoke failed");

        let result = manager
            .validate_token_with_blacklist(&pair.access_token)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_blacklist_removes_expired() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");
        let token_data = manager
            .validate_token(&pair.access_token)
            .expect("Validation failed");

        manager
            .revoke_token_with_expiry(&token_data.claims.jti, Utc::now().timestamp() - 3600)
            .await
            .expect("Revoke failed");

        assert!(manager.is_revoked(&token_data.claims.jti).await);

        let removed = manager.cleanup_blacklist(Utc::now()).await;
        assert_eq!(removed, 1);
        assert!(!manager.is_revoked(&token_data.claims.jti).await);
        assert_eq!(manager.blacklist_size().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_blacklist_keeps_recent() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");
        let token_data = manager
            .validate_token(&pair.access_token)
            .expect("Validation failed");

        // Revoked now, expires in the future → must survive cleanup.
        manager
            .revoke_token_with_expiry(&token_data.claims.jti, Utc::now().timestamp() + 3600)
            .await
            .expect("Revoke failed");

        let removed = manager.cleanup_blacklist(Utc::now()).await;
        assert_eq!(removed, 0);
        assert!(manager.is_revoked(&token_data.claims.jti).await);
        assert_eq!(manager.blacklist_size().await, 1);
    }

    #[tokio::test]
    async fn test_refresh_tokens() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager
            .generate_token_pair_with_claims(
                user_id,
                Some("test@example.com".into()),
                Some("testuser".into()),
                None,
                None,
            )
            .expect("Failed to generate");

        let new_pair = manager
            .refresh_tokens(&pair.refresh_token)
            .await
            .expect("Refresh failed");

        assert_ne!(new_pair.access_token, pair.access_token);
        assert_ne!(new_pair.refresh_token, pair.refresh_token);

        let claims = manager
            .validate_access_token(&new_pair.access_token)
            .expect("Validation failed");
        assert_eq!(claims.email, Some("test@example.com".into()));
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(
            extract_bearer_token("bearer abc123"),
            Some("abc123")
        );
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn test_claims_builder() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            "issuer",
            "audience",
            TokenType::Access,
            Utc::now() + Duration::hours(1),
        )
        .with_email("test@example.com".into())
        .with_username("testuser".into())
        .with_roles(vec!["admin".into()])
        .with_organization_id("org-123".into());

        assert_eq!(claims.email, Some("test@example.com".into()));
        assert_eq!(claims.username, Some("testuser".into()));
        assert_eq!(claims.roles, Some(vec!["admin".into()]));
        assert_eq!(claims.organization_id, Some("org-123".into()));
    }

    #[test]
    fn test_token_type() {
        assert_eq!(TokenType::Access.as_str(), "access");
        assert_eq!(TokenType::Refresh.as_str(), "refresh");
        assert_eq!(TokenType::IdToken.as_str(), "id_token");
    }

    #[test]
    fn test_jwt_algorithm() {
        assert!(JwtAlgorithm::HS256.is_symmetric());
        assert!(JwtAlgorithm::HS384.is_symmetric());
        assert!(JwtAlgorithm::HS512.is_symmetric());
        assert!(!JwtAlgorithm::RS256.is_symmetric());
        assert!(!JwtAlgorithm::ES256.is_symmetric());
    }

    #[test]
    fn test_token_introspection_response() {
        let inactive = TokenIntrospectionResponse::inactive();
        assert!(!inactive.active);

        let user_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            "issuer",
            "audience",
            TokenType::Access,
            Utc::now() + Duration::hours(1),
        );

        let active = TokenIntrospectionResponse::from_claims(&claims, true);
        assert!(active.active);
        assert_eq!(active.sub, Some(user_id.to_string()));
    }

    #[test]
    fn test_jwk_set() {
        let mut jwk_set = JwkSet::new();
        assert!(jwk_set.keys.is_empty());

        jwk_set.add_key(Jwk {
            kty: "RSA".into(),
            use_: Some("sig".into()),
            kid: Some("key-1".into()),
            alg: Some("RS256".into()),
            n: None,
            e: None,
            x: None,
            y: None,
            crv: None,
        });

        assert_eq!(jwk_set.keys.len(), 1);
    }
}
