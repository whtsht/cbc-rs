use super::*;

#[derive(Debug)]
pub enum ParamsNode {
    Void,
    Some { fixed: Vec<Param>, variable: bool },
}

#[derive(Debug)]
pub struct Param {
    _type: TypeNode,
    name: String,
}
