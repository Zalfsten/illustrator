use super::*;

#[test]
fn gather_reads_mapped_entries() {
    let p = vec![10.0, 20.0, 30.0, 40.0];
    let c = Connectivity::new(vec![1, 3]);
    assert_eq!(c.gather(&p), vec![20.0, 40.0]);
}
