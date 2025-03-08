use crate::ast;

pub trait Visit {
    #[inline]
    fn enter_scope(&mut self) {}

    #[inline]
    fn leave_scope(&mut self) {}

    // MARK: Module

    fn visit_module(&mut self, node: &ast::Module) {
        for item in &node.items {
            self.visit_module_item(item);
        }
    }

    fn visit_module_item(&mut self, node: &ast::ModuleItem) {
        use ast::ModuleItem;
        match node {
            ModuleItem::Statement(node) => self.visit_statement(node),
        }
    }

    // MARK: Statement

    fn visit_statement(&mut self, node: &ast::Statement) {
        use ast::Statement;
        match node {
            Statement::Let(node) => self.visit_let(node),
            Statement::Assign(node) => self.visit_assign(node),
            Statement::Expression(node) => {
                self.visit_expression(node);
            }
        }
    }

    fn visit_let(&mut self, node: &ast::Let) {
        self.visit_expression(&node.init);
        self.visit_ident(&node.name);
    }

    fn visit_assign(&mut self, node: &ast::Assign) {
        self.visit_ident(&node.target);
        self.visit_expression(&node.value);
    }

    // MARK: Expression

    type Result: Default;

    fn visit_expression(&mut self, node: &ast::Expression) -> Self::Result {
        use ast::Expression;
        match node {
            Expression::Ident(node) => self.visit_ident(node),
            Expression::NatLiteral(node) => self.visit_nat_literal(node),
            Expression::Block(node) => self.visit_block(node),
            Expression::Fn(node) => self.visit_fn(node),
        }
    }

    #[inline]
    fn visit_ident(&mut self, _node: &ast::Ident) -> Self::Result {
        Default::default()
    }

    #[inline]
    fn visit_nat_literal(&mut self, _node: &ast::NatLiteral) -> Self::Result {
        Default::default()
    }

    fn visit_block(&mut self, node: &ast::Block) -> Self::Result {
        self.enter_scope();

        for statement in &node.statements {
            self.visit_statement(statement);
        }

        let result = if let Some(last) = &node.last {
            self.visit_expression(last)
        } else {
            Default::default()
        };

        self.leave_scope();

        result
    }

    fn visit_fn(&mut self, node: &ast::Fn) -> Self::Result {
        self.enter_scope();

        for argument in &node.parameters {
            self.visit_ident(argument);
        }

        self.visit_expression(&node.body);

        self.leave_scope();

        Default::default()
    }
}
