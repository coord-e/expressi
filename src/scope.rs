use value::Value;

use std::collections::HashMap;

use cranelift::prelude::{Variable, EntityRef};

pub struct Scope {
    variables: HashMap<String, usize>,
    variable_values: HashMap<usize, Value>
}

impl Default for Scope {
    fn default() -> Scope {
        Scope::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            variable_values: HashMap::new()
        }
    }

    pub fn get(&self, s: &str) -> Option<Value> {
        self.variables.get(s).and_then(|var| self.variable_values.get(var)).cloned()
    }

    pub fn get_var(&self, s: &str) -> Option<Variable> {
        self.variables.get(s).map(|v| Variable::with_u32(*v as u32))
    }

    pub fn set(&mut self, s: &str, val: Value) {
        self.variable_values.insert(*self.variables.get(s).unwrap(), val);
    }

    pub fn add(&mut self, s: &str, val: Value, var: Variable) {
        let idx = var.index();
        self.variables.insert(s.to_string(), idx);
        self.variable_values.insert(idx, val);
    }

    pub fn variables(&self) -> impl Iterator<Item=(&String, Variable)> {
        self.variables.iter().map(|(k, v)| (k, Variable::with_u32(*v as u32)))
    }

    pub fn values(&self) -> impl Iterator<Item=(&String, &Value)> {
        self.variables.iter().map(move |(k, v)| (k, self.variable_values.get(&v).unwrap()))
    }
}

pub struct ScopeStack {
    scopes: Vec<Scope>
}

impl ScopeStack {
    pub fn new() -> Self {
        ScopeStack {
            scopes: vec![Scope::default()]
        }
    }

    pub fn push(&mut self, sc: Scope) {
        self.scopes.push(sc)
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    pub fn variables(&self) -> impl Iterator<Item=(&String, Variable)> {
        self.scopes.iter().flat_map(|it| it.variables())
    }

    pub fn values(&self) -> impl Iterator<Item=(&String, &Value)> {
        self.scopes.iter().flat_map(|it| it.values())
    }

    pub fn add(&mut self, s: &str, val: Value, var: Variable) {
        self.scopes.last_mut().unwrap().add(s, val, var)
    }

    pub fn get(&self, s: &str) -> Option<&Value> {
        self.values().find(|(k, _)| k == &s).map(|(_, v)| v)
    }

    pub fn get_var(&self, s: &str) -> Option<Variable> {
        self.variables().find(|(k, _)| k == &s).map(|(_, v)| v)
    }

    pub fn set(&mut self, s: &str, val: Value) {
        self.scopes.last_mut().unwrap().set(s, val)
    }
}
