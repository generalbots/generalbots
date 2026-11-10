//! #2069 — the collaborative-editing transform. Two concurrent operations must
//! be rebased so that neither loses the other's edit; the branches below are
//! the whole contract.
use super::*;

fn op(kind: TransformOpType, path: &[usize]) -> TransformOperation {
    TransformOperation {
        op_type: kind,
        path: path.to_vec(),
        value: None,
        old_value: None,
    }
}

#[test]
fn an_empty_path_is_never_rebased() {
    let a = op(TransformOpType::Insert, &[]);
    let b = op(TransformOpType::Insert, &[]);
    let (a2, b2) = transform_operations(&a, &b);
    assert_eq!(a2.path, Vec::<usize>::new());
    assert_eq!(b2.path, Vec::<usize>::new());
}

#[test]
fn operations_in_unrelated_containers_are_untouched() {
    let a = op(TransformOpType::Insert, &[0, 1]);
    let b = op(TransformOpType::Insert, &[1, 1]);
    let (a2, b2) = transform_operations(&a, &b);
    assert_eq!(a2.path, vec![0, 1]);
    assert_eq!(b2.path, vec![1, 1]);
}

#[test]
fn identical_inserts_shift_the_second_one_after_the_first() {
    // #2069 — both paths were left at [0], so the second insert landed BEFORE
    // the first instead of after it.
    let a = op(TransformOpType::Insert, &[0]);
    let b = op(TransformOpType::Insert, &[0]);
    let (a2, b2) = transform_operations(&a, &b);

    assert_eq!(a2.path, vec![0]);
    assert_eq!(b2.path, vec![1]);

    let a = op(TransformOpType::Insert, &[2, 3]);
    let b = op(TransformOpType::Insert, &[2, 3]);
    let (a2, b2) = transform_operations(&a, &b);
    assert_eq!(a2.path, vec![2, 3]);
    assert_eq!(b2.path, vec![2, 4]);
}

#[test]
fn a_later_insert_in_the_same_container_shifts() {
    let a = op(TransformOpType::Insert, &[0, 1]);
    let b = op(TransformOpType::Insert, &[0, 2]);
    let (a2, b2) = transform_operations(&a, &b);

    assert_eq!(a2.path, vec![0, 1]);
    assert_eq!(b2.path, vec![0, 3]);
}

#[test]
fn an_earlier_insert_shifts_the_other_side_too() {
    let a = op(TransformOpType::Insert, &[0, 5]);
    let b = op(TransformOpType::Insert, &[0, 2]);
    let (a2, b2) = transform_operations(&a, &b);

    assert_eq!(a2.path, vec![0, 6]);
    assert_eq!(b2.path, vec![0, 2]);
}

#[test]
fn an_insert_before_a_delete_does_not_move_the_deleted_index_backwards() {
    let delete = op(TransformOpType::Delete, &[0, 1]);
    let insert = op(TransformOpType::Insert, &[0, 3]);
    let (d2, i2) = transform_operations(&delete, &insert);

    assert_eq!(d2.path, vec![0, 1]);
    assert_eq!(i2.path, vec![0, 2]);
}

#[test]
fn a_delete_before_an_insert_pulls_the_insert_back() {
    let insert = op(TransformOpType::Insert, &[0, 3]);
    let delete = op(TransformOpType::Delete, &[0, 1]);
    let (i2, d2) = transform_operations(&insert, &delete);

    assert_eq!(i2.path, vec![0, 2]);
    assert_eq!(d2.path, vec![0, 1]);
}

#[test]
fn a_delete_at_the_head_saturates_instead_of_underflowing() {
    let delete = op(TransformOpType::Delete, &[0, 0]);
    let insert = op(TransformOpType::Insert, &[0, 1]);
    let (_, i2) = transform_operations(&delete, &insert);
    assert_eq!(i2.path, vec![0, 0]);
}

#[test]
fn deleting_the_same_block_twice_turns_the_second_into_a_no_op_replace() {
    let a = op(TransformOpType::Delete, &[0, 4]);
    let mut b = op(TransformOpType::Delete, &[0, 4]);
    b.value = Some(serde_json::json!("stale"));

    let (a2, b2) = transform_operations(&a, &b);

    assert_eq!(a2.op_type, TransformOpType::Delete);
    assert_eq!(b2.op_type, TransformOpType::Replace);
    assert_eq!(b2.value, None);
    assert_eq!(b2.old_value, None);
}

#[test]
fn different_deletes_are_untouched() {
    let a = op(TransformOpType::Delete, &[0, 1]);
    let b = op(TransformOpType::Delete, &[0, 2]);
    let (a2, b2) = transform_operations(&a, &b);
    assert_eq!(a2.op_type, TransformOpType::Delete);
    assert_eq!(b2.op_type, TransformOpType::Delete);
    assert_eq!(b2.path, vec![0, 2]);
}

#[test]
fn replace_and_move_pairs_are_passed_through() {
    let replace = op(TransformOpType::Replace, &[0, 0]);
    let mov = op(TransformOpType::Move, &[0, 1]);
    let (r2, m2) = transform_operations(&replace, &mov);
    assert_eq!(r2.op_type, TransformOpType::Replace);
    assert_eq!(r2.path, vec![0, 0]);
    assert_eq!(m2.op_type, TransformOpType::Move);
    assert_eq!(m2.path, vec![0, 1]);
}
