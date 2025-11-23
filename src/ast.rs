use core::fmt;

/// A literal value in the RollKit expression AST.
#[derive(Debug, Clone)]
pub enum Literal {
    /// An integer literal.
    Int(i64),
    /// An explicit list literal e.g., `{1, 2, 3}`.
    List(Vec<i64>),
    /// A range list literal e.g., `[1, 5, 2]`.
    Range {
        start: i64,
        end: i64,
        step: Option<i64>,
    },
}

/// A binary operator in the RollKit expression AST.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    // Dice operations
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

    // Arithmetic operations
    /// The multiplication operator `*`.
    Multiplication,
    /// The addition operator `+`.
    Addition,
    /// The subtraction operator `-`.
    Subtraction,

    // Comparison operations
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

/// An expression in the RollKit expression AST.
#[derive(Debug, Clone)]
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
    /// Formats this RollKit expression in a single line.
    pub fn format_inline(&self) -> String {
        let mut formatter = InlineFormatter;
        formatter.visit_expr(self)
    }
}

/// A visitor for traversing RollKit expressions ASTs and returning a value.
pub trait ExprVisitor {
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

/// A formatter that formats RollKit expressions in a single line.
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
            Literal::Range { start, end, step } => {
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
