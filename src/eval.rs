use rand::{
    Rng, rng,
    seq::{IndexedRandom, SliceRandom},
};

use crate::ast::{BinaryOperator, Expr, ExprVisitor, Literal};

/// The value returned by the evaluator.
#[derive(Debug, Clone)]
pub enum Value {
    /// An integer value.
    Integer(i64),
    /// A list value.
    List(Vec<i64>),
}

/// The internal representation of a list, which can be either a concrete list of integers
/// or a range defined by a start, end, and optional step. Used during evaluation for efficiency.
#[derive(Debug, Clone)]
pub enum ListInner {
    /// A concrete list of integers.
    List(Vec<i64>),
    /// A range defined by start, end, and optional step.
    Range {
        start: i64,
        end: i64,
        step: Option<i64>,
    },
}

impl ListInner {
    /// Returns an iterator over the range defined by start, end, and step.
    fn iter_range(start: i64, end: i64, step: Option<i64>) -> impl Iterator<Item = i64> {
        let inc = end >= start;
        let step = step.map(i64::wrapping_abs).unwrap_or(1);
        let step = if inc { step } else { step.wrapping_neg() };

        std::iter::successors(Some(start), move |&cur| {
            let next = cur.wrapping_add(step);
            if (inc && (next > end || next < cur)) || (!inc && (next < end || next > cur)) {
                None
            } else {
                Some(next)
            }
        })
    }

    /// Returns the sum of all elements in the list.
    pub fn sum(&self) -> i64 {
        match self {
            ListInner::List(lst) => lst.iter().sum(),
            ListInner::Range { start, end, step } => Self::iter_range(*start, *end, *step).sum(),
        }
    }

    /// Clones the list into a concrete Vec<i64>.
    pub fn clone_vec(&self) -> Vec<i64> {
        self.clone().into_vec()
    }

    /// Converts the list into a concrete Vec<i64>.
    pub fn into_vec(self) -> Vec<i64> {
        match self {
            ListInner::List(lst) => lst,
            ListInner::Range { start, end, step } => Self::iter_range(start, end, step).collect(),
        }
    }

    /// Samples `count` random elements from the list using the provided random number generator.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R, count: i64) -> Vec<i64> {
        let mut sampler: Box<dyn FnMut() -> i64> = match self {
            ListInner::List(lst) => Box::new(|| *lst.choose(rng).unwrap()),
            ListInner::Range { start, end, step } => {
                if step.is_none_or(|step| step.wrapping_abs() == 1) {
                    let range = if end >= start {
                        *start..=*end
                    } else {
                        *end..=*start
                    };
                    Box::new(move || rng.random_range(range.clone()).into())
                } else {
                    let values: Vec<i64> = self.clone_vec();
                    Box::new(move || *values.choose(rng).unwrap())
                }
            }
        };

        (0..count).map(|_| sampler()).collect()
    }
}

/// The internal representation of a value during evaluation.
#[derive(Debug, Clone)]
enum InnerValue {
    /// An integer value.
    Integer(i64),
    /// A list value, with a flag indicating if it's strong or weak.
    List { strong: bool, inner: ListInner },
}

impl InnerValue {
    /// Asserts that the value is an integer and returns it, or returns an error.
    pub fn assert_integer(self) -> Result<i64, EvalError> {
        match self {
            InnerValue::Integer(i) => Ok(i),
            _ => Err(EvalError::IntegerExpected),
        }
    }

    /// Asserts that the value is a list and returns its inner representation and strength, or
    /// returns an error.
    pub fn assert_list(self) -> Result<(bool, ListInner), EvalError> {
        match self {
            InnerValue::List { strong, inner } => Ok((strong, inner)),
            _ => Err(EvalError::ListExpected),
        }
    }

    /// Tries to convert the value into an integer. If it's a strong list, returns the inner
    /// representation of the list as an error.
    pub fn try_into_integer(self) -> Result<i64, ListInner> {
        match self {
            InnerValue::Integer(i) => Ok(i),
            InnerValue::List {
                strong: false,
                inner,
            } => Ok(inner.sum()),
            InnerValue::List {
                strong: true,
                inner,
            } => Err(inner),
        }
    }

    /// Converts the internal value into a public representation, i.e., [`Value`].
    fn into_public(self) -> Value {
        match self {
            InnerValue::Integer(i) => Value::Integer(i),
            InnerValue::List { inner, .. } => Value::List(inner.into_vec()),
        }
    }
}

/// Errors that can occur during evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalError {
    /// An integer was expected but a different type was found.
    IntegerExpected,
    /// A list was expected but a different type was found.
    ListExpected,
    /// Tried to keep more elements than available in the list.
    KeepTooMany { available: usize, requested: i64 },
    /// Tried to drop more elements than available in the list.
    DropTooMany { available: usize, requested: i64 },
    /// Tried to keep fewer elements than zero.
    KeepTooLess { requested: i64 },
    /// Tried to drop fewer elements than zero.
    DropTooLess { requested: i64 },
    /// The lengths of two lists did not match.
    ListMismatch { left_len: usize, right_len: usize },
}

/// The evaluator visitor that traverses the AST and computes the result.
struct EvalVisitor<'a, R>
where
    R: Rng + ?Sized,
{
    rng: &'a mut R,
}

/// Evaluates keep/drop operations on lists.
fn eval_keep_drop_op<R: Rng + ?Sized>(
    left: InnerValue,
    right: InnerValue,
    keep: bool,
    highest: bool,
    rng: &mut R,
) -> Result<InnerValue, EvalError> {
    let (strong, list) = left.assert_list()?;
    let mut vec = list.into_vec();
    let requested = right.assert_integer()?;
    let available = vec.len();

    if requested < 0 {
        return if keep {
            Err(EvalError::KeepTooLess { requested })
        } else {
            Err(EvalError::DropTooLess { requested })
        };
    }

    if requested as usize > available {
        return if keep {
            Err(EvalError::KeepTooMany {
                available,
                requested,
            })
        } else {
            Err(EvalError::DropTooMany {
                available,
                requested,
            })
        };
    }

    vec.sort_unstable_by(|a, b| if keep ^ highest { a.cmp(b) } else { b.cmp(a) });

    vec.truncate(if keep {
        requested as usize
    } else {
        available - requested as usize
    });

    vec.shuffle(rng);

    Ok(InnerValue::List {
        strong: strong,
        inner: ListInner::List(vec),
    })
}

/// Evaluates arithmetic and comparison operations on integers and lists.
fn eval_arith_cmp_op(
    left: InnerValue,
    right: InnerValue,
    op: fn(i64, i64) -> i64,
) -> Result<InnerValue, EvalError> {
    match (left.try_into_integer(), right.try_into_integer()) {
        (Ok(l), Ok(r)) => Ok(InnerValue::Integer(op(l, r))),
        (Ok(l), Err(list)) => {
            let mut vec = list.into_vec();
            for r in &mut vec {
                *r = op(l, *r);
            }
            Ok(InnerValue::List {
                strong: true,
                inner: ListInner::List(vec),
            })
        }
        (Err(list), Ok(r)) => {
            let mut vec = list.into_vec();
            for l in &mut vec {
                *l = op(*l, r);
            }
            Ok(InnerValue::List {
                strong: true,
                inner: ListInner::List(vec),
            })
        }
        (Err(list), Err(rlist)) => {
            let lvec = list.into_vec();
            let rvec = rlist.into_vec();
            if lvec.len() != rvec.len() {
                return Err(EvalError::ListMismatch {
                    left_len: lvec.len(),
                    right_len: rvec.len(),
                });
            }

            let vec: Vec<i64> = lvec
                .into_iter()
                .zip(rvec.into_iter())
                .map(|(l, r)| op(l, r))
                .collect();

            Ok(InnerValue::List {
                strong: true,
                inner: ListInner::List(vec),
            })
        }
    }
}

/// A wrapper macro to create "0-1" comparison operations.
macro_rules! bi_cmp_op {
    ($op:tt) => {
        |a: i64, b: i64| if a $op b { 1 } else { 0 }
    };
}

impl<'a, R> ExprVisitor for EvalVisitor<'a, R>
where
    R: Rng + ?Sized,
{
    type Output = Result<InnerValue, EvalError>;

    fn visit_literal(&mut self, literal: &Literal) -> Self::Output {
        Ok(match literal {
            Literal::Int(n) => InnerValue::Integer(*n),
            Literal::List(lst) => InnerValue::List {
                strong: false,
                inner: ListInner::List(lst.clone()),
            },
            Literal::Range { start, end, step } => InnerValue::List {
                strong: false,
                inner: ListInner::Range {
                    start: *start,
                    end: *end,
                    step: *step,
                },
            },
        })
    }

    fn visit_binary_op(&mut self, left: &Expr, op: &BinaryOperator, right: &Expr) -> Self::Output {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;

        match op {
            BinaryOperator::DiceRoll => {
                let count = left.assert_integer()?;
                let sides = match right {
                    InnerValue::Integer(n) => ListInner::Range {
                        start: 1,
                        end: n,
                        step: None,
                    },
                    InnerValue::List { inner, .. } => inner,
                };

                Ok(InnerValue::List {
                    strong: false,
                    inner: ListInner::List(sides.sample(self.rng, count)),
                })
            }
            BinaryOperator::KeepHighest => eval_keep_drop_op(left, right, true, true, self.rng),
            BinaryOperator::KeepLowest => eval_keep_drop_op(left, right, true, false, self.rng),
            BinaryOperator::DropHighest => eval_keep_drop_op(left, right, false, true, self.rng),
            BinaryOperator::DropLowest => eval_keep_drop_op(left, right, false, false, self.rng),
            BinaryOperator::Multiplication => eval_arith_cmp_op(left, right, i64::wrapping_mul),
            BinaryOperator::Addition => eval_arith_cmp_op(left, right, i64::wrapping_add),
            BinaryOperator::Subtraction => eval_arith_cmp_op(left, right, i64::wrapping_sub),
            BinaryOperator::Equal => eval_arith_cmp_op(left, right, bi_cmp_op!(==)),
            BinaryOperator::NotEqual => eval_arith_cmp_op(left, right, bi_cmp_op!(!=)),
            BinaryOperator::LessThan => eval_arith_cmp_op(left, right, bi_cmp_op!(<)),
            BinaryOperator::LessEqual => eval_arith_cmp_op(left, right, bi_cmp_op!(<=)),
            BinaryOperator::GreaterThan => eval_arith_cmp_op(left, right, bi_cmp_op!(>)),
            BinaryOperator::GreaterEqual => eval_arith_cmp_op(left, right, bi_cmp_op!(>=)),
        }
    }

    fn visit_function_call(&mut self, name: &str, args: &[Expr]) -> Self::Output {
        todo!(
            "Function calls are not yet implemented: {}, {:?}",
            name,
            args
        )
    }

    fn visit_strong_list(&mut self, expr: &Expr) -> Self::Output {
        match self.visit_expr(expr)? {
            InnerValue::Integer(i) => Ok(InnerValue::List {
                strong: false,
                inner: ListInner::List(vec![i]),
            }),
            InnerValue::List { inner, .. } => Ok(InnerValue::List {
                strong: true,
                inner,
            }),
        }
    }
}

/// Evaluates an expression and returns the result.
pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    eval_with(expr, &mut rng())
}

/// Evaluates an expression using the provided random number generator and returns the result.
pub fn eval_with<R: Rng + ?Sized>(expr: &Expr, rng: &mut R) -> Result<Value, EvalError> {
    let mut visitor = EvalVisitor { rng };
    visitor.visit_expr(expr).map(InnerValue::into_public)
}
