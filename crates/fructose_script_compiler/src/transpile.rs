use std::{mem, ops::Range};

use fructose_script_parser::{Visit, ast};
use oxc::{
    allocator::Allocator,
    ast::{AstBuilder, NONE, ast as oxc_ast},
    span::{GetSpan, SPAN, Span},
};

struct Block<'a> {
    statements: oxc::allocator::Vec<'a, oxc_ast::Statement<'a>>,
}

impl<'a> Block<'a> {
    #[inline]
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            statements: oxc::allocator::Vec::new_in(allocator),
        }
    }

    #[inline]
    fn append_statement(&mut self, statement: oxc_ast::Statement<'a>) {
        self.statements.push(statement);
    }
}

pub struct JsGenerator<'a> {
    ast: AstBuilder<'a>,
    current_block: Block<'a>,
}

impl<'a> JsGenerator<'a> {
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            ast: AstBuilder::new(allocator),
            current_block: Block::new(allocator),
        }
    }

    pub fn into_program(self) -> oxc_ast::Program<'a> {
        use oxc_ast::SourceType;

        self.ast.program(
            SPAN,
            SourceType::mjs(),
            "",
            self.ast.vec(),
            None,
            self.ast.vec(),
            self.current_block.statements,
        )
    }
}

#[inline]
fn span(range: &Range<usize>) -> Span {
    Span::new(range.start as u32, range.end as u32)
}

impl<'a> Visit for JsGenerator<'a> {
    fn visit_let(&mut self, node: &ast::Let) {
        use oxc_ast::VariableDeclarationKind;

        let value = self.visit_expression(&node.init);

        let pattern = self.ast.binding_pattern(
            self.ast
                .binding_pattern_kind_binding_identifier(span(&node.name.range), &node.name.value),
            NONE,
            false,
        );

        let declarations = self.ast.vec1(self.ast.variable_declarator(
            span(&node.range),
            VariableDeclarationKind::Let,
            pattern,
            value,
            false,
        ));

        let declaration = self.ast.declaration_variable(
            span(&node.range),
            VariableDeclarationKind::Let,
            declarations,
            false,
        );

        self.current_block.append_statement(declaration.into());
    }

    fn visit_assign(&mut self, node: &ast::Assign) {
        use oxc_ast::AssignmentOperator;

        let target = self
            .ast
            .simple_assignment_target_assignment_target_identifier(
                span(&node.range),
                &node.target.value,
            );

        let value = self.visit_expression(&node.value);

        let assignment = self.ast.expression_assignment(
            span(&node.range),
            AssignmentOperator::Assign,
            target.into(),
            value.unwrap_or_else(|| self.ast.void_0(SPAN)),
        );

        let statement = self.ast.statement_expression(span(&node.range), assignment);

        self.current_block.append_statement(statement);
    }

    type Result = Option<oxc_ast::Expression<'a>>;

    fn visit_ident(&mut self, node: &ast::Ident) -> Self::Result {
        Some(
            self.ast
                .expression_identifier(span(&node.range), &node.value),
        )
    }

    fn visit_nat_literal(&mut self, node: &ast::NatLiteral) -> Self::Result {
        use oxc_ast::NumberBase;

        Some(self.ast.expression_numeric_literal(
            span(&node.range),
            node.value as f64,
            None,
            NumberBase::Decimal,
        ))
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
        use oxc_ast::FormalParameterKind;

        let parent_block = mem::replace(&mut self.current_block, Block::new(self.ast.allocator));

        let mut parameters = self.ast.vec();
        for parameter in &node.parameters {
            let pattern = self.ast.binding_pattern(
                self.ast.binding_pattern_kind_binding_identifier(
                    span(&parameter.range),
                    &parameter.value,
                ),
                NONE,
                false,
            );

            parameters.push(self.ast.formal_parameter(
                span(&parameter.range),
                self.ast.vec(),
                pattern,
                None,
                false,
                false,
            ));
        }

        if let Some(result) = self.visit_expression(&node.body) {
            let r#return = self.ast.statement_return(result.span(), Some(result));
            self.current_block.append_statement(r#return);
        }

        let current_block = mem::replace(&mut self.current_block, parent_block);

        Some(self.ast.expression_arrow_function(
            span(&node.range),
            false,
            false,
            NONE,
            self.ast.alloc_formal_parameters(
                span(&node.range),
                FormalParameterKind::ArrowFormalParameters,
                parameters,
                NONE,
            ),
            NONE,
            self.ast.alloc_function_body(
                span(&node.range),
                self.ast.vec(),
                current_block.statements,
            ),
        ))
    }
}
