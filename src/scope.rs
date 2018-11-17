use error::{ImmutableAssignError, UndeclaredVariableError, UnexpectedScopePopError};
use value::{Atom, TypeID, ValueID, ValueManagerRef};

use std::collections::HashMap;
use std::rc::Rc;

use failure::Error;

use inkwell::values::PointerValue;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub enum BindingKind {
    Mutable,
    Immutable
}

struct BoundAtom {
    kind: BindingKind,
    atom: Atom
}

pub struct Scope {
    variable_pointers: HashMap<String, PointerValue>,
    manager: ValueManagerRef,
    bindings: HashMap<String, BoundAtom>,
}

impl Scope {
    pub fn new(manager: ValueManagerRef) -> Self {
        Scope {
            variable_pointers: HashMap::new(),
            bindings: HashMap::new(),
            manager,
        }
    }

    pub fn get(&self, s: &str) -> Option<Atom> {
        self.bindings
            .get(s)
            .map(|b| b.atom.clone())
    }

    pub fn get_var(&self, s: &str) -> Option<PointerValue> {
        self.variable_pointers
            .get(s)
            .cloned()
    }

    pub fn assign(&mut self, s: &str, atom: Atom) -> Result<(), Error> {
        let new_entry = self
            .bindings
            .get(s)
            .map(|v| {
                if v.kind == BindingKind::Immutable {
                    Err(ImmutableAssignError)
                } else {
                    Ok(BoundAtom {
                        kind: v.kind.clone(),
                        atom,
                    })
                }
            }).ok_or(UndeclaredVariableError)??;
        self.bindings.insert(s.to_string(), new_entry);
        Ok(())
    }

    pub fn bind(&mut self, s: &str, atom: Atom, kind: BindingKind) {
        self.bindings.insert(s.to_string(), BoundAtom { kind, atom });
    }

    pub fn add_var(&mut self, s: &str, var: PointerValue) {
        self.variable_pointers.insert(s.to_string(), var);
    }

    pub fn variables(&self) -> impl Iterator<Item = (&String, PointerValue)> {
        self.variable_pointers
            .iter()
            .map(|(k, v)| (k, v.clone()))
    }

    pub fn bindings(&self) -> impl Iterator<Item = (&String, Atom)> {
        self.bindings
            .iter()
            .map(|(k, v)| (k, v.atom.clone()))
    }
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
    manager: ValueManagerRef,
}

impl ScopeStack {
    pub fn new(manager: ValueManagerRef) -> Self {
        ScopeStack {
            scopes: vec![Scope::new(manager.clone())],
            manager,
        }
    }

    pub fn new_scope(&self) -> Scope {
        Scope::new(self.manager.clone())
    }

    pub fn push(&mut self, sc: Scope) {
        self.scopes.push(sc)
    }

    pub fn pop(&mut self) -> Result<Scope, Error> {
        if self.scopes.len() == 1 {
            return Err(UnexpectedScopePopError.into());
        }
        self.scopes.pop().ok_or(UnexpectedScopePopError.into())
    }

    pub fn variables(&self) -> impl Iterator<Item = (&String, PointerValue)> {
        self.scopes.iter().rev().flat_map(|it| it.variables())
    }

    pub fn bindings(&self) -> impl Iterator<Item = (&String, Atom)> {
        self.scopes.iter().rev().flat_map(|it| it.bindings())
    }

    pub fn add_var(&mut self, s: &str, var: PointerValue) {
        self.scopes.last_mut().unwrap().add_var(s, var)
    }

    pub fn get(&self, s: &str) -> Option<Atom> {
        self.bindings().find(|(k, _)| k == &s).map(|(_, v)| v)
    }

    pub fn get_var(&self, s: &str) -> Option<PointerValue> {
        self.variables().find(|(k, _)| k == &s).map(|(_, v)| v)
    }

    pub fn assign(&mut self, s: &str, val: Atom) -> Result<(), Error> {
        self.scopes
            .iter_mut()
            .rev()
            .find(|sc| sc.get(s).is_some())
            .ok_or(UndeclaredVariableError.into())
            .and_then(|v| v.assign(s, val))
    }

    pub fn bind(&mut self, s: &str, val: Atom, kind: BindingKind) {
        self.scopes.last_mut().unwrap().bind(s, val, kind);
    }

    pub fn unique_name(&self, s: &str) -> String {
        let num_vars = self.variables().count();
        format!("{}.{}", s, num_vars)
    }
}
