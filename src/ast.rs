use core::{fmt, iter};

/// A range literal in the RollKit expression AST.
///
/// It's defined by an inclusive start point, an inclusive end point, and an optional step value.
/// If the step is not provided, it defaults to 1.
///
/// The range can be ascending (start <= end) or descending (start > end). The sign of the step is
/// automatically adjusted based on the direction of the range.
///
/// # Example
///
/// ```
/// # use rollkit::parsing::RangeLiteral;
/// assert_eq!(RangeLiteral { start: 1, end: 5, step: Some(2) }.to_vec(), vec![1, 3, 5]);
/// assert_eq!(RangeLiteral { start: 5, end: 1, step: Some(2) }.to_vec(), vec![5, 3, 1]);
/// assert_eq!(RangeLiteral { start: 1, end: 5, step: None }.to_vec(), vec![1, 2, 3, 4, 5]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeLiteral {
    /// The start of the range, inclusive.
    pub start: i64,
    /// The end of the range, inclusive.
    pub end: i64,
    /// The step of the range. If `None`, defaults to 1.
    pub step: Option<i64>,
}

impl RangeLiteral {
    /// Returns an iterator over the range.
    ///
    /// # Example
    ///
    /// ```
    /// # use rollkit::parsing::RangeLiteral;
    /// let range = RangeLiteral { start: 1, end: 5, step: Some(2) };
    /// let collected: Vec<i64> = range.to_iter().collect();
    /// assert_eq!(collected, vec![1, 3, 5]);
    /// ```
    pub fn to_iter(&self) -> impl Iterator<Item = i64> {
        let inc = self.end >= self.start;
        let step = self.step.map(i64::wrapping_abs).unwrap_or(1);
        let step = if inc { step } else { step.wrapping_neg() };

        iter::successors(Some(self.start), move |&cur| {
            let next = cur.wrapping_add(step);
            if (inc && (next > self.end || next < cur)) || (!inc && (next < self.end || next > cur))
            {
                None
            } else {
                Some(next)
            }
        })
    }

    /// Collects the range into a vector.
    ///
    /// # Example
    ///
    /// ```
    /// # use rollkit::parsing::RangeLiteral;
    /// let range = RangeLiteral { start: 1, end: 5, step: Some(2) };
    /// assert_eq!(range.to_vec(), vec![1, 3, 5]);
    /// ```
    pub fn to_vec(&self) -> Vec<i64> {
        self.to_iter().collect()
    }
}

/// A literal value in the RollKit expression AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    /// An integer literal.
    Int(i64),
    /// An explicit list literal e.g., `{1, 2, 3}`.
    List(Vec<i64>),
    /// A range list literal e.g., `[1, 5, 2]`.
    Range(RangeLiteral),
}

/// A binary operator in the RollKit expression AST.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    // Dice operators
    /// The dice roll operator `d`.
    DiceRoll,
    /// The keep highest operator `kh`.
    KeepHighest,
    /// The keep lowest operator `kl`.
    KeepLowest,
    /// The drop highest operator `dh`.
    DropHighest,
    /// The drop lowest operator `dl`.
    DropLowest,

    // Arithmetic operators
    /// The multiplication operator `*`.
    Multiplication,
    /// The addition operator `+`.
    Addition,
    /// The subtraction operator `-`.
    Subtraction,

    // Comparison operators
    /// The equal operator `==`.
    Equal,
    /// The not equal operator `!=`.
    NotEqual,
    /// The less than operator `<`.
    LessThan,
    /// The less than or equal operator `<=`.
    LessEqual,
    /// The greater than operator `>`.
    GreaterThan,
    /// The greater than or equal operator `>=`.
    GreaterEqual,
}

impl BinaryOperator {
    /// Returns the precedence of the operator.
    pub fn precedence(&self) -> u16 {
        match self {
            BinaryOperator::DiceRoll => 150,
            BinaryOperator::KeepHighest
            | BinaryOperator::KeepLowest
            | BinaryOperator::DropHighest
            | BinaryOperator::DropLowest => 130,
            BinaryOperator::Multiplication => 90,
            BinaryOperator::Addition | BinaryOperator::Subtraction => 70,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqual => 50,
        }
    }

    /// Returns the string representation of the operator.
    pub fn to_str(&self) -> &'static str {
        match self {
            BinaryOperator::DiceRoll => "d",
            BinaryOperator::KeepHighest => "kh",
            BinaryOperator::KeepLowest => "kl",
            BinaryOperator::DropHighest => "dh",
            BinaryOperator::DropLowest => "dl",
            BinaryOperator::Multiplication => "*",
            BinaryOperator::Addition => "+",
            BinaryOperator::Subtraction => "-",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::GreaterEqual => ">=",
        }
    }

    /// Returns a description of the operator.
    pub fn desc(&self) -> &'static str {
        match self {
            BinaryOperator::DiceRoll => "Dice Roll",
            BinaryOperator::KeepHighest => "Keep Highest",
            BinaryOperator::KeepLowest => "Keep Lowest",
            BinaryOperator::DropHighest => "Drop Highest",
            BinaryOperator::DropLowest => "Drop Lowest",
            BinaryOperator::Multiplication => "Multiplication",
            BinaryOperator::Addition => "Addition",
            BinaryOperator::Subtraction => "Subtraction",
            BinaryOperator::Equal => "Equal",
            BinaryOperator::NotEqual => "Not Equal",
            BinaryOperator::LessThan => "Less Than",
            BinaryOperator::LessEqual => "Less or Equal",
            BinaryOperator::GreaterThan => "Greater Than",
            BinaryOperator::GreaterEqual => "Greater or Equal",
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// A RollKit expression.
///
/// It's an AST node representing one of the possible expressions types in RollKit: literals,
/// binary operations, function calls, and strong lists.
///
/// # Creation
///
/// [`Expr`]s are typically created by [parsing](crate::parse) RollKit expressions from strings.
/// They can (naturally) also be constructed manually.
///
/// # Usage
///
/// Expressions can be evaluated using the evaluation functions ([`eval`](crate::eval) and
/// [`eval_with`](crate::eval_with)) or traversed using the [visitor pattern](ExprVisitor).
///
/// # Example
///
/// ```
/// # use rollkit::{parsing::Expr, parse, eval};
/// let expr: Expr = parse("2d6 + 3").unwrap();
/// let result = eval(&expr).unwrap();
/// assert!(matches!(result, rollkit::Value::Integer(n) if n >= 5 && n <= 15));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// The expression is a literal value.
    Literal(Literal),
    /// The expression is a binary operation.
    BinaryOp {
        /// The left operand of the binary operation.
        left: Box<Expr>,
        /// The binary operator.
        op: BinaryOperator,
        /// The right operand of the binary operation.
        right: Box<Expr>,
    },
    /// The expression is a function call.
    FunctionCall {
        /// The name of the function.
        name: String,
        /// The arguments of the function.
        args: Vec<Expr>,
    },
    /// The expression is a strong list.
    StrongList(Box<Expr>),
}

impl Expr {
    /// Formats this RollKit expression in a single line, with parentheses to indicate precedence.
    ///
    /// This is a wrapper around the [InlineFormatter](crate::parsing::InlineFormatter).
    ///
    /// # Example
    ///
    /// ```
    /// # use rollkit::{parsing::{ExprVisitor, InlineFormatter}, parse};
    /// let expr = parse("2d6 + 3").unwrap();
    /// let formatted = expr.format_inline();
    /// assert_eq!(formatted, "((2 d 6) + 3)");
    /// ```
    pub fn format_inline(&self) -> String {
        let mut formatter = InlineFormatter;
        formatter.visit_expr(self)
    }
}

/// Trait for visitors traversing RollKit expressions using the visitor pattern.
///
/// # Example
/// ```
/// # use rollkit::{parsing::{ExprVisitor, InlineFormatter}, parse};
/// let expr = parse("2d6 + 3").unwrap();
/// let mut formatter = InlineFormatter;
/// let formatted = ExprVisitor::visit_expr(&mut formatter, &expr);
/// assert_eq!(formatted, "((2 d 6) + 3)");
/// ```
pub trait ExprVisitor {
    /// The output type of the visitor.
    type Output;

    /// Visits a literal.
    fn visit_literal(&mut self, literal: &Literal) -> Self::Output;
    /// Visits a binary operation.
    fn visit_binary_op(&mut self, left: &Expr, op: &BinaryOperator, right: &Expr) -> Self::Output;
    /// Visits a function call.
    fn visit_function_call(&mut self, name: &str, args: &[Expr]) -> Self::Output;
    /// Visits a strong list.
    fn visit_strong_list(&mut self, expr: &Expr) -> Self::Output;

    /// Visits an expression.
    fn visit_expr(&mut self, expr: &Expr) -> Self::Output {
        match expr {
            Expr::Literal(lit) => self.visit_literal(lit),
            Expr::BinaryOp { left, op, right } => self.visit_binary_op(left, op, right),
            Expr::FunctionCall { name, args } => self.visit_function_call(name, args),
            Expr::StrongList(inner) => self.visit_strong_list(inner),
        }
    }
}

/// A formatter that formats RollKit expressions in a single line, with parentheses to indicate
/// precedence.
///
/// # Example
///
/// ```
/// # use rollkit::{parsing::{ExprVisitor, InlineFormatter}, parse};
/// let expr = parse("2d6 + 3").unwrap();
/// let mut formatter = InlineFormatter;
/// let formatted = formatter.visit_expr(&expr);
/// assert_eq!(formatted, "((2 d 6) + 3)");
/// ```
pub struct InlineFormatter;

impl ExprVisitor for InlineFormatter {
    type Output = String;

    fn visit_literal(&mut self, literal: &Literal) -> Self::Output {
        match literal {
            Literal::Int(n) => n.to_string(),
            Literal::List(lst) => format!(
                "{{{}}}",
                lst.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Literal::Range(RangeLiteral { start, end, step }) => {
                if let Some(s) = step {
                    format!("[{}, {}, {}]", start, end, s)
                } else {
                    format!("[{}, {}]", start, end)
                }
            }
        }
    }

    fn visit_binary_op(&mut self, left: &Expr, op: &BinaryOperator, right: &Expr) -> Self::Output {
        let left_str = self.visit_expr(left);
        let right_str = self.visit_expr(right);
        format!("({} {} {})", left_str, op, right_str)
    }

    fn visit_function_call(&mut self, name: &str, args: &[Expr]) -> Self::Output {
        let args_str = args
            .iter()
            .map(|arg| self.visit_expr(arg))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({})", name, args_str)
    }

    fn visit_strong_list(&mut self, expr: &Expr) -> Self::Output {
        format!("{{{}}}", self.visit_expr(expr))
    }
}
