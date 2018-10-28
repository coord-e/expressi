use value::{ValueID, ValueManager, TypeID};
use error::UnexpectedScopePopError;

use std::collections::HashMap;
use std::rc::Rc;

use failure::Error;

use inkwell::values::PointerValue;

type VariableId = usize;

pub struct Scope {
    variables: HashMap<String, VariableId>,
    variable_values: HashMap<VariableId, ValueID>,
    variable_pointers: HashMap<VariableId, PointerValue>,
    manager: Rc<ValueManager>
}

impl Scope {
    pub fn new(manager: Rc<ValueManager>) -> Self {
        Scope {
            variables: HashMap::new(),
            variable_values: HashMap::new(),
            variable_pointers: HashMap::new(),
            manager
        }
    }

    pub fn get(&self, s: &str) -> Option<ValueID> {
        self.variables.get(s).and_then(|var| self.variable_values.get(var)).cloned()
    }

    pub fn get_var(&self, s: &str) -> Option<PointerValue> {
        self.variables.get(s).and_then(|var| self.variable_pointers.get(var)).cloned()
    }

    pub fn set(&mut self, s: &str, val: ValueID) {
        self.variable_values.insert(*self.variables.get(s).unwrap(), val);
    }

    pub fn add(&mut self, s: &str, val: ValueID, var: PointerValue) {
        let idx = self.variables.len();
        self.variables.insert(s.to_string(), idx);
        self.variable_values.insert(idx, val);
        self.variable_pointers.insert(idx, var);
    }

    pub fn variables(&self) -> impl Iterator<Item=(&String, PointerValue)> {
        self.variables.iter().map(move |(k, v)| (k, self.variable_pointers.get(&v).cloned().unwrap()))
    }

    pub fn values(&self) -> impl Iterator<Item=(&String, &ValueID)> {
        self.variables.iter().map(move |(k, v)| (k, self.variable_values.get(&v).unwrap()))
    }

    pub fn types(&self) -> impl Iterator<Item=(&String, &TypeID)> {
        self.types.iter()
    }

    pub fn resolve_type(&self, id: &str) -> Option<TypeID> {
        self.types.get(id).cloned()
    }
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
    manager: Rc<ValueManager>
}

impl ScopeStack {
    pub fn new(manager: Rc<ValueManager>) -> Self {
        ScopeStack {
            scopes: vec![Scope::new(manager.clone())],
            manager
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

    pub fn variables(&self) -> impl Iterator<Item=(&String, PointerValue)> {
        self.scopes.iter().flat_map(|it| it.variables())
    }

    pub fn values(&self) -> impl Iterator<Item=(&String, &ValueID)> {
        self.scopes.iter().flat_map(|it| it.values())
    }

    pub fn types(&self) -> impl Iterator<Item=(&String, &TypeID)> {
        self.scopes.iter().flat_map(|it| it.types())
    }

    pub fn add(&mut self, s: &str, val: ValueID, var: PointerValue) {
        self.scopes.last_mut().unwrap().add(s, val, var)
    }

    pub fn get(&self, s: &str) -> Option<&ValueID> {
        self.values().find(|(k, _)| k == &s).map(|(_, v)| v)
    }

    pub fn get_var(&self, s: &str) -> Option<PointerValue> {
        self.variables().find(|(k, _)| k == &s).map(|(_, v)| v)
    }

    pub fn set(&mut self, s: &str, val: ValueID) {
        let mut it = self.scopes.iter_mut();
        it.find(|sc| sc.get(s).is_some()).or(it.last()).unwrap().set(s, val)
    }

    pub fn unique_name(&self, s: &str) -> String {
        let num_vars = self.variables().count();
        format!("{}.{}", s, num_vars)
    }

    pub fn resolve_type(&self, id: &str) -> Option<TypeID> {
        self.types().find(|(k, _)| k == &id).map(|(_, v)| v).cloned()
    }
}
