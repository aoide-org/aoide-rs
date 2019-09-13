use super::*;

#[test]
fn parse() {
    assert_eq!(ColorRgb::BLACK, "#000000".parse().unwrap());
    assert_eq!(ColorRgb::RED, "#FF0000".parse().unwrap());
    assert_eq!(ColorRgb::GREEN, "#00FF00".parse().unwrap());
    assert_eq!(ColorRgb::BLUE, "#0000FF".parse().unwrap());
}

#[test]
fn format() {
    assert_eq!("#000000", ColorRgb::BLACK.to_string());
    assert_eq!("#FF0000", ColorRgb::RED.to_string());
    assert_eq!("#00FF00", ColorRgb::GREEN.to_string());
    assert_eq!("#0000FF", ColorRgb::BLUE.to_string());
}
