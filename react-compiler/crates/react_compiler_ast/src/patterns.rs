use serde::{Deserialize, Serialize};

use crate::common::BaseNode;
use crate::common::RawNode;
use crate::expressions::{Expression, Identifier};

/// Covers assignment targets and patterns.
/// In Babel, LVal includes Identifier, MemberExpression, ObjectPattern, ArrayPattern,
/// RestElement, AssignmentPattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PatternLike {
    Identifier(Identifier),
    ObjectPattern(ObjectPattern),
    ArrayPattern(ArrayPattern),
    AssignmentPattern(AssignmentPattern),
    RestElement(RestElement),
    // Expressions can appear in pattern positions (e.g., MemberExpression as LVal)
    MemberExpression(crate::expressions::MemberExpression),
    TSAsExpression(crate::expressions::TSAsExpression),
    TSSatisfiesExpression(crate::expressions::TSSatisfiesExpression),
    TSNonNullExpression(crate::expressions::TSNonNullExpression),
    TSTypeAssertion(crate::expressions::TSTypeAssertion),
    // Flow's analogue of the TS cast wrappers: `(expr: SomeType)`.
    TypeCastExpression(crate::expressions::TypeCastExpression),
}

impl PatternLike {
    /// Convert to the matching [`Expression`] variant when this pattern shares
    /// a node `type` with `Expression` (i.e. it can appear in expression
    /// position), otherwise `None`.
    ///
    /// Reproduces exactly the set that `serde_json::from_value::<Expression>`
    /// of the same node would accept: the eight variants below wrap the same
    /// inner types as their `Expression` counterparts (`AssignmentPattern`
    /// included — `Expression` carries it for error-recovery positions), while
    /// the pattern-only variants (`ObjectPattern`, `ArrayPattern`,
    /// `RestElement`) are not expressions and yield `None`.
    pub fn as_expression(&self) -> Option<Expression> {
        match self {
            PatternLike::Identifier(x) => Some(Expression::Identifier(x.clone())),
            PatternLike::MemberExpression(x) => Some(Expression::MemberExpression(x.clone())),
            PatternLike::AssignmentPattern(x) => Some(Expression::AssignmentPattern(x.clone())),
            PatternLike::TSAsExpression(x) => Some(Expression::TSAsExpression(x.clone())),
            PatternLike::TSSatisfiesExpression(x) => {
                Some(Expression::TSSatisfiesExpression(x.clone()))
            }
            PatternLike::TSNonNullExpression(x) => Some(Expression::TSNonNullExpression(x.clone())),
            PatternLike::TSTypeAssertion(x) => Some(Expression::TSTypeAssertion(x.clone())),
            PatternLike::TypeCastExpression(x) => Some(Expression::TypeCastExpression(x.clone())),
            PatternLike::ObjectPattern(_)
            | PatternLike::ArrayPattern(_)
            | PatternLike::RestElement(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPattern {
    #[serde(flatten)]
    pub base: BaseNode,
    pub properties: Vec<ObjectPatternProperty>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "typeAnnotation"
    )]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ObjectPatternProperty {
    ObjectProperty(ObjectPatternProp),
    RestElement(RestElement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPatternProp {
    #[serde(flatten)]
    pub base: BaseNode,
    pub key: Box<Expression>,
    pub value: Box<PatternLike>,
    pub computed: bool,
    pub shorthand: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayPattern {
    #[serde(flatten)]
    pub base: BaseNode,
    pub elements: Vec<Option<PatternLike>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "typeAnnotation"
    )]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentPattern {
    #[serde(flatten)]
    pub base: BaseNode,
    pub left: Box<PatternLike>,
    pub right: Box<Expression>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "typeAnnotation"
    )]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestElement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub argument: Box<PatternLike>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "typeAnnotation"
    )]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[cfg(test)]
mod tests {
    use super::PatternLike;
    use crate::expressions::Expression;

    /// Variants shared with `Expression` coerce to the matching expression,
    /// reproducing what a `from_value::<Expression>` of the node accepted.
    #[test]
    fn as_expression_converts_shared_variants() {
        let ident: PatternLike =
            serde_json::from_value(serde_json::json!({ "type": "Identifier", "name": "x" }))
                .unwrap();
        assert!(matches!(
            ident.as_expression(),
            Some(Expression::Identifier(_))
        ));

        // AssignmentPattern is shared: `Expression` carries it for
        // error-recovery positions, so it must convert (not fall back).
        let assign: PatternLike = serde_json::from_value(serde_json::json!({
            "type": "AssignmentPattern",
            "left": { "type": "Identifier", "name": "x" },
            "right": { "type": "Identifier", "name": "y" }
        }))
        .unwrap();
        assert!(matches!(
            assign.as_expression(),
            Some(Expression::AssignmentPattern(_))
        ));
    }

    /// Pattern-only variants are not expressions and yield `None`.
    #[test]
    fn as_expression_rejects_pattern_only_variants() {
        let object: PatternLike = serde_json::from_value(
            serde_json::json!({ "type": "ObjectPattern", "properties": [] }),
        )
        .unwrap();
        assert!(object.as_expression().is_none());
    }
}
