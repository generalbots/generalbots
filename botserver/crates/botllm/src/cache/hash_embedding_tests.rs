//! Offline tests for the deterministic fallback embedding and the cosine
//! similarity used when the embedding server is unreachable (issue #2075).
//! Both helpers are pure: SHA-256 over the input, no network, no clock.

use super::*;

fn service() -> LocalEmbeddingService {
    LocalEmbeddingService::new("http://127.0.0.1:8081".to_string(), "m".to_string(), None)
}

fn similarity(a: &[f32], b: &[f32]) -> f32 {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(service().compute_similarity(a, b))
}

#[test]
fn hash_embedding_has_the_published_dimension_and_is_unit_length() {
    let embedding = service().hash_embedding("general bots");

    assert_eq!(embedding.len(), 384);
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (norm - 1.0).abs() < 1e-4,
        "embedding must be normalised, got norm {norm}"
    );
    assert!(embedding.iter().all(|v| v.is_finite()));
}

#[test]
fn hash_embedding_is_deterministic_for_the_same_text() {
    let service = service();
    assert_eq!(
        service.hash_embedding("repeatable"),
        service.hash_embedding("repeatable")
    );
}

#[test]
fn different_texts_produce_different_embeddings() {
    let service = service();
    let a = service.hash_embedding("invoice 42");
    let b = service.hash_embedding("invoice 43");
    assert_ne!(a, b);
    assert!(similarity(&a, &b) < 0.999);
}

#[test]
fn empty_text_still_yields_a_normalised_vector() {
    let embedding = service().hash_embedding("");
    assert_eq!(embedding.len(), 384);
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4);
}

#[test]
fn cosine_similarity_of_identical_vectors_is_one() {
    let embedding = service().hash_embedding("same text");
    let score = similarity(&embedding, &embedding);
    assert!((score - 1.0).abs() < 1e-5, "score {score}");
}

#[test]
fn cosine_similarity_of_opposite_vectors_is_minus_one() {
    assert!((similarity(&[1.0, 0.0], &[-1.0, 0.0]) + 1.0).abs() < 1e-6);
}

#[test]
fn mismatched_dimensions_and_zero_vectors_score_zero() {
    assert_eq!(similarity(&[1.0, 0.0], &[1.0, 0.0, 0.0]), 0.0);
    assert_eq!(similarity(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
    assert_eq!(similarity(&[], &[]), 0.0);
}

#[test]
fn cosine_similarity_is_orthogonal_aware() {
    assert_eq!(similarity(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
}
