use self::{
    def_const::def_const,
    def_fun::{def_fun, DefFun},
    def_struct::def_struct,
    def_type::def_type,
    def_union::def_union,
    def_var::{parse_def_vars, DefVars},
};
use super::{
    expr::ExprNode,
    type_::{parse_type_node, TypeNode},
    Node, NodeError,
};
use crate::Rule;
use pest::iterators::Pair;

pub mod def_const;
pub mod def_fun;
pub mod def_struct;
pub mod def_type;
pub mod def_union;
pub mod def_var;

#[derive(Debug, Clone)]
pub enum DefNode {
    Vars(DefVars),
    Fun(DefFun),
    Const {
        _type: TypeNode,
        name: String,
        expr: ExprNode,
    },
    Type {
        _type: TypeNode,
        ident: String,
    },
    Struct {
        name: String,
        member_list: Vec<Member>,
    },
    Union {
        name: String,
        member_list: Vec<Member>,
    },
}

#[derive(Debug, Clone)]
pub struct Member {
    pub _type: TypeNode,
    pub name: String,
}

pub fn parse_member_list(pair: Pair<Rule>) -> Result<Vec<Member>, NodeError> {
    let mut pairs = pair.into_inner();
    let mut member_list = vec![];

    while let Some(pair) = pairs.next() {
        let mut pairs = pair.into_inner();
        let _type = parse_type_node(pairs.next().unwrap())?;
        let name = pairs.next().unwrap().as_str().into();
        member_list.push(Member { _type, name });
    }

    Ok(member_list)
}

#[test]
pub fn test_member_list() {
    use crate::CBCScanner;
    use pest::Parser;
    assert_eq!(
        parse_member_list(
            CBCScanner::parse(Rule::MEMBER_LIST, "{ int x; int y; }",)
                .unwrap()
                .next()
                .unwrap()
        )
        .unwrap()
        .len(),
        2
    );
}

pub fn parse_topdef_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let mut pairs = pair.into_inner().peekable();

    let def_node = match pairs.peek().unwrap().as_rule() {
        Rule::DEF_VARS => DefNode::Vars(parse_def_vars(pairs.next().unwrap())?),
        Rule::DEF_FUNCTION => def_fun(pairs.next().unwrap())?,
        Rule::DEF_CONST => def_const(pairs.next().unwrap())?,
        Rule::DEF_TYPE => def_type(pairs.next().unwrap())?,
        Rule::DEF_STRUCT => def_struct(pairs.next().unwrap())?,
        Rule::DEF_UNION => def_union(pairs.next().unwrap())?,
        e => panic!("{:?}", e),
    };

    Ok(Node::Def(Box::new(def_node)))
}
