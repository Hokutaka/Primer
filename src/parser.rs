use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, TypeSpec, UnaryOp};
use crate::lexer::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    Parser { tokens, current: 0 }.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while !matches!(&self.peek().kind, TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match &self.peek().kind {
            TokenKind::Identifier(_) => self.parse_binding(),
            TokenKind::Print => self.parse_print(),
            other => Err(self.error(format!("expected statement, found {other:?}"))),
        }
    }

    fn parse_binding(&mut self) -> Result<Stmt, String> {
        let token = self.advance().clone();

        let name = match token.kind {
            TokenKind::Identifier(name) => name,
            other => {
                return Err(format!(
                    "expected identifier, found {other:?} at byte {}",
                    token.offset
                ));
            }
        };

        self.expect_simple(TokenKind::Colon)?;

        let type_spec = self.parse_type_spec()?;

        self.expect_simple(TokenKind::Equal)?;

        let value = self.parse_expression()?;

        self.expect_simple(TokenKind::Semicolon)?;

        Ok(Stmt::Binding {
            name,
            type_spec,
            value,
        })
    }

    fn parse_type_spec(&mut self) -> Result<TypeSpec, String> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Identifier(name) => match name.as_str() {
                "i64" => Ok(TypeSpec::Explicit(Type::I64)),
                "f32" => Ok(TypeSpec::Explicit(Type::F32)),
                "f64" => Ok(TypeSpec::Explicit(Type::F64)),
                "infer" => Ok(TypeSpec::Infer),

                _ => Err(format!("unknown type `{name}` at byte {}", token.offset)),
            },

            other => Err(format!(
                "expected type, found {other:?} at byte {}",
                token.offset
            )),
        }
    }

    fn parse_print(&mut self) -> Result<Stmt, String> {
        self.advance();

        self.expect_simple(TokenKind::LeftParen)?;

        let value = self.parse_expression()?;

        self.expect_simple(TokenKind::RightParen)?;
        self.expect_simple(TokenKind::Semicolon)?;

        Ok(Stmt::Print { value })
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = match &self.peek().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_multiplicative()?;

            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match &self.peek().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                _ => break,
            };

            self.advance();

            let right = self.parse_unary()?;

            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(&self.peek().kind, TokenKind::Minus) {
            self.advance();

            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                value: Box::new(self.parse_unary()?),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Integer(value) => Ok(Expr::Integer(value)),

            TokenKind::Float(text) => Ok(parse_float_literal(text)),

            TokenKind::Identifier(name) => Ok(Expr::Variable(name)),

            TokenKind::LeftParen => {
                let expr = self.parse_expression()?;

                self.expect_simple(TokenKind::RightParen)?;

                Ok(expr)
            }

            other => Err(format!(
                "expected expression, found {other:?} at byte {}",
                token.offset
            )),
        }
    }

    fn expect_simple(&mut self, expected: TokenKind) -> Result<(), String> {
        let token = self.advance().clone();

        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) {
            Ok(())
        } else {
            Err(format!(
                "expected {expected:?}, found {:?} at byte {}",
                token.kind, token.offset
            ))
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        let index = self.current;

        if !matches!(&self.tokens[self.current].kind, TokenKind::Eof) {
            self.current += 1;
        }

        &self.tokens[index]
    }

    fn error(&self, message: String) -> String {
        format!("{message} at byte {}", self.peek().offset)
    }
}

fn parse_float_literal(text: String) -> Expr {
    if let Some(value) = text.strip_suffix("f32") {
        Expr::Float {
            text: value.to_owned(),
            explicit_type: Some(Type::F32),
        }
    } else if let Some(value) = text.strip_suffix("f64") {
        Expr::Float {
            text: value.to_owned(),
            explicit_type: Some(Type::F64),
        }
    } else {
        Expr::Float {
            text,
            explicit_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{BinaryOp, Expr, Stmt, Type, TypeSpec};
    use crate::lexer::lex;

    use super::parse;

    #[test]
    fn parses_binding() {
        let program = parse(lex("x: i64 = 42;").unwrap()).unwrap();

        let Stmt::Binding {
            name,
            type_spec,
            value,
        } = &program.statements[0]
        else {
            panic!("expected binding");
        };

        assert_eq!(name, "x");
        assert_eq!(*type_spec, TypeSpec::Explicit(Type::I64));
        assert_eq!(*value, Expr::Integer(42));
    }

    #[test]
    fn parses_infer() {
        let program = parse(lex("x: infer = 1 + 2;").unwrap()).unwrap();

        let Stmt::Binding { type_spec, .. } = &program.statements[0] else {
            panic!("expected binding");
        };

        assert_eq!(*type_spec, TypeSpec::Infer);
    }

    #[test]
    fn parses_float_without_explicit_type() {
        let program = parse(lex("x: f32 = 0.1;").unwrap()).unwrap();

        let Stmt::Binding { value, .. } = &program.statements[0] else {
            panic!("expected binding");
        };

        assert_eq!(
            *value,
            Expr::Float {
                text: "0.1".into(),
                explicit_type: None,
            }
        );
    }

    #[test]
    fn multiplication_has_higher_precedence() {
        let program = parse(lex("x: i64 = 1 + 2 * 3;").unwrap()).unwrap();

        let Stmt::Binding { value, .. } = &program.statements[0] else {
            panic!("expected binding");
        };

        let Expr::Binary { op, right, .. } = value else {
            panic!("expected binary expression");
        };

        assert_eq!(*op, BinaryOp::Add);

        assert!(matches!(
            **right,
            Expr::Binary {
                op: BinaryOp::Multiply,
                ..
            }
        ));
    }
}
