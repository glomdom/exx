// src/ast.rs

#[allow(dead_code)]
#[derive(Debug)]
pub enum Stmt {
    VariableDecl {
        is_mutable: bool,
        name: String,
        type_annotation: Option<Type>,
        initializer: Option<Expr>,
    },

    FunctionDecl {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },

    ClassDecl {
        name: String,
        fields: Vec<Stmt>,  // e.g. variable declarations
        methods: Vec<Stmt>, // function declarations
    },

    ModuleDecl {
        name: String,
        declarations: Vec<Stmt>,
    },

    Import(String),
    Expression(Expr),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<Type>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Literal(Literal),
    Identifier(String),
    Grouping(Box<Expr>),

    Block(Vec<Stmt>),
    PropertyAccess {
        object: Box<Expr>,
        name: String,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expr>,
    },
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Type {
    Simple(String),
    Function(Vec<Type>, Box<Type>), // parameter types, then return type

    Generic { name: String, params: Vec<Type> },
}
