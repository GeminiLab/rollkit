use rollkit::{eval, parse};

fn main() {
    // Test basic dice roll
    let result = parse("3d6");
    println!("3d6: {}\n", result.unwrap().format_inline());

    // Test dice roll with keep highest
    let result = parse("4d6kh3");
    println!("4d6kh3: {}\n", result.unwrap().format_inline());
    // Test arithmetic
    let result = parse("2d6 + 5");
    println!("2d6 + 5: {}\n", result.unwrap().format_inline());

    // Test comparison
    let result = parse("3d6 >= 10");
    println!("3d6 >= 10: {}\n", result.unwrap().format_inline());
    // Test explicit list
    let result = parse("{1, 2, 3}");
    println!("{{1, 2, 3}}: {}\n", result.unwrap().format_inline());

    // Test range list
    let result = parse("[1, 5, 2]");
    println!("[1, 5, 2]: {}\n", result.unwrap().format_inline());
    // Test function call
    let result = parse("max(3d6, 10)");
    println!("max(3d6, 10): {}\n", result.unwrap().format_inline());

    // Test complex expression
    let result = parse("(3 + 2d6) * 4 + 5 * 1d2d3");
    println!(
        "(3 + 2d6) * 4 + 5 * 1d2d3: {}\n",
        result.unwrap().format_inline()
    );

    // Test overflow handling
    let result = parse("9999999999999999999999");
    match result {
        Ok(expr) => println!("9999999999999999999999: {}\n", expr.format_inline()),
        Err(errors) => {
            println!(
                "9999999999999999999999: Error - {}\n",
                errors.first().unwrap()
            );
        }
    }

    let result = parse("{1d2d6}");
    match &result {
        Ok(expr) => println!("{{1d2d6}}: {}\n", expr.format_inline()),
        Err(errors) => {
            println!("{{1d2d6}}: Error - {}\n", errors.first().unwrap());
        }
    }

    let result = eval(&parse("{2d6} + 1").unwrap());
    match result {
        Ok(value) => println!("Evaluated value: {:?}\n", value),
        Err(e) => println!("Evaluation error: {:?}\n", e),
    }
}
