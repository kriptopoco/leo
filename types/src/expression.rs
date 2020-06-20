use crate::{CircuitFieldDefinition, Identifier, Integer, RangeOrExpression, Span, SpreadOrExpression};
use leo_ast::{
    access::{Access, AssigneeAccess},
    common::{Assignee, Identifier as AstIdentifier},
    expressions::{
        ArrayInitializerExpression,
        ArrayInlineExpression,
        BinaryExpression,
        CircuitInlineExpression,
        Expression as AstExpression,
        NotExpression,
        PostfixExpression,
        TernaryExpression,
    },
    operations::BinaryOperation,
    values::{BooleanValue, FieldValue, GroupValue, IntegerValue, NumberImplicitValue, Value},
};

use snarkos_models::gadgets::utilities::boolean::Boolean;

use std::fmt;

/// Expression that evaluates to a value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    // Identifier
    Identifier(Identifier),

    // Values
    Integer(Integer),
    Field(String),
    Group(String),
    Boolean(Boolean),
    Implicit(String),

    // Number operations
    Add(Box<Expression>, Box<Expression>, Span),
    Sub(Box<Expression>, Box<Expression>, Span),
    Mul(Box<Expression>, Box<Expression>, Span),
    Div(Box<Expression>, Box<Expression>, Span),
    Pow(Box<Expression>, Box<Expression>, Span),

    // Boolean operations
    Not(Box<Expression>),
    Or(Box<Expression>, Box<Expression>, Span),
    And(Box<Expression>, Box<Expression>, Span),
    Eq(Box<Expression>, Box<Expression>, Span),
    Ge(Box<Expression>, Box<Expression>, Span),
    Gt(Box<Expression>, Box<Expression>, Span),
    Le(Box<Expression>, Box<Expression>, Span),
    Lt(Box<Expression>, Box<Expression>, Span),

    // Conditionals
    IfElse(Box<Expression>, Box<Expression>, Box<Expression>, Span),

    // Arrays
    Array(Vec<Box<SpreadOrExpression>>, Span),
    ArrayAccess(Box<Expression>, Box<RangeOrExpression>, Span), // (array name, range)

    // Circuits
    Circuit(Identifier, Vec<CircuitFieldDefinition>, Span),
    CircuitMemberAccess(Box<Expression>, Identifier, Span), // (declared circuit name, circuit member name)
    CircuitStaticFunctionAccess(Box<Expression>, Identifier, Span), // (defined circuit name, circuit static member name)

    // Functions
    FunctionCall(Box<Expression>, Vec<Expression>, Span),
}

impl<'ast> Expression {
    pub(crate) fn get_count(count: Value<'ast>) -> usize {
        match count {
            Value::Integer(integer) => integer
                .number
                .value
                .parse::<usize>()
                .expect("Unable to read array size"),
            Value::Implicit(number) => number.number.value.parse::<usize>().expect("Unable to read array size"),
            size => unimplemented!("Array size should be an integer {}", size),
        }
    }
}

impl<'ast> fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Variables
            Expression::Identifier(ref variable) => write!(f, "{}", variable),

            // Values
            Expression::Integer(ref integer) => write!(f, "{}", integer),
            Expression::Field(ref field) => write!(f, "{}", field),
            Expression::Group(ref group) => write!(f, "{}", group),
            Expression::Boolean(ref bool) => write!(f, "{}", bool.get_value().unwrap()),
            Expression::Implicit(ref value) => write!(f, "{}", value),

            // Number operations
            Expression::Add(ref left, ref right, ref _span) => write!(f, "{} + {}", left, right),
            Expression::Sub(ref left, ref right, ref _span) => write!(f, "{} - {}", left, right),
            Expression::Mul(ref left, ref right, ref _span) => write!(f, "{} * {}", left, right),
            Expression::Div(ref left, ref right, ref _span) => write!(f, "{} / {}", left, right),
            Expression::Pow(ref left, ref right, ref _span) => write!(f, "{} ** {}", left, right),

            // Boolean operations
            Expression::Not(ref expression) => write!(f, "!{}", expression),
            Expression::Or(ref lhs, ref rhs, ref _span) => write!(f, "{} || {}", lhs, rhs),
            Expression::And(ref lhs, ref rhs, ref _span) => write!(f, "{} && {}", lhs, rhs),
            Expression::Eq(ref lhs, ref rhs, ref _span) => write!(f, "{} == {}", lhs, rhs),
            Expression::Ge(ref lhs, ref rhs, ref _span) => write!(f, "{} >= {}", lhs, rhs),
            Expression::Gt(ref lhs, ref rhs, ref _span) => write!(f, "{} > {}", lhs, rhs),
            Expression::Le(ref lhs, ref rhs, ref _span) => write!(f, "{} <= {}", lhs, rhs),
            Expression::Lt(ref lhs, ref rhs, ref _span) => write!(f, "{} < {}", lhs, rhs),

            // Conditionals
            Expression::IfElse(ref first, ref second, ref third, ref _span) => {
                write!(f, "if {} then {} else {} fi", first, second, third)
            }

            // Arrays
            Expression::Array(ref array, ref _span) => {
                write!(f, "[")?;
                for (i, e) in array.iter().enumerate() {
                    write!(f, "{}", e)?;
                    if i < array.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Expression::ArrayAccess(ref array, ref index, ref _span) => write!(f, "{}[{}]", array, index),

            // Circuits
            Expression::Circuit(ref var, ref members, ref _span) => {
                write!(f, "{} {{", var)?;
                for (i, member) in members.iter().enumerate() {
                    write!(f, "{}: {}", member.identifier, member.expression)?;
                    if i < members.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            Expression::CircuitMemberAccess(ref circuit_name, ref member, ref _span) => {
                write!(f, "{}.{}", circuit_name, member)
            }
            Expression::CircuitStaticFunctionAccess(ref circuit_name, ref member, ref _span) => {
                write!(f, "{}::{}", circuit_name, member)
            }

            // Function calls
            Expression::FunctionCall(ref function, ref arguments, ref _span) => {
                write!(f, "{}(", function,)?;
                for (i, param) in arguments.iter().enumerate() {
                    write!(f, "{}", param)?;
                    if i < arguments.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
        }
    }
}

impl<'ast> From<CircuitInlineExpression<'ast>> for Expression {
    fn from(expression: CircuitInlineExpression<'ast>) -> Self {
        let variable = Identifier::from(expression.identifier);
        let members = expression
            .members
            .into_iter()
            .map(|member| CircuitFieldDefinition::from(member))
            .collect::<Vec<CircuitFieldDefinition>>();

        Expression::Circuit(variable, members, Span::from(expression.span))
    }
}

impl<'ast> From<PostfixExpression<'ast>> for Expression {
    fn from(expression: PostfixExpression<'ast>) -> Self {
        let variable = Expression::Identifier(Identifier::from(expression.identifier));

        // ast::PostFixExpression contains an array of "accesses": `a(34)[42]` is represented as `[a, [Call(34), Select(42)]]`, but Access call expressions
        // are recursive, so it is `Select(Call(a, 34), 42)`. We apply this transformation here

        // we start with the id, and we fold the array of accesses by wrapping the current value
        expression
            .accesses
            .into_iter()
            .fold(variable, |acc, access| match access {
                // Handle array accesses
                Access::Array(array) => Expression::ArrayAccess(
                    Box::new(acc),
                    Box::new(RangeOrExpression::from(array.expression)),
                    Span::from(array.span),
                ),

                // Handle function calls
                Access::Call(function) => Expression::FunctionCall(
                    Box::new(acc),
                    function
                        .expressions
                        .into_iter()
                        .map(|expression| Expression::from(expression))
                        .collect(),
                    Span::from(function.span),
                ),

                // Handle circuit member accesses
                Access::Object(circuit_object) => Expression::CircuitMemberAccess(
                    Box::new(acc),
                    Identifier::from(circuit_object.identifier),
                    Span::from(circuit_object.span),
                ),
                Access::StaticObject(circuit_object) => Expression::CircuitStaticFunctionAccess(
                    Box::new(acc),
                    Identifier::from(circuit_object.identifier),
                    Span::from(circuit_object.span),
                ),
            })
    }
}

impl<'ast> From<AstExpression<'ast>> for Expression {
    fn from(expression: AstExpression<'ast>) -> Self {
        match expression {
            AstExpression::Value(value) => Expression::from(value),
            AstExpression::Identifier(variable) => Expression::from(variable),
            AstExpression::Not(expression) => Expression::from(expression),
            AstExpression::Binary(expression) => Expression::from(expression),
            AstExpression::Ternary(expression) => Expression::from(expression),
            AstExpression::ArrayInline(expression) => Expression::from(expression),
            AstExpression::ArrayInitializer(expression) => Expression::from(expression),
            AstExpression::CircuitInline(expression) => Expression::from(expression),
            AstExpression::Postfix(expression) => Expression::from(expression),
        }
    }
}

// Assignee -> Expression for operator assign statements
impl<'ast> From<Assignee<'ast>> for Expression {
    fn from(assignee: Assignee<'ast>) -> Self {
        let variable = Expression::Identifier(Identifier::from(assignee.identifier));

        // we start with the id, and we fold the array of accesses by wrapping the current value
        assignee
            .accesses
            .into_iter()
            .fold(variable, |acc, access| match access {
                AssigneeAccess::Member(circuit_member) => Expression::CircuitMemberAccess(
                    Box::new(acc),
                    Identifier::from(circuit_member.identifier),
                    Span::from(circuit_member.span),
                ),
                AssigneeAccess::Array(array) => Expression::ArrayAccess(
                    Box::new(acc),
                    Box::new(RangeOrExpression::from(array.expression)),
                    Span::from(array.span),
                ),
            })
    }
}

impl<'ast> From<BinaryExpression<'ast>> for Expression {
    fn from(expression: BinaryExpression<'ast>) -> Self {
        match expression.operation {
            // Boolean operations
            BinaryOperation::Or => Expression::Or(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::And => Expression::And(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Eq => Expression::Eq(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Ne => Expression::Not(Box::new(Expression::from(expression))),
            BinaryOperation::Ge => Expression::Ge(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Gt => Expression::Gt(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Le => Expression::Le(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Lt => Expression::Lt(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            // Number operations
            BinaryOperation::Add => Expression::Add(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Sub => Expression::Sub(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Mul => Expression::Mul(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Div => Expression::Div(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
            BinaryOperation::Pow => Expression::Pow(
                Box::new(Expression::from(*expression.left)),
                Box::new(Expression::from(*expression.right)),
                Span::from(expression.span),
            ),
        }
    }
}

impl<'ast> From<TernaryExpression<'ast>> for Expression {
    fn from(expression: TernaryExpression<'ast>) -> Self {
        Expression::IfElse(
            Box::new(Expression::from(*expression.first)),
            Box::new(Expression::from(*expression.second)),
            Box::new(Expression::from(*expression.third)),
            Span::from(expression.span),
        )
    }
}

impl<'ast> From<ArrayInlineExpression<'ast>> for Expression {
    fn from(array: ArrayInlineExpression<'ast>) -> Self {
        Expression::Array(
            array
                .expressions
                .into_iter()
                .map(|s_or_e| Box::new(SpreadOrExpression::from(s_or_e)))
                .collect(),
            Span::from(array.span),
        )
    }
}

impl<'ast> From<ArrayInitializerExpression<'ast>> for Expression {
    fn from(array: ArrayInitializerExpression<'ast>) -> Self {
        let count = Expression::get_count(array.count);
        let expression = Box::new(SpreadOrExpression::from(*array.expression));

        Expression::Array(vec![expression; count], Span::from(array.span))
    }
}

impl<'ast> From<Value<'ast>> for Expression {
    fn from(value: Value<'ast>) -> Self {
        match value {
            Value::Integer(num) => Expression::from(num),
            Value::Field(field) => Expression::from(field),
            Value::Group(group) => Expression::from(group),
            Value::Boolean(bool) => Expression::from(bool),
            Value::Implicit(value) => Expression::from(value),
        }
    }
}

impl<'ast> From<NotExpression<'ast>> for Expression {
    fn from(expression: NotExpression<'ast>) -> Self {
        Expression::Not(Box::new(Expression::from(*expression.expression)))
    }
}

impl<'ast> From<FieldValue<'ast>> for Expression {
    fn from(field: FieldValue<'ast>) -> Self {
        Expression::Field(field.number.value)
    }
}

impl<'ast> From<GroupValue<'ast>> for Expression {
    fn from(group: GroupValue<'ast>) -> Self {
        Expression::Group(group.to_string())
    }
}

impl<'ast> From<BooleanValue<'ast>> for Expression {
    fn from(boolean: BooleanValue<'ast>) -> Self {
        Expression::Boolean(Boolean::Constant(
            boolean.value.parse::<bool>().expect("unable to parse boolean"),
        ))
    }
}

impl<'ast> From<NumberImplicitValue<'ast>> for Expression {
    fn from(number: NumberImplicitValue<'ast>) -> Self {
        Expression::Implicit(number.number.value)
    }
}

impl<'ast> From<IntegerValue<'ast>> for Expression {
    fn from(field: IntegerValue<'ast>) -> Self {
        Expression::Integer(Integer::from(field.number, field._type))
    }
}

impl<'ast> From<AstIdentifier<'ast>> for Expression {
    fn from(identifier: AstIdentifier<'ast>) -> Self {
        Expression::Identifier(Identifier::from(identifier))
    }
}
