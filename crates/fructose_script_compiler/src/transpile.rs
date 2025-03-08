use std::{cell::Cell, mem, ops::Range};

use fructose_script_parser::{Visit, ast};
use oxc::{
    allocator::{Allocator, FromIn},
    ast::ast::{self as oxc_ast, Expression, Statement},
    span::{Atom, GetSpan, SPAN, Span},
};

struct Block<'a> {
    statements: oxc::allocator::Vec<'a, Statement<'a>>,
}

impl<'a> Block<'a> {
    #[inline]
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            statements: oxc::allocator::Vec::new_in(allocator),
        }
    }

    #[inline]
    fn append_statement(&mut self, statement: Statement<'a>) {
        self.statements.push(statement);
    }
}

pub struct JsGenerator<'a> {
    allocator: &'a Allocator,
    current_block: Block<'a>,
}

impl<'a> JsGenerator<'a> {
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            current_block: Block::new(allocator),
        }
    }

    pub fn into_program(self) -> oxc_ast::Program<'a> {
        use oxc_ast::{Program, SourceType};
        Program {
            span: SPAN,
            source_type: SourceType::mjs(),
            source_text: "",
            comments: oxc::allocator::Vec::new_in(self.allocator),
            hashbang: None,
            directives: oxc::allocator::Vec::new_in(self.allocator),
            body: self.current_block.statements,
            scope_id: Cell::new(None),
        }
    }

    #[inline]
    fn boxed<T>(&self, value: T) -> oxc::allocator::Box<'a, T> {
        oxc::allocator::Box::new_in(value, self.allocator)
    }
    #[inline]
    fn atom(&self, name: &str) -> Atom<'a> {
        Atom::from_in(name, self.allocator)
    }
}

#[inline]
fn span(range: &Range<usize>) -> Span {
    Span::new(range.start as u32, range.end as u32)
}

impl<'a> Visit for JsGenerator<'a> {
    fn visit_let(&mut self, node: &ast::Let) {
        use oxc_ast::{
            BindingIdentifier, BindingPattern, BindingPatternKind, VariableDeclaration,
            VariableDeclarationKind, VariableDeclarator,
        };

        let value = self.visit_expression(&node.init);

        let mut declarations = oxc::allocator::Vec::new_in(self.allocator);
        let binding_identifier = self.boxed(BindingIdentifier {
            span: span(&node.name.range),
            name: self.atom(&node.name.value),
            symbol_id: Cell::new(None),
        });

        let span = span(&node.range);

        declarations.push(VariableDeclarator {
            span,
            kind: VariableDeclarationKind::Let,
            id: BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(binding_identifier),
                type_annotation: None,
                optional: false,
            },
            init: value,
            definite: false,
        });

        let statement = self.boxed(VariableDeclaration {
            span,
            kind: VariableDeclarationKind::Let,
            declarations,
            declare: false,
        });

        self.current_block
            .append_statement(Statement::VariableDeclaration(statement));
    }

    fn visit_assign(&mut self, node: &ast::Assign) {
        use oxc_ast::{AssignmentExpression, AssignmentOperator, AssignmentTarget};

        let target = self.boxed(oxc_ast::IdentifierReference {
            span: span(&node.range),
            name: self.atom(&node.target.value),
            reference_id: Cell::new(None),
        });
        let value = self.visit_expression(&node.value);

        let assignment = self.boxed(AssignmentExpression {
            span: span(&node.range),
            operator: AssignmentOperator::Assign,
            left: AssignmentTarget::AssignmentTargetIdentifier(target),
            right: value.unwrap(),
        });

        let statement = self.boxed(oxc_ast::ExpressionStatement {
            span: span(&node.range),
            expression: Expression::AssignmentExpression(assignment),
        });

        self.current_block
            .append_statement(Statement::ExpressionStatement(statement));
    }

    type Result = Option<Expression<'a>>;

    fn visit_ident(&mut self, node: &ast::Ident) -> Self::Result {
        let identifier = self.boxed(oxc_ast::IdentifierReference {
            span: span(&node.range),
            name: self.atom(&node.value),
            reference_id: Cell::new(None),
        });

        Some(Expression::Identifier(identifier))
    }

    fn visit_nat_literal(&mut self, node: &ast::NatLiteral) -> Self::Result {
        use oxc_ast::{NumberBase, NumericLiteral};

        let numeric_literal = self.boxed(NumericLiteral {
            span: span(&node.range),
            value: node.value as f64,
            raw: None,
            base: NumberBase::Decimal,
        });

        Some(Expression::NumericLiteral(numeric_literal))
    }

    fn visit_block(&mut self, node: &ast::Block) -> Self::Result {
        for statement in &node.statements {
            self.visit_statement(statement);
        }

        node.last
            .as_ref()
            .and_then(|last| self.visit_expression(last))
    }

    fn visit_fn(&mut self, node: &ast::Fn) -> Self::Result {
        use oxc_ast::{
            ArrowFunctionExpression, BindingIdentifier, BindingPattern, BindingPatternKind,
            FormalParameter, FormalParameterKind, FormalParameters, FunctionBody, ReturnStatement,
        };

        let parent_block = mem::replace(&mut self.current_block, Block::new(self.allocator));

        let mut parameters = oxc::allocator::Vec::new_in(self.allocator);
        for parameter in &node.parameters {
            let binding_identifier = self.boxed(BindingIdentifier {
                span: span(&parameter.range),
                name: self.atom(&parameter.value),
                symbol_id: Cell::new(None),
            });

            parameters.push(FormalParameter {
                span: span(&parameter.range),
                decorators: oxc::allocator::Vec::new_in(self.allocator),
                pattern: BindingPattern {
                    kind: BindingPatternKind::BindingIdentifier(binding_identifier),
                    type_annotation: None,
                    optional: false,
                },
                accessibility: None,
                readonly: false,
                r#override: false,
            });
        }

        if let Some(result) = self.visit_expression(&node.body) {
            let r#return = self.boxed(ReturnStatement {
                span: result.span(),
                argument: Some(result),
            });

            self.current_block
                .append_statement(Statement::ReturnStatement(r#return));
        }

        let current_block = mem::replace(&mut self.current_block, parent_block);

        let arrow_fn = self.boxed(ArrowFunctionExpression {
            span: span(&node.range),
            expression: false,
            r#async: false,
            type_parameters: None,
            params: self.boxed(FormalParameters {
                span: span(&node.range),
                kind: FormalParameterKind::ArrowFormalParameters,
                items: parameters,
                rest: None,
            }),
            return_type: None,
            body: self.boxed(FunctionBody {
                span: span(&node.range),
                directives: oxc::allocator::Vec::new_in(self.allocator),
                statements: current_block.statements,
            }),
            scope_id: Cell::new(None),
        });

        Some(Expression::ArrowFunctionExpression(arrow_fn))
    }
}
