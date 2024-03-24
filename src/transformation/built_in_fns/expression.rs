use crate::FeatureValueType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Expression {
    top: ExpressionNode,
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO(tatiana): recurisive expression
        f.write_fmt(format_args!("Expression"))
    }
}

#[derive(Serialize, Deserialize)]
pub(super) enum ExpressionNode {
    Column(String),
    Constant(String),
    Function(FunctionNode),
}

#[derive(Serialize, Deserialize)]
pub(super) struct FunctionNode {
    children: Vec<ExpressionNode>,
    op: String,
}

impl Expression {
    pub fn new(expr: &str) -> Self {
        // TODO(tatiana): parse expression, below is an example result
        Expression {
            top: ExpressionNode::Function(FunctionNode {
                children: vec![
                    ExpressionNode::Column(expr.to_string()),
                    ExpressionNode::Constant("1".to_string()),
                ],
                op: "=".to_string(),
            }),
        }
    }

    pub fn get_col_name(&self) -> Option<String> {
        // TODO(tatiana)
        None
    }

    pub fn get_type(&self) -> FeatureValueType {
        // TODO(tatiana)
        FeatureValueType::Int
    }
}
