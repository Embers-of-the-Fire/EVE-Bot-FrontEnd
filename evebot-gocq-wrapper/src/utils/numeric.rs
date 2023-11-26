pub fn format_price(price: f64) -> String {
    let price_trunc = price.trunc() as u64;
    let price_fract = (price.fract() * 100.0).round() as u8;
    let mut buffer = itoa::Buffer::new();
    let mut string = String::new();

    let mut calc = price_trunc;
    for (idx, divider) in (0..=6).rev().map(|s| 1000_u64.pow(s)).enumerate() {
        let f = calc.div_euclid(divider);
        calc = calc.rem_euclid(divider);
        if f > 0 {
            if string.is_empty() || idx == 0 {
                string += buffer.format(f);
            } else {
                string += &format!("{:0>3}", buffer.format(f));
            }
            if idx != 6 {
                string += ","
            }
        }
    }
    if string.is_empty() {
        string += "0";
    }
    string += ".";
    string += &format!("{:0>2}", buffer.format(price_fract));
    string
}

#[test]
fn test_format_price() {
    let res = format_price(12.50);
    assert_eq!(res, "12.50");
    let res = format_price(1250.50);
    assert_eq!(res, "1,250.50");
    let res = format_price(1250468.5095);
    assert_eq!(res, "1,250,468.51");
    let res = format_price(1250468.5295);
    assert_eq!(res, "1,250,468.53");
    let res = format_price(0.0);
    assert_eq!(res, "0.00");
    let res = format_price(0.006);
    assert_eq!(res, "0.01");
}
