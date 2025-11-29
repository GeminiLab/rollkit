use rollkit::parsing::{BinaryOperator, Expr, ExprVisitor, Literal, range_to_iter};
use yansi::Paint;

/// Visitor that explains the structure of an expression
struct ExplainVisitor {
    depth: usize,
}

impl ExplainVisitor {
    fn new() -> Self {
        Self { depth: 2 }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }

    fn with_depth<F>(&mut self, f: F) -> String
    where
        F: FnOnce(&mut Self) -> String,
    {
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }
}

impl ExprVisitor for ExplainVisitor {
    type Output = String;

    fn visit_literal(&mut self, literal: &Literal) -> Self::Output {
        match literal {
            Literal::Int(n) => format!(
                "{}Literal: {} ({})",
                self.indent(),
                n.to_string().magenta(),
                "Integer".blue()
            ),
            Literal::List(lst) => {
                format!(
                    "{}Literal: {{{}}} ({} with {} elements)",
                    self.indent(),
                    lst.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .magenta(),
                    "List".blue(),
                    lst.len().to_string().blue()
                )
            }
            Literal::Range { start, end, step } => {
                let count = range_to_iter(*start, *end, *step).count();
                let repr = format!(
                    "{}, {}{}",
                    start,
                    end,
                    step.map(|s| format!(", {}", s)).unwrap_or_default()
                );
                format!(
                    "{}Literal: [{}] ({} with {} elements)",
                    self.indent(),
                    repr.magenta(),
                    "Range".blue(),
                    count.to_string().blue()
                )
            }
        }
    }

    fn visit_binary_op(&mut self, left: &Expr, op: &BinaryOperator, right: &Expr) -> Self::Output {
        let header = format!(
            "{}Binary Operation: {} ({})",
            self.indent(),
            op.to_str().magenta(),
            op.desc().blue()
        );
        let left_str = self.with_depth(|v| v.visit_expr(left));
        let right_str = self.with_depth(|v| v.visit_expr(right));

        format!("{}\n{}\n{}", header, left_str, right_str)
    }

    fn visit_function_call(&mut self, name: &str, args: &[Expr]) -> Self::Output {
        let header = format!(
            "{}Function Call: {} ({} args)",
            self.indent(),
            name.magenta(),
            args.len().to_string().blue()
        );
        let args_str = self.with_depth(|v| {
            args.iter()
                .map(|arg| v.visit_expr(arg))
                .collect::<Vec<_>>()
                .join("\n")
        });

        if args.is_empty() {
            header
        } else {
            format!("{}\n{}", header, args_str)
        }
    }

    fn visit_strong_list(&mut self, expr: &Expr) -> Self::Output {
        let header = format!("{}Strong List:", self.indent());
        let inner_str = self.with_depth(|v| v.visit_expr(expr));
        format!("{}\n{}", header, inner_str)
    }
}

/// Print explanation of the expression structure
pub fn explain_expr(expr: &Expr) {
    let mut visitor = ExplainVisitor::new();
    let explanation = visitor.visit_expr(expr);
    println!("{}", explanation);
}
