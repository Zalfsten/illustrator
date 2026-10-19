use super::*;

#[test]
fn double_index_vector() {
    let mut p = Vector::zeros(3);
    p[1] = 4.0;
    assert_eq!(p[1], 4.0);
}
