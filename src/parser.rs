use crate::ast::*;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Let,
    Var,
    Fn,
    Rec,
    Class,
    Type,
    Module,
    Import,
    Return,
    If,
    Else,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Semicolon,
    Arrow, // ->
    Dot,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    Equal,

    Identifier(String),
    Number(String),
    String(String),
    Boolean(bool),

    Eof,
}

#[derive(Debug, Clone)]
pub struct ParserToken {
    pub kind: TokenKind,
}

impl fmt::Display for ParserToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::Identifier(s) => write!(f, "identifier({})", s),
            TokenKind::Number(n) => write!(f, "number({})", n),
            TokenKind::String(s) => write!(f, "string({})", s),
            other => write!(f, "{:?}", other),
        }
    }
}

pub struct Parser {
    tokens: Vec<ParserToken>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl Parser {
    pub fn new(tokens: Vec<ParserToken>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            declarations.push(self.declaration()?);
        }

        Ok(declarations)
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenKind::Import]) {
            return self.import_declaration();
        }

        if self.match_token(&[TokenKind::Module]) {
            return self.module_declaration();
        }

        if self.match_token(&[TokenKind::Class]) {
            return self.class_declaration();
        }

        if self.match_token(&[TokenKind::Fn]) {
            return self.function_declaration();
        }

        if self.match_token(&[TokenKind::Return]) {
            let expr = if !self.check(&TokenKind::Semicolon) {
                Some(self.expression()?)
            } else {
                None
            };

            self.consume(TokenKind::Semicolon, "Expected ';' after return statement")?;

            return Ok(Stmt::Return(expr));
        }

        if self.match_token(&[TokenKind::Let, TokenKind::Var]) {
            return self.variable_declaration();
        }

        self.statement()
    }

    fn import_declaration(&mut self) -> Result<Stmt, ParseError> {
        let module_name = self.consume_identifier("Expected module name after 'import'")?;

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after import declaration",
        )?;

        Ok(Stmt::Import(module_name))
    }

    fn module_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expected module name")?;
        self.consume(TokenKind::LeftBrace, "Expected '{' after module name")?;

        let mut declarations = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            declarations.push(self.declaration()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after module body")?;

        Ok(Stmt::ModuleDecl { name, declarations })
    }

    fn class_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expected class name")?;
        self.consume(TokenKind::LeftBrace, "Expected '{' after class name")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.match_token(&[TokenKind::Fn]) {
                methods.push(self.function_declaration()?);
            } else if self.match_token(&[TokenKind::Let, TokenKind::Var]) {
                fields.push(self.variable_declaration()?);
            } else {
                return Err(ParseError {
                    message: "Expected field declaration starting with 'let' or 'var', or a method declaration starting with 'fn'.".into(),
                });
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after class body")?;

        Ok(Stmt::ClassDecl {
            name,
            fields,
            methods,
        })
    }

    fn function_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expected function name")?;
        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_name = self.consume_identifier("Expected parameter name")?;
                let type_annotation = if self.match_token(&[TokenKind::Colon]) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                params.push(Parameter {
                    name: param_name,
                    type_annotation,
                });

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&[TokenKind::Arrow]) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftBrace, "Expected '{' before function body")?;
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.declaration()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after function body")?;

        Ok(Stmt::FunctionDecl {
            name,
            params,
            return_type,
            body,
        })
    }

    fn variable_declaration(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous_token_kind();
        let is_mutable = match keyword {
            TokenKind::Let => false,
            TokenKind::Var => true,

            _ => false,
        };

        let name = self.consume_identifier("Expected variable name")?;
        let type_annotation = if self.match_token(&[TokenKind::Colon]) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.match_token(&[TokenKind::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Stmt::VariableDecl {
            is_mutable,
            name,
            type_annotation,
            initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        self.consume(TokenKind::Semicolon, "Expected ';' after expression")?;

        Ok(Stmt::Expression(expr))
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if self.match_token(&[TokenKind::LeftParen]) {
            let mut params = Vec::new();

            loop {
                params.push(self.parse_type()?);

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }

            self.consume(TokenKind::RightParen, "Expected ')' in function type")?;
            self.consume(TokenKind::Arrow, "Expected '->' in function type")?;

            let return_type = Box::new(self.parse_type()?);

            Ok(Type::Function(params, return_type))
        } else {
            let name = self.consume_identifier("Expected type name")?;

            if self.match_token(&[TokenKind::Less]) {
                let mut params = Vec::new();

                loop {
                    params.push(self.parse_type()?);

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }

                self.consume(TokenKind::Greater, "Expected '>' in generic type")?;

                Ok(Type::Generic { name, params })
            } else {
                Ok(Type::Simple(name))
            }
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        let params = self.parse_parameters()?;

        self.consume(TokenKind::Arrow, "Expected '->' after lambda parameters")?;

        let body = self.expression()?;
        Ok(Expr::Lambda {
            params,
            body: Box::new(body),
        })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_and()?;

        while self.match_token(&[TokenKind::Or]) {
            let op = BinaryOp::Or;
            let right = self.logical_and()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenKind::And]) {
            let op = BinaryOp::And;
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenKind::EqualEqual, TokenKind::NotEqual]) {
            let op = match self.previous_token_kind() {
                TokenKind::EqualEqual => BinaryOp::EqualEqual,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };

            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.addition()?;

        while self.match_token(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let op = match self.previous_token_kind() {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,

                _ => unreachable!(),
            };

            let right = self.addition()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.multiplication()?;

        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = match self.previous_token_kind() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,

                _ => unreachable!(),
            };

            let right = self.multiplication()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = match self.previous_token_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };

            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenKind::Not, TokenKind::Minus]) {
            let op = match self.previous_token_kind() {
                TokenKind::Not => UnaryOp::Not,
                TokenKind::Minus => UnaryOp::Negate,

                _ => unreachable!(),
            };

            let expr = self.unary()?;

            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let mut expr = if self.match_token(&[TokenKind::Number(String::new())]) {
            let token = self.previous().clone();
            if let TokenKind::Number(n) = token.kind {
                let num = n.parse::<f64>().map_err(|_| ParseError {
                    message: format!("Invalid number literal: {}", n),
                })?;
                Expr::Literal(Literal::Number(num))
            } else {
                return Err(ParseError {
                    message: "Expected number literal".into(),
                });
            }
        } else if self.match_token(&[TokenKind::String(String::new())]) {
            let token = self.previous().clone();
            if let TokenKind::String(s) = token.kind {
                Expr::Literal(Literal::String(s))
            } else {
                return Err(ParseError {
                    message: "Expected string literal".into(),
                });
            }
        } else if self.match_token(&[TokenKind::Boolean(true)]) {
            Expr::Literal(Literal::Boolean(true))
        } else if self.match_token(&[TokenKind::Boolean(false)]) {
            Expr::Literal(Literal::Boolean(false))
        } else if self.match_token(&[TokenKind::Identifier(String::new())]) {
            let token = self.previous().clone();

            if let TokenKind::Identifier(name) = token.kind {
                if self.check(&TokenKind::LeftBrace) {
                    self.advance();

                    let mut fields = Vec::new();

                    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                        let field_name =
                            self.consume_identifier("Expected field name in struct literal")?;

                        fields.push((field_name.clone(), Expr::Identifier(field_name)));

                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                    self.consume(TokenKind::RightBrace, "Expected '}' after struct literal")?;
                    return Ok(Expr::StructLiteral { name, fields });
                } else {
                    return Ok(Expr::Identifier(name));
                }
            } else {
                return Err(ParseError {
                    message: "Expected identifier".into(),
                });
            }
        } else if self.match_token(&[TokenKind::LeftParen]) {
            if self.lambda_check() {
                self.parse_lambda()?
            } else {
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, "Expected ')' after expression")?;

                Expr::Grouping(Box::new(expr))
            }
        } else if self.match_token(&[TokenKind::LeftBrace]) {
            let mut body = Vec::new();

            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                body.push(self.declaration()?);
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after block")?;

            Expr::Block(body)
        } else {
            return Err(ParseError {
                message: format!("Unexpected token: {:?}", self.peek().kind),
            });
        };

        loop {
            if self.match_token(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenKind::Dot]) {
                expr = self.finish_property_access(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn peek(&self) -> &ParserToken {
        &self.tokens[self.current]
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen, "Expected ')' after arguments")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn finish_property_access(&mut self, object: Expr) -> Result<Expr, ParseError> {
        let name = self.consume_identifier("Expected property name after '.'")?;
        Ok(Expr::PropertyAccess {
            object: Box::new(object),
            name,
        })
    }

    #[allow(dead_code)]
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                let name = self.consume_identifier("Expected parameter name")?;
                let type_annotation = if self.match_token(&[TokenKind::Colon]) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                params.push(Parameter {
                    name,
                    type_annotation,
                });

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        Ok(params)
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();

                return true;
            }
        }

        false
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        match (kind, &self.tokens[self.current].kind) {
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            (TokenKind::Number(_), TokenKind::Number(_)) => true,
            (TokenKind::String(_), TokenKind::String(_)) => true,
            (expected, actual) => expected == actual,
        }
    }

    fn lambda_check(&self) -> bool {
        let mut index = self.current;
        let mut paren_count = 1;

        while index < self.tokens.len() {
            match self.tokens[index].kind {
                TokenKind::LeftParen => paren_count += 1,
                TokenKind::RightParen => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        break;
                    }
                }

                _ => {}
            }

            index += 1;
        }

        if index < self.tokens.len() && self.tokens[index].kind == TokenKind::RightParen {
            if index + 1 < self.tokens.len() && self.tokens[index + 1].kind == TokenKind::Arrow {
                return true;
            }
        }

        false
    }

    fn advance(&mut self) -> &ParserToken {
        if !self.is_at_end() {
            self.current += 1;
        }

        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.current].kind == TokenKind::Eof
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&ParserToken, ParseError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: message.into(),
            })
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if self.is_at_end() {
            return Err(ParseError {
                message: message.into(),
            });
        }

        let token = self.tokens[self.current].clone();

        if let TokenKind::Identifier(name) = token.kind {
            self.advance();

            Ok(name)
        } else {
            Err(ParseError {
                message: message.into(),
            })
        }
    }

    fn previous_token_kind(&self) -> TokenKind {
        self.tokens[self.current - 1].kind.clone()
    }

    fn previous(&self) -> &ParserToken {
        &self.tokens[self.current - 1]
    }
}

impl From<crate::token::Token> for ParserToken {
    fn from(token: crate::token::Token) -> Self {
        use crate::parser::TokenKind::*;
        use crate::tokentype::TokenType;

        let kind = match token.token_type {
            TokenType::Number(n) => Number(n),
            TokenType::String(s) => String(s),
            TokenType::Identifier(s) => Identifier(s),
            TokenType::Boolean(b) => Boolean(b),
            TokenType::Keyword(ref kw) if kw == "let" => Let,
            TokenType::Keyword(ref kw) if kw == "var" => Var,
            TokenType::Keyword(ref kw) if kw == "fn" => Fn,
            TokenType::Keyword(ref kw) if kw == "class" => Class,
            TokenType::Keyword(ref kw) if kw == "return" => Return,
            TokenType::Semicolon => Semicolon,
            TokenType::Colon => Colon,
            TokenType::Arrow => Arrow,
            TokenType::LeftParen => LeftParen,
            TokenType::RightParen => RightParen,
            TokenType::LeftBrace => LeftBrace,
            TokenType::RightBrace => RightBrace,
            TokenType::Plus => Plus,
            TokenType::Minus => Minus,
            TokenType::Star => Star,
            TokenType::Slash => Slash,
            TokenType::Modulo => Percent,
            TokenType::EqualEqual => EqualEqual,
            TokenType::NotEqual => NotEqual,
            TokenType::Less => Less,
            TokenType::LessEqual => LessEqual,
            TokenType::Greater => Greater,
            TokenType::GreaterEqual => GreaterEqual,
            TokenType::And => And,
            TokenType::Or => Or,
            TokenType::Bang => Not,
            TokenType::Equal => Equal,
            TokenType::Dot => Dot,
            TokenType::Comma => Comma,
            TokenType::Eof => Eof,

            _ => Identifier("unknown".into()),
        };

        ParserToken { kind }
    }
}
