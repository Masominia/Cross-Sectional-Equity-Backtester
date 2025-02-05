use backlab::data::csv::parse;
use backlab::Panel;

#[test]
fn returns_and_accessors() {
    let p = Panel::new(vec!["A".into(), "B".into()], vec![100.0, 50.0, 110.0, 45.0, 99.0, 54.0]).unwrap();
    assert_eq!((p.days, p.assets), (3, 2));
    assert_eq!(p.price(2, 1), 54.0);
    assert_eq!(p.returns_on(0), &[0.0, 0.0]);
    assert!((p.returns_on(1)[0] - 0.10).abs() < 1e-15 && (p.returns_on(2)[1] - 0.20).abs() < 1e-15);
    let ew = p.equal_weight_returns();
    assert!(ew[1].abs() < 1e-15 && (ew[2] - 0.05).abs() < 1e-15);
    let one = p.truncated(1);
    assert_eq!((one.assets, one.days, one.price(1, 0)), (1, 3, 110.0));
}

#[test]
fn invalid_tables_are_rejected() {
    assert!(Panel::new(vec![], vec![1.0]).is_err());
    assert!(Panel::new(vec!["A".into(), "B".into()], vec![1.0, 2.0, 3.0]).is_err());
    assert!(Panel::new(vec!["A".into()], vec![1.0, 0.0]).is_err());
    assert!(Panel::new(vec!["A".into()], vec![1.0, f64::NAN]).is_err());
}

#[test]
fn csv_parsing() {
    let p = parse("day,AAA,BBB\n0,10,20\n1,11,19\r\n\n2,12.5,18\n").unwrap();
    assert_eq!((p.days, p.assets), (3, 2));
    assert_eq!(p.names, vec!["AAA", "BBB"]);
    assert_eq!(p.price(2, 0), 12.5);
    assert!(parse("").is_err());
    assert!(parse("day\n0\n").is_err());
    assert!(parse("day,A,B\n0,1\n").unwrap_err().contains("row 1"));
    assert!(parse("day,A\n0,abc\n").unwrap_err().contains("abc"));
    assert!(parse("day,A\n0,-1\n").is_err());
}
