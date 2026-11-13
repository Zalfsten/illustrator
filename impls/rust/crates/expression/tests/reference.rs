//! Values against the GNU Octave reference table (tests/reference/generate.m).

use expression::Expression;

const TABLE: &str = include_str!("reference/reference.tsv");

/// About four ulps relative, with an absolute floor near zero.
fn close(got: f64, want: f64) -> bool {
    (got - want).abs() <= (want.abs() * 2f64.powi(-50)).max(1e-300)
}

#[test]
fn values_match_octave() {
    let mut checked = 0;
    for line in TABLE.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let [source, x, value] = cols[..] else {
            panic!("malformed row {line:?}");
        };
        let x: f64 = x.parse().unwrap();
        let e = Expression::parse(source, "x").unwrap_or_else(|err| panic!("{source}: {err}"));
        let result = e.eval(x);
        if value == "domain-error" {
            assert!(result.is_err(), "{source} at {x}: expected a domain error, got {result:?}");
        } else {
            let want: f64 = value.parse().unwrap();
            let got = result.unwrap_or_else(|err| panic!("{source} at {x}: {err}")).value;
            assert!(close(got, want), "{source} at {x}: {got:e} vs Octave {want:e}");
        }
        checked += 1;
    }
    assert_eq!(checked, 67);
}
