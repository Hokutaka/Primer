use crate::ast::{
    BinaryOp, Expr, ExprKind, Item, Program, Stmt, StmtKind, Type, TypeSpec, UnaryOp,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::source::Span;

type ParseResult<T> = Result<T, Diagnostic>;

pub fn parse(tokens: Vec<Token>) -> Result<Program, Diagnostic> {
    Parser { tokens, current: 0 }.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn parse_program(&mut self) -> ParseResult<Program> {
        let mut items = Vec::new();

        while !matches!(&self.peek().kind, TokenKind::Eof) {
            items.push(Item::Statement(self.parse_statement()?));
        }

        Ok(Program { items })
    }

    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        match &self.peek().kind {
            TokenKind::Mut => self.parse_binding(),
            TokenKind::Identifier(_) => match &self.peek_next().kind {
                TokenKind::Colon => self.parse_binding(),
                TokenKind::Equal => self.parse_assignment(),
                other => Err(Diagnostic::new(
                    format!("expected `:` or `=` after identifier, found {other:?}"),
                    self.peek_next().span,
                )),
            },
            TokenKind::Print => self.parse_print(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Break => self.parse_loop_control(StmtKind::Break),
            TokenKind::Continue => self.parse_loop_control(StmtKind::Continue),
            other => Err(self.error(format!("expected statement, found {other:?}"))),
        }
    }

    fn parse_binding(&mut self) -> ParseResult<Stmt> {
        let (mutable, start) = if matches!(&self.peek().kind, TokenKind::Mut) {
            (true, self.advance().span.start())
        } else {
            (false, self.peek().span.start())
        };

        let token = self.advance().clone();

        let name = match token.kind {
            TokenKind::Identifier(name) => name,
            other => {
                return Err(Diagnostic::new(
                    format!("expected identifier, found {other:?}"),
                    token.span,
                ));
            }
        };

        self.expect_simple(TokenKind::Colon)?;
        let type_spec = self.parse_type_spec()?;
        self.expect_simple(TokenKind::Equal)?;
        let value = self.parse_expression()?;
        let semicolon = self.expect_simple(TokenKind::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Binding {
                mutable,
                name,
                type_spec,
                value,
            },
            span: Span::new(start, semicolon.end()),
        })
    }

    fn parse_assignment(&mut self) -> ParseResult<Stmt> {
        let mut statement = self.parse_assignment_clause()?;
        let semicolon = self.expect_simple(TokenKind::Semicolon)?;
        statement.span = Span::new(statement.span.start(), semicolon.end());
        Ok(statement)
    }

    fn parse_assignment_clause(&mut self) -> ParseResult<Stmt> {
        let token = self.advance().clone();
        let start = token.span.start();
        let name_span = token.span;

        let name = match token.kind {
            TokenKind::Identifier(name) => name,
            other => {
                return Err(Diagnostic::new(
                    format!("expected identifier, found {other:?}"),
                    token.span,
                ));
            }
        };

        self.expect_simple(TokenKind::Equal)?;
        let value = self.parse_expression()?;
        let end = value.span.end();

        Ok(Stmt {
            kind: StmtKind::Assignment {
                name,
                name_span,
                value,
            },
            span: Span::new(start, end),
        })
    }

    fn parse_type_spec(&mut self) -> ParseResult<TypeSpec> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Identifier(name) => match name.as_str() {
                "i64" => Ok(TypeSpec::Explicit(Type::I64)),
                "f32" => Ok(TypeSpec::Explicit(Type::F32)),
                "f64" => Ok(TypeSpec::Explicit(Type::F64)),
                "bool" => Ok(TypeSpec::Explicit(Type::Bool)),
                "infer" => Ok(TypeSpec::Infer),

                _ => Err(Diagnostic::new(
                    format!("unknown type `{name}`"),
                    token.span,
                )),
            },

            other => Err(Diagnostic::new(
                format!("expected type, found {other:?}"),
                token.span,
            )),
        }
    }

    fn parse_print(&mut self) -> ParseResult<Stmt> {
        let start = self.advance().span.start();

        self.expect_simple(TokenKind::LeftParen)?;
        let value = self.parse_expression()?;
        self.expect_simple(TokenKind::RightParen)?;
        let semicolon = self.expect_simple(TokenKind::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Print { value },
            span: Span::new(start, semicolon.end()),
        })
    }

    fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_equality()
    }

    fn parse_if(&mut self) -> ParseResult<Stmt> {
        let start = self.advance().span.start();
        let condition = self.parse_expression()?;
        let (then_body, then_end) = self.parse_block()?;

        let (else_body, end) = if matches!(&self.peek().kind, TokenKind::Else) {
            self.advance();
            let (body, end) = self.parse_block()?;
            (body, end)
        } else {
            (Vec::new(), then_end)
        };

        Ok(Stmt {
            kind: StmtKind::If {
                condition,
                then_body,
                else_body,
            },
            span: Span::new(start, end),
        })
    }

    fn parse_while(&mut self) -> ParseResult<Stmt> {
        let start = self.advance().span.start();
        let condition = self.parse_expression()?;
        let (body, end) = self.parse_block()?;

        Ok(Stmt {
            kind: StmtKind::While { condition, body },
            span: Span::new(start, end),
        })
    }

    fn parse_for(&mut self) -> ParseResult<Stmt> {
        let start = self.advance().span.start();
        if !matches!(&self.peek().kind, TokenKind::LeftParen) {
            return Err(Diagnostic::new(
                "expected `(` after `for`",
                self.peek().span,
            ));
        }
        self.advance();

        let initializer = match (&self.peek().kind, &self.peek_next().kind) {
            (TokenKind::Mut, _) | (TokenKind::Identifier(_), TokenKind::Colon) => {
                self.parse_binding()?
            }
            (TokenKind::Identifier(_), TokenKind::Equal) => self.parse_assignment()?,
            _ => {
                return Err(Diagnostic::new(
                    "expected a binding or assignment at the start of `for`",
                    self.peek().span,
                ));
            }
        };

        let condition = self.parse_expression()?;
        self.expect_simple(TokenKind::Semicolon)?;

        let update = match (&self.peek().kind, &self.peek_next().kind) {
            (TokenKind::Identifier(_), TokenKind::Equal) => self.parse_assignment_clause()?,
            _ => {
                return Err(Diagnostic::new(
                    "for update must be an assignment",
                    self.peek().span,
                ));
            }
        };
        if !matches!(&self.peek().kind, TokenKind::RightParen) {
            return Err(Diagnostic::new(
                "expected `)` after the `for` update",
                self.peek().span,
            ));
        }
        self.advance();

        let (body, end) = self.parse_block()?;

        Ok(Stmt {
            kind: StmtKind::For {
                initializer: Box::new(initializer),
                condition,
                update: Box::new(update),
                body,
            },
            span: Span::new(start, end),
        })
    }

    fn parse_loop_control(&mut self, kind: StmtKind) -> ParseResult<Stmt> {
        let start = self.advance().span.start();
        let end = self.expect_simple(TokenKind::Semicolon)?.end();

        Ok(Stmt {
            kind,
            span: Span::new(start, end),
        })
    }

    fn parse_block(&mut self) -> ParseResult<(Vec<Stmt>, usize)> {
        self.expect_simple(TokenKind::LeftBrace)?;
        let mut statements = Vec::new();

        while !matches!(&self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }

        let closing = self.expect_simple(TokenKind::RightBrace)?;
        Ok((statements, closing.end()))
    }

    fn parse_equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match &self.peek().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::BangEqual => BinaryOp::NotEqual,
                _ => break,
            };

            self.advance();

            let right = self.parse_comparison()?;
            let span = Span::new(expr.span.start(), right.span.end());

            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_additive()?;

        loop {
            let op = match &self.peek().kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };

            self.advance();

            let right = self.parse_additive()?;
            let span = Span::new(expr.span.start(), right.span.end());

            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = match &self.peek().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_multiplicative()?;

            let span = Span::new(expr.span.start(), right.span.end());

            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match &self.peek().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                _ => break,
            };

            self.advance();

            let right = self.parse_unary()?;
            let span = Span::new(expr.span.start(), right.span.end());

            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        let op = match &self.peek().kind {
            TokenKind::Minus => Some(UnaryOp::Negate),
            TokenKind::Bang => Some(UnaryOp::Not),
            _ => None,
        };

        if let Some(op) = op {
            let operator_span = self.advance().span;
            let value = self.parse_unary()?;
            let span = Span::new(operator_span.start(), value.span.end());

            return Ok(Expr {
                kind: ExprKind::Unary {
                    op,
                    value: Box::new(value),
                },
                span,
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let token = self.advance().clone();
        let span = token.span;

        match token.kind {
            TokenKind::True => Ok(Expr {
                kind: ExprKind::Boolean(true),
                span,
            }),

            TokenKind::False => Ok(Expr {
                kind: ExprKind::Boolean(false),
                span,
            }),

            TokenKind::Integer(value) => Ok(Expr {
                kind: ExprKind::Integer(value),
                span,
            }),

            TokenKind::Float(text) => Ok(parse_float_literal(text, span)),

            TokenKind::Identifier(name) => Ok(Expr {
                kind: ExprKind::Variable(name),
                span,
            }),

            TokenKind::LeftParen => {
                let expr = self.parse_expression()?;
                let closing_span = self.expect_simple(TokenKind::RightParen)?;
                let span = Span::new(span.start(), closing_span.end());

                Ok(Expr { span, ..expr })
            }

            other => Err(Diagnostic::new(
                format!("expected expression, found {other:?}"),
                token.span,
            )),
        }
    }

    fn expect_simple(&mut self, expected: TokenKind) -> ParseResult<Span> {
        let token = self.advance().clone();

        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new(
                format!("expected {expected:?}, found {:?}", token.kind),
                token.span,
            ))
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> &Token {
        self.tokens
            .get(self.current + 1)
            .unwrap_or_else(|| self.peek())
    }

    fn advance(&mut self) -> &Token {
        let index = self.current;

        if !matches!(&self.tokens[self.current].kind, TokenKind::Eof) {
            self.current += 1;
        }

        &self.tokens[index]
    }

    fn error(&self, message: String) -> Diagnostic {
        Diagnostic::new(message, self.peek().span)
    }
}

fn parse_float_literal(text: String, span: Span) -> Expr {
    let kind = if let Some(value) = text.strip_suffix("f32") {
        ExprKind::Float {
            text: value.to_owned(),
            explicit_type: Some(Type::F32),
        }
    } else if let Some(value) = text.strip_suffix("f64") {
        ExprKind::Float {
            text: value.to_owned(),
            explicit_type: Some(Type::F64),
        }
    } else {
        ExprKind::Float {
            text,
            explicit_type: None,
        }
    };

    Expr { kind, span }
}

#[cfg(test)]
mod tests {
    use crate::ast::{BinaryOp, ExprKind, StmtKind, Type, TypeSpec, UnaryOp};
    use crate::lexer::lex;
    use crate::source::Span;

    use super::parse;

    #[test]
    fn parses_binding() {
        let program = parse(lex("x: i64 = 42;").unwrap()).unwrap();

        let StmtKind::Binding {
            mutable,
            name,
            type_spec,
            value,
        } = &program.statement(0).kind
        else {
            panic!("expected binding");
        };

        assert!(!mutable);
        assert_eq!(name, "x");
        assert_eq!(*type_spec, TypeSpec::Explicit(Type::I64));
        assert_eq!(value.kind, ExprKind::Integer(42));
        assert_eq!(value.span, Span::new(9, 11));
        assert_eq!(program.statement(0).span, Span::new(0, 12));
    }

    #[test]
    fn parses_mutable_binding() {
        let program = parse(lex("mut x: i64 = 1;").unwrap()).unwrap();

        let StmtKind::Binding { mutable, name, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        assert!(*mutable);
        assert_eq!(name, "x");
        assert_eq!(program.statement(0).span, Span::new(0, 15));
    }

    #[test]
    fn parses_assignment() {
        let program = parse(lex("x = x + 1;").unwrap()).unwrap();

        let StmtKind::Assignment {
            name,
            name_span,
            value,
        } = &program.statement(0).kind
        else {
            panic!("expected assignment");
        };

        assert_eq!(name, "x");
        assert_eq!(*name_span, Span::new(0, 1));
        assert_eq!(value.span, Span::new(4, 9));
        assert_eq!(program.statement(0).span, Span::new(0, 10));
    }

    #[test]
    fn parses_infer() {
        let program = parse(lex("x: infer = 1 + 2;").unwrap()).unwrap();

        let StmtKind::Binding { type_spec, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        assert_eq!(*type_spec, TypeSpec::Infer);
    }

    #[test]
    fn parses_float_without_explicit_type() {
        let program = parse(lex("x: f32 = 0.1;").unwrap()).unwrap();

        let StmtKind::Binding { value, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        assert_eq!(
            value.kind,
            ExprKind::Float {
                text: "0.1".into(),
                explicit_type: None
            }
        );
    }

    #[test]
    fn multiplication_has_higher_precedence() {
        let program = parse(lex("x: i64 = 1 + 2 * 3;").unwrap()).unwrap();

        let StmtKind::Binding { value, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        let ExprKind::Binary { op, right, .. } = &value.kind else {
            panic!("expected binary expression");
        };

        assert_eq!(*op, BinaryOp::Add);
        assert_eq!(value.span, Span::new(9, 18));

        assert!(matches!(
            &right.kind,
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                ..
            }
        ));
        assert_eq!(right.span, Span::new(13, 18));
    }

    #[test]
    fn comparison_has_lower_precedence_than_arithmetic() {
        let program = parse(lex("x: bool = 1 + 2 < 4;").unwrap()).unwrap();

        let StmtKind::Binding { value, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        let ExprKind::Binary { op, left, .. } = &value.kind else {
            panic!("expected comparison");
        };

        assert_eq!(*op, BinaryOp::Less);
        assert!(matches!(
            &left.kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn parses_boolean_literal_and_not() {
        let program = parse(lex("x: bool = !false;").unwrap()).unwrap();

        let StmtKind::Binding { value, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        assert!(matches!(
            &value.kind,
            ExprKind::Unary {
                op: UnaryOp::Not,
                ..
            }
        ));
    }

    #[test]
    fn parses_if_else_blocks() {
        let program = parse(lex("if true { print(1); } else { print(2); }").unwrap()).unwrap();

        let StmtKind::If {
            condition,
            then_body,
            else_body,
        } = &program.statement(0).kind
        else {
            panic!("expected if statement");
        };

        assert_eq!(condition.kind, ExprKind::Boolean(true));
        assert_eq!(then_body.len(), 1);
        assert_eq!(else_body.len(), 1);
        assert_eq!(program.statement(0).span, Span::new(0, 40));
    }

    #[test]
    fn parses_while_block() {
        let program = parse(lex("while true { print(1); }").unwrap()).unwrap();

        let StmtKind::While { condition, body } = &program.statement(0).kind else {
            panic!("expected while statement");
        };

        assert_eq!(condition.kind, ExprKind::Boolean(true));
        assert_eq!(body.len(), 1);
        assert_eq!(program.statement(0).span, Span::new(0, 24));
    }

    #[test]
    fn parses_break_and_continue() {
        let program = parse(lex("while true { continue; break; }").unwrap()).unwrap();

        let StmtKind::While { body, .. } = &program.statement(0).kind else {
            panic!("expected while statement");
        };

        assert_eq!(body[0].kind, StmtKind::Continue);
        assert_eq!(body[1].kind, StmtKind::Break);
        assert_eq!(body[0].span, Span::new(13, 22));
        assert_eq!(body[1].span, Span::new(23, 29));
    }

    #[test]
    fn parses_for_block() {
        let source = "for (mut i: i64 = 0; i < 3; i = i + 1) { print(i); }";
        let program = parse(lex(source).unwrap()).unwrap();

        let StmtKind::For {
            initializer,
            condition,
            update,
            body,
        } = &program.statement(0).kind
        else {
            panic!("expected for statement");
        };

        assert!(matches!(initializer.kind, StmtKind::Binding { .. }));
        assert!(matches!(condition.kind, ExprKind::Binary { .. }));
        assert!(matches!(update.kind, StmtKind::Assignment { .. }));
        assert_eq!(body.len(), 1);
        assert_eq!(program.statement(0).span, Span::new(0, source.len()));
    }

    #[test]
    fn parses_for_block_with_assignment_as_start() {
        let source = "mut i: i64 = 3; for (i = 0; i < 3; i = i + 1) { print(i); }";
        let program = parse(lex(source).unwrap()).unwrap();

        let StmtKind::For { initializer, .. } = &program.statement(1).kind else {
            panic!("expected for statement");
        };

        assert!(matches!(initializer.kind, StmtKind::Assignment { .. }));
    }

    #[test]
    fn requires_parentheses_around_for_header() {
        let error =
            parse(lex("for mut i: i64 = 0; i < 3; i = i + 1 { print(i); }").unwrap()).unwrap_err();

        assert_eq!(error.message(), "expected `(` after `for`");
    }

    #[test]
    fn reports_missing_closing_parenthesis_in_for_header() {
        let error =
            parse(lex("for (mut i: i64 = 0; i < 3; i = i + 1 { print(i); }").unwrap()).unwrap_err();

        assert_eq!(error.message(), "expected `)` after the `for` update");
    }

    #[test]
    fn unary_span_includes_operator() {
        let program = parse(lex("x: i64 = -1;").unwrap()).unwrap();

        let StmtKind::Binding { value, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        assert_eq!(value.span, Span::new(9, 11));
    }

    #[test]
    fn parenthesized_span_includes_parentheses() {
        let program = parse(lex("x: i64 = (1 + 2);").unwrap()).unwrap();

        let StmtKind::Binding { value, .. } = &program.statement(0).kind else {
            panic!("expected binding");
        };

        assert_eq!(value.span, Span::new(9, 16));
    }

    #[test]
    fn reports_missing_semicolon_at_end_of_input() {
        let error = parse(lex("x: i64 = 1").unwrap()).unwrap_err();

        assert_eq!(error.message(), "expected Semicolon, found Eof");
        assert_eq!(error.primary_span(), Some(Span::empty(10)));
    }

    #[test]
    fn print_span_covers_keyword_through_semicolon() {
        let program = parse(lex("  print(1);").unwrap()).unwrap();
        let statement = program.statement(0);

        let StmtKind::Print { value } = &statement.kind else {
            panic!("expected print");
        };

        assert_eq!(statement.span, Span::new(2, 11));
        assert_eq!(value.span, Span::new(8, 9));
    }
}
