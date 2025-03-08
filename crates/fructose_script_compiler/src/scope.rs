use std::{collections::HashMap, ops::Range};

use fructose_script_parser::{Visit, ast};

#[derive(Debug)]
pub struct Location {
    range: Range<usize>,
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub definition: Location,
    pub references: Vec<Location>,
}

#[derive(Debug, Clone, Copy)]
struct ScopeId(usize);

struct Scope {
    id: ScopeId,
    parent: Option<ScopeId>,
    entries: HashMap<String, Variable>,
}

impl Scope {
    fn define(&mut self, name: String, definition: Location) {
        let variable = Variable {
            name: name.clone(),
            definition,
            references: Vec::new(),
        };
        self.entries.insert(name, variable);
    }
}

#[derive(Debug)]
enum ScopeBuilderError {
    NotDefined(String, Range<usize>),
}

pub struct ScopeBuilder {
    scopes: Vec<Scope>,
    current_scope_id: ScopeId,

    errors: Vec<ScopeBuilderError>,
}

impl ScopeBuilder {
    #[inline]
    pub fn new() -> Self {
        let id = ScopeId(0);
        let root_scope = Scope {
            id,
            parent: None,
            entries: HashMap::new(),
        };
        Self {
            scopes: vec![root_scope],
            current_scope_id: id,
            errors: Vec::new(),
        }
    }

    fn child(&mut self, parent_scope_id: ScopeId) -> &mut Scope {
        let id = ScopeId(self.scopes.len());
        let child_scope = Scope {
            id,
            parent: Some(parent_scope_id),
            entries: HashMap::new(),
        };
        self.scopes.push(child_scope);
        &mut self.scopes[id.0]
    }

    fn reference(&mut self, name: &str) -> Option<&mut Variable> {
        let mut scope_id = self.current_scope_id;
        loop {
            let scope = &self.scopes[scope_id.0];

            if scope.entries.contains_key(name) {
                // loop直下で&mutでscopeを取ってくるとborrow checkerがケチをつけてくる。
                let scope = &mut self.scopes[scope_id.0];
                return scope.entries.get_mut(name);
            }

            scope_id = scope.parent?;
        }
    }

    #[inline]
    fn current_scope(&mut self) -> &mut Scope {
        &mut self.scopes[self.current_scope_id.0]
    }
}

impl Visit for ScopeBuilder {
    type Result = ();

    fn visit_let(&mut self, node: &ast::Let) {
        self.visit_expression(&node.init);
        let _var = self.current_scope().define(
            node.name.value.clone(),
            Location {
                range: node.range.clone(),
            },
        );

        // TODO: このnodeからvarを辿れるようにする。
        // node.var = var;
    }

    fn visit_fn(&mut self, node: &ast::Fn) {
        self.enter_scope();

        for parameter in &node.parameters {
            let _var = self.current_scope().define(
                parameter.value.clone(),
                Location {
                    range: parameter.range.clone(),
                },
            );

            // TODO: このnodeからvarを辿れるようにする。
            // argument.var = var;
        }

        self.visit_expression(&node.body);

        self.leave_scope();
    }

    fn visit_ident(&mut self, node: &ast::Ident) {
        if let Some(var) = self.reference(&node.value) {
            var.references.push(Location {
                range: node.range.clone(),
            });

            // TODO: このnodeからvarを辿れるようにする。
            // node.var = var;
        } else {
            self.errors.push(ScopeBuilderError::NotDefined(
                node.value.clone(),
                node.range.clone(),
            ));
        }
    }

    fn enter_scope(&mut self) {
        let child_scope = self.child(self.current_scope_id);
        self.current_scope_id = child_scope.id;
    }

    fn leave_scope(&mut self) {
        let parent_scope_id = self.current_scope().parent;
        self.current_scope_id =
            parent_scope_id.expect("enter_scopeよりleave_scopeの方が呼ばれる回数が多いぜ！");
    }
}
