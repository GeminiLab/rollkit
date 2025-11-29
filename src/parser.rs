use super::ast::{BinaryOperator, Expr, Literal};

use chumsky::{
    pratt::{Associativity, infix, left, right},
    prelude::*,
};

type ParserInput<'a> = &'a str;
type ParserError<'a> = extra::Err<Rich<'a, char>>;

/// Parse a RollKit expression from a string input.
/// 
/// # Examples
/// 
/// ```
/// # use rollkit::{parse, parsing::*};
/// assert_eq!(parse("2d6 + 3"), Ok(Expr::BinaryOp {
///     left: Box::new(Expr::BinaryOp {
///         left: Box::new(Expr::Literal(Literal::Int(2))),
///         op: BinaryOperator::DiceRoll,
///         right: Box::new(Expr::Literal(Literal::Int(6))),
///     }),
///     op: BinaryOperator::Addition,
///     right: Box::new(Expr::Literal(Literal::Int(3))),
/// }));
/// 
/// assert!(parse("4d{1,2,3}kh2").is_ok());
/// assert!(parse("[1, 10, 2] + 5").is_ok());
/// assert!(parse("max(3d6)").is_ok());
/// assert!(parse("1+").is_err());
/// ```
pub fn parse(input: &str) -> Result<Expr, Vec<Rich<'_, char>>> {
    parser().parse(input).into_result()
}

/// Main parser for the RollKit expression.
pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, Expr, ParserError<'a>> + Clone {
    expression_parser()
}

/// Creates a parser for integer literals with overflow handling.
fn integer_parser<'a>() -> impl Parser<'a, ParserInput<'a>, i64, ParserError<'a>> + Clone {
    just('-')
        .or_not()
        .then(text::int(10))
        .validate(|(neg, num): (Option<char>, &str), extra, emitter| {
            match if neg.is_some() {
                format!("-{}", num).parse::<i64>()
            } else {
                num.parse::<i64>()
            } {
                Ok(val) => val,
                Err(e) => {
                    emitter.emit(Rich::custom(
                        extra.span(),
                        format!("illegal integer literal: {}", e),
                    ));
                    0 // Return a default value
                }
            }
        })
        .padded()
        .labelled("integer")
}

/// Creates a parser for range list literals.
fn range_list_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Literal, ParserError<'a>> + Clone {
    let integer = integer_parser();

    // Parse range list literal: [start, end] or [start, end, step]
    integer
        .clone()
        .then_ignore(just(',').padded())
        .then(integer.clone())
        .then(just(',').padded().ignore_then(integer.clone()).or_not())
        .delimited_by(just('[').padded(), just(']').padded())
        .map(|((start, end), step)| Literal::Range { start, end, step })
        .labelled("range list")
}

/// Creates a parser for RollKit expressions.
fn expression_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Expr, ParserError<'a>> + Clone {
    recursive(|expr| {
        // Parse integer literals (positive and negative)
        let integer = integer_parser();

        // Parse range list literal: [start, end] or [start, end, step]
        let range_list = range_list_parser();

        // Function call: functionName(arg1, arg2, ...)
        let function_call = text::ascii::ident()
            .padded()
            .then(
                expr.clone()
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<Expr>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(|(name, args): (&str, Vec<Expr>)| Expr::FunctionCall {
                name: name.to_string(),
                args,
            })
            .labelled("function call");

        // Explicit list literal: {1, 2, 3} or {{...}} for strong lists
        let list = expr
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<Expr>>()
            .delimited_by(just('{').padded(), just('}').padded())
            .validate(|exprs, extra, emitter| {
                // Check if all expressions are integer literals to create a List
                let mut int_values = Vec::new();
                let mut all_ints = true;

                for e in &exprs {
                    if let Expr::Literal(Literal::Int(val)) = e {
                        int_values.push(*val);
                    } else {
                        all_ints = false;
                        break;
                    }
                }

                if all_ints && !exprs.is_empty() {
                    Expr::Literal(Literal::List(int_values))
                } else if exprs.len() == 1 {
                    // Single element could be making a strong list
                    Expr::StrongList(Box::new(exprs[0].clone()))
                } else {
                    emitter.emit(Rich::custom(
                        extra.span(),
                        "things inside braces must be all integers for a list, or a single expression for a strong list",
                    ));
                    Expr::Literal(Literal::List(vec![])) // Default empty list on error
                }
            });

        // Atom: integer, range list, explicit list, function call, or parenthesized expression
        let atom = choice((
            function_call,
            range_list.map(Expr::Literal),
            list,
            integer.clone().map(|i| Expr::Literal(Literal::Int(i))),
            expr.clone()
                .delimited_by(just('(').padded(), just(')').padded()),
        ))
        .padded();

        let binary_op_to_pratt = |op: BinaryOperator, accos: fn(u16) -> Associativity| {
            // let v = just(op.to_str()).padded();
            infix(accos(op.precedence()), just(op.to_str()).padded(), move |left: Expr, _, right: Expr, _| Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })
        };

        let expr = atom.clone().pratt((
            binary_op_to_pratt(BinaryOperator::DiceRoll, right),
            binary_op_to_pratt(BinaryOperator::KeepHighest, left),
            binary_op_to_pratt(BinaryOperator::KeepLowest, left),
            binary_op_to_pratt(BinaryOperator::DropHighest, left),
            binary_op_to_pratt(BinaryOperator::DropLowest, left),
            binary_op_to_pratt(BinaryOperator::Multiplication, left),
            binary_op_to_pratt(BinaryOperator::Addition, left),
            binary_op_to_pratt(BinaryOperator::Subtraction, left),
            binary_op_to_pratt(BinaryOperator::Equal, left),
            binary_op_to_pratt(BinaryOperator::NotEqual, left),
            binary_op_to_pratt(BinaryOperator::LessThan, left),
            binary_op_to_pratt(BinaryOperator::LessEqual, left),
            binary_op_to_pratt(BinaryOperator::GreaterThan, left),
            binary_op_to_pratt(BinaryOperator::GreaterEqual, left),
        )).padded();

        expr
    })
    .then_ignore(end())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_integer_parsing() {
        let cases = vec![
            ("42", Ok(42)),
            ("-42", Ok(-42)),
            ("9223372036854775807", Ok(9223372036854775807)),
            ("-9223372036854775808", Ok(-9223372036854775808)),
            ("9223372036854775808", Err(())),  // Overflow
            ("-9223372036854775809", Err(())), // Underflow
        ];

        for (input, expected) in cases {
            let result = integer_parser().parse(input).into_result();
            match (&result, expected) {
                (Ok(val), Ok(exp)) => assert_eq!(*val, exp, "Input: {}", input),
                (Err(_), Err(())) => {} // Expected error
                _ => panic!("Unexpected result for input {}: {:?}", input, result),
            }
        }
    }
}
