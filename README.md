# RollKit

RollKit is a dice-rolling library for defining and evaluating dice expressions from the simple to the complex, with a syntax similar to classical RPG dice notations, but modified and extended slightly in a backward-compatible way to make it more formal, powerful, and flexible.

RollKit supports a variety of dice rolling and picking mechanics, arithmetic operations, comparison operations, and function calls. It can be used as a library in Rust projects or via its command-line REPL tool. 

```plaintext
rollkit:[1]> 3d6
[1] 9 (from list with 3 elements: {4, 1, 4})
rollkit:[2]> 4d[1,100]kh2
[2] 139 (from list with 2 elements: {58, 81})
rollkit:[3]> :explain 5d{1,3,5,7,9}dl2
[3] 13 (from list with 3 elements: {3, 9, 1})
Explanation:
  Parsed: ((5 d {1, 3, 5, 7, 9}) dl 2)
  Expression Structure:
    Binary Operation: dl (Drop Lowest)
      Binary Operation: d (Dice Roll)
        Literal: 5 (Integer)
        Literal: {1, 3, 5, 7, 9} (List with 5 elements)
      Literal: 2 (Integer)
```

## Syntax

RollKit expressions are math-like expressions composed of values and operators described below. Programmers will find the syntax similar to expressions in many programming languages, with some additions for dice rolling.

### Values

RollKit supports two primary types of values: 

- **Integers**: Positive and negative integers we all know and love, e.g., `1`, `42`, `-7`, `3 + 4`.
- **Lists**: Ordered lists of 0 or more integers, e.g., `{1, 2, 3}`, `{7, 6, 5, 4}`, `3d6`, `{}`.

Lists can be further divided into two subtypes: **Strong Lists** and **Normal Lists**. The only difference is that when performing arithmetic and comparison operations, Normal Lists will be automatically reduced to their sum, while Strong Lists will not and the operation will be performed element-wise. Strong Lists are created by wrapping a Normal List with braces, e.g., `{{1, 2, 3}}` or `{3d6}`.

```plaintext
rollkit:[1]> {1,2,3} + 5
[1] 11
rollkit:[2]> {{1,2,3}} + 5
[2] 21 (from list with 3 elements: {6, 7, 8})
```

### Literals

RollKit supports three types of literals for creating values:

- **Integer Literal**: Just a plain integer, e.g., `42`, `-7`.
- **Explicit List Literal**: A list defined by curly braces, e.g., `{1, 2, 3}`.
- **Range List Literal**: A list defined by a range and an optional step, e.g., `[1, 3]` for `{1, 2, 3}`, `[5, 15, 5]` for `{5, 10, 15}`. Note that:
    - The start and end values are inclusive.
    - The start value can be greater than the end value for descending ranges, e.g., `[10, 5]` for `{10, 9, 8, 7, 6, 5}`.
    - The sign of the step value is ignored, a step of `0` is invalid, e.g., both `[1, 10, 2]` and `[1, 10, -2]` produce `{1, 3, 5, 7, 9}`.

### Operators

RollKit supports a variety of operators, and here is the complete list, with their precedence (from highest to lowest):

- Dice Operators:
    - Dice Roll: `d: Integer x Integer | List -> List`, right associative, e.g., `3d6` rolls three six-sided dice, `2d{1,2,3}` rolls two dice with faces 1, 2, and 3.
    - Keep/Drop Highest/Lowest: `kh/kl/dh/dl: List x Integer -> List`, left associative, e.g., `4d6kh3` rolls four six-sided dice and keeps the highest three.
- Arithmetic Operators:
    - Multiplication: `*: Integer x Integer -> Integer`, left associative, e.g., `3 * 4` results in `12`.
    - Addition and Subtraction: `+ / -: Integer x Integer -> Integer`, left associative, e.g., `5 + 2` results in `7`, `10 - 3` results in `7`.
- Comparison Operators:
    - Comparison: `== / != / < / <= / > / >=: Integer x Integer -> Integer`, left associative, e.g., `5 > 3` results in `1` (true), `2 == 4` results in `0` (false).

Parentheses `()` can be used to group expressions and override the default precedence, as in other programming languages.

Function calls are also supported, with the syntax: `functionName(arg1, arg2, ...)`, where `functionName` is the name of the function and `arg1`, `arg2`, etc. are the arguments passed to the function.

## Library Usage

To use RollKit in Rust, use `parse` and `eval` to parse and evaluate expressions:

```rust
use rollkit::{parse, eval};

let expr = parse("4d6kh3 + 2").unwrap();
let result = eval(&expr);
println!("Result: {:?}", result);
```

You can use `eval_with` to specify a custom RNG:

```rust
use rand::{SeedableRng, rngs::StdRng};
use rollkit::{parse, eval_with};

let mut rng = StdRng::from_os_rng();
let expr = parse("4d6kh3 + 2").unwrap();
let result = eval_with(&expr, &mut rng).unwrap(); 
println!("Result: {:?}", result);
```

## REPL

RollKit comes with a command-line REPL tool for interactive dice rolling. To start the REPL, run:

```bash
cargo run -p rollkit_repl
```

You can enter RollKit expressions directly, and use commands `:explain <expr>` to see the parsed structure of an expression. Use `:help` to see all available commands.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
