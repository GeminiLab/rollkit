use rollkit::parse;

fn main() {
    println!("=== Testing integer overflow handling ===\n");

    // Test valid i64 max
    let result = parse("9223372036854775807");
    println!("i64::MAX (9223372036854775807): {:?}\n", result);

    // Test valid i64 min
    let result = parse("-9223372036854775808");
    println!("i64::MIN (-9223372036854775808): {:?}\n", result);

    // Test overflow - too large positive
    let result = parse("9223372036854775808");
    println!("Overflow (9223372036854775808): {:?}\n", result);

    // Test overflow - too large negative
    let result = parse("-9223372036854775809");
    println!("Overflow (-9223372036854775809): {:?}\n", result);

    // Test very large number
    let result = parse("99999999999999999999");
    println!("Very large (99999999999999999999): {:?}\n", result);

    // Test overflow in expression
    let result = parse("9999999999999999999999 + 5");
    println!(
        "Overflow in expression (9999999999999999999999 + 5): {:?}\n",
        result
    );

    // Test valid expression
    let result = parse("100 + 200");
    println!("Valid expression (100 + 200): {:?}\n", result);
}
