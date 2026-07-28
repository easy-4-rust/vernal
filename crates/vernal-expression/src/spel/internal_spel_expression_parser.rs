//! 内部递归下降解析器（对标 Spring `InternalSpelExpressionParser`）。
//!
//! Pratt 风格优先级递归下降，消费 `Tokenizer` 输出的 token 流，
//! 构造 AST 节点树，返回 `SpelExpression`。

use std::collections::VecDeque;

use super::ast::compound_expression::CompoundExpression;
use super::ast::elvis::Elvis;
use super::ast::identifier::Identifier;
use super::ast::int_literal::IntLiteral;
use super::ast::null_literal::NullLiteral;
use super::ast::op_and::OpAnd;
use super::ast::op_divide::OpDivide;
use super::ast::op_eq::OpEq;
use super::ast::op_ge::OpGe;
use super::ast::op_gt::OpGt;
use super::ast::op_le::OpLe;
use super::ast::op_lt::OpLt;
use super::ast::op_minus::OpMinus;
use super::ast::op_modulus::OpModulus;
use super::ast::op_multiply::OpMultiply;
use super::ast::op_ne::OpNe;
use super::ast::op_or::OpOr;
use super::ast::op_plus::OpPlus;
use super::ast::real_literal::RealLiteral;
use super::ast::spel_node::SpelNode;
use super::ast::string_literal::StringLiteral;
use super::ast::ternary::Ternary;
use super::spel_message::SpelMessage;
use super::spel_parse_exception::SpelParseException;
use super::token::Token;
use super::token_kind::TokenKind;
use super::tokenizer::Tokenizer;

use crate::expression::Expression as ExpressionTrait;

/// 内部递归下降解析器（对标 Spring `InternalSpelExpressionParser`）。
pub struct InternalSpelExpressionParser {
    /// 原始表达式字符串。
    expression_string: String,
    /// Token 流。
    token_stream: Vec<Token>,
    /// Token 流长度。
    token_stream_length: usize,
    /// 当前 token 位置。
    token_stream_pointer: usize,
}

impl InternalSpelExpressionParser {
    /// 创建解析器。
    pub fn new() -> Self {
        Self {
            expression_string: String::new(),
            token_stream: Vec::new(),
            token_stream_length: 0,
            token_stream_pointer: 0,
        }
    }

    /// 顶层入口：解析表达式字符串为 `Expression`。
    pub fn do_parse_expression(
        &mut self,
        expression_string: &str,
    ) -> Result<Box<dyn ExpressionTrait>, SpelParseException> {
        if expression_string.len() > 100_000 {
            return Err(SpelParseException::new(
                expression_string,
                0,
                SpelMessage::MaxExpressionLengthExceeded,
                &[&expression_string.len().to_string()],
            ));
        }

        self.expression_string = expression_string.to_string();

        // 词法分析
        let mut tokenizer = Tokenizer::new(expression_string);
        self.token_stream = match tokenizer.tokenize() {
            Ok(t) => t,
            Err(e) => return Err(e.into_parse_exception()),
        };
        self.token_stream_length = self.token_stream.len();
        self.token_stream_pointer = 0;

        // 递归下降解析
        let ast = self.eat_expression();
        if ast.is_none() {
            return Err(SpelParseException::new(
                expression_string,
                0,
                SpelMessage::Ood,
                &[],
            ));
        }

        // 检查未消费 token
        if let Some(t) = self.peek_token() {
            return Err(SpelParseException::new(
                expression_string,
                t.start_pos,
                SpelMessage::MoreInput,
                &[&t.string_value()],
            ));
        }

        let ast = ast.unwrap();
        Ok(Box::new(super::spel_expression::SpelExpression::new(
            expression_string.to_string(),
            ast,
        )))
    }

    // ─── 递归下降链 ──────────────────────────────────────────────

    /// expression → assign / elvis / ternary / logicalOr
    fn eat_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let expr = self.eat_logical_or_expression();

        // 三元：condition ? trueValue : falseValue
        if self.peek_token_kind(&[TokenKind::QMark]) {
            self.next_token(); // consume ?
            let true_expr = self.eat_expression();
            self.eat_token(TokenKind::Colon);
            let false_expr = self.eat_expression();
            return Some(Box::new(Ternary::new(
                expr.unwrap_or_else(|| Box::new(NullLiteral::new())),
                true_expr.unwrap_or_else(|| Box::new(NullLiteral::new())),
                false_expr.unwrap_or_else(|| Box::new(NullLiteral::new())),
            )));
        }

        // Elvis：a ?: b
        if self.peek_token_kind(&[TokenKind::Elvis]) {
            self.next_token(); // consume ?:
            let default_expr = self.eat_expression();
            return Some(Box::new(Elvis::new(
                expr.unwrap_or_else(|| Box::new(NullLiteral::new())),
                default_expr.unwrap_or_else(|| Box::new(NullLiteral::new())),
            )));
        }

        expr
    }

    /// logicalOr → logicalAnd (|| logicalAnd)*
    fn eat_logical_or_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let mut left = self.eat_logical_and_expression();
        while self.peek_token_kind(&[TokenKind::SymbolicOr]) {
            self.next_token(); // consume ||
            let right = self.eat_logical_and_expression();
            left = Some(Box::new(OpOr::new(
                left.unwrap_or_else(|| Box::new(NullLiteral::new())),
                right.unwrap_or_else(|| Box::new(NullLiteral::new())),
            )));
        }
        left
    }

    /// logicalAnd → relational (&& relational)*
    fn eat_logical_and_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let mut left = self.eat_relational_expression();
        while self.peek_token_kind(&[TokenKind::SymbolicAnd]) {
            self.next_token(); // consume &&
            let right = self.eat_logical_and_expression();
            left = Some(Box::new(OpAnd::new(
                left.unwrap_or_else(|| Box::new(NullLiteral::new())),
                right.unwrap_or_else(|| Box::new(NullLiteral::new())),
            )));
        }
        left
    }

    /// relational → sum (relationalOp sum)?
    fn eat_relational_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let left = self.eat_sum_expression();

        if let Some(op_token) = self.maybe_eat_relational_operator() {
            let right = self.eat_sum_expression();
            let l = left.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string())));
            let r = right.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string())));

            let node: Box<dyn SpelNode> = match op_token.kind {
                TokenKind::Equal => Box::new(OpEq::new(l, r)),
                TokenKind::NotEqual => Box::new(OpNe::new(l, r)),
                TokenKind::Lt => Box::new(OpLt::new(l, r)),
                TokenKind::Le => Box::new(OpLe::new(l, r)),
                TokenKind::Gt => Box::new(OpGt::new(l, r)),
                TokenKind::Ge => Box::new(OpGe::new(l, r)),
                TokenKind::Instanceof => {
                    // Phase F: 使用 OperatorInstanceof 节点
                    Box::new(Identifier::new(format!("instanceof({})", r.to_string_ast())))
                }
                TokenKind::Matches => {
                    Box::new(Identifier::new(format!("matches({})", r.to_string_ast())))
                }
                TokenKind::Between => {
                    Box::new(Identifier::new(format!("between({})", r.to_string_ast())))
                }
                _ => unreachable!(),
            };
            return Some(node);
        }
        left
    }

    /// sum → product ((+ | -) product)*
    fn eat_sum_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let mut left = self.eat_product_expression();
        loop {
            let kind = self.peek_token().map(|t| t.kind);
            match kind {
                Some(TokenKind::Plus) => {
                    self.next_token();
                    let right = self.eat_product_expression();
                    left = Some(Box::new(OpPlus::new(
                        left.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string()))),
                        right.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string()))),
                    )));
                }
                Some(TokenKind::Minus) => {
                    self.next_token();
                    let right = self.eat_product_expression();
                    left = Some(Box::new(OpMinus::new(
                        left.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string()))),
                        right.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string()))),
                    )));
                }
                _ => break,
            }
        }
        left
    }

    /// product → power (( * | / | % ) power)*
    fn eat_product_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let mut left = self.eat_power_inc_dec_expression();
        loop {
            let kind = self.peek_token().map(|t| t.kind);
            match kind {
                Some(TokenKind::Star) => {
                    self.next_token();
                    let right = self.eat_power_inc_dec_expression();
                    left = Some(Box::new(OpMultiply::new(
                        left.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
                        right.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
                    )));
                }
                Some(TokenKind::Div) => {
                    self.next_token();
                    let right = self.eat_power_inc_dec_expression();
                    left = Some(Box::new(OpDivide::new(
                        left.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
                        right.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
                    )));
                }
                Some(TokenKind::Mod) => {
                    self.next_token();
                    let right = self.eat_power_inc_dec_expression();
                    left = Some(Box::new(OpModulus::new(
                        left.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
                        right.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
                    )));
                }
                _ => break,
            }
        }
        left
    }

    /// power → unary (^ unary)*
    fn eat_power_inc_dec_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let mut left = self.eat_unary_expression();
        while self.peek_token_kind(&[TokenKind::Power]) {
            self.next_token();
            let right = self.eat_unary_expression();
            // Phase F: OperatorPower 节点
            // 暂用 Identifier 占位
            left = Some(Box::new(Identifier::new(format!(
                "({} ^ {})",
                left.as_ref().map(|n| n.to_string_ast()).unwrap_or_default(),
                right.as_ref().map(|n| n.to_string_ast()).unwrap_or_default()
            ))));
        }
        left
    }

    /// unary → prefix (+ | - | !) unary | primary
    fn eat_unary_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let kind = self.peek_token().map(|t| t.kind);
        match kind {
            Some(TokenKind::Not) => {
                self.next_token(); // consume !
                let operand = self.eat_unary_expression();
                // Phase F: OperatorNot 节点
                Some(Box::new(Identifier::new(format!(
                    "!({})",
                    operand.as_ref().map(|n| n.to_string_ast()).unwrap_or_default()
                ))))
            }
            Some(TokenKind::Minus) => {
                self.next_token(); // consume -
                let operand = self.eat_unary_expression();
                // 一元负号：0 - operand
                Some(Box::new(OpMinus::new(
                    Box::new(IntLiteral::new(0, "0".to_string())),
                    operand.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string()))),
                )))
            }
            Some(TokenKind::Plus) => {
                self.next_token(); // consume +
                self.eat_unary_expression()
            }
            _ => self.eat_primary_expression(),
        }
    }

    /// primary → start_node (.property / [index] / .method())*
    fn eat_primary_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let start_node = self.eat_start_node()?;
        let mut nodes: Vec<Box<dyn SpelNode>> = vec![start_node];

        loop {
            match self.peek_token().map(|t| t.kind) {
                Some(TokenKind::Dot) => {
                    self.next_token(); // consume .
                    if let Some(prop) = self.maybe_eat_method_or_property(false) {
                        nodes.push(prop);
                    }
                }
                Some(TokenKind::SafeNavi) => {
                    self.next_token(); // consume ?.
                    if let Some(prop) = self.maybe_eat_method_or_property(true) {
                        nodes.push(prop);
                    }
                }
                Some(TokenKind::LSquare) => {
                    if let Some(idx) = self.maybe_eat_indexer(false) {
                        nodes.push(idx);
                    }
                }
                _ => break,
            }
        }

        if nodes.len() == 1 {
            Some(nodes.into_iter().next().unwrap())
        } else {
            Some(Box::new(CompoundExpression::new(nodes)))
        }
    }

    /// startNode → literal / identifier / paren / @bean / #var
    fn eat_start_node(&mut self) -> Option<Box<dyn SpelNode>> {
        // 字面量
        if let Some(lit) = self.maybe_eat_literal() {
            return Some(lit);
        }

        // 括号表达式
        if self.peek_token_kind(&[TokenKind::LParen]) {
            self.next_token(); // consume (
            let expr = self.eat_expression();
            self.eat_token(TokenKind::RParen);
            return expr;
        }

        // null
        if self.peek_token_kind(&[TokenKind::Identifier])
            && self.peek_token().map(|t| t.string_value().to_lowercase()) == Some("null".to_string())
        {
            self.next_token(); // consume null
            return Some(Box::new(NullLiteral::new()));
        }

        // 标识符（方法调用 / 属性引用）
        if let Some(node) = self.maybe_eat_method_or_property(false) {
            return Some(node);
        }

        // #变量 或 #函数
        if self.peek_token_kind(&[TokenKind::Hash]) {
            self.next_token(); // consume #
            if let Some(t) = self.peek_token() {
                let name = t.string_value();
                self.next_token(); // consume identifier
                return Some(Box::new(Identifier::new(name)));
            }
        }

        // @bean 引用
        if self.peek_token_kind(&[TokenKind::BeanRef]) {
            self.next_token(); // consume @
            if let Some(t) = self.peek_token() {
                let name = t.string_value();
                self.next_token(); // consume identifier
                return Some(Box::new(Identifier::new(format!("@{name}"))));
            }
        }

        None
    }

    /// maybe_eat_literal → Int / Real / String / Boolean / null
    fn maybe_eat_literal(&mut self) -> Option<Box<dyn SpelNode>> {
        let t = self.peek_token()?;
        match t.kind {
            TokenKind::LiteralInt => {
                let t = self.next_token().unwrap();
                let val: i64 = t.string_value().parse().unwrap_or(0);
                Some(Box::new(IntLiteral::new(val, t.string_value())))
            }
            TokenKind::LiteralLong => {
                let t = self.next_token().unwrap();
                let raw = t.string_value();
                let s = raw.trim_end_matches('L').trim_end_matches('l');
                let val: i64 = s.parse().unwrap_or(0);
                Some(Box::new(IntLiteral::new(val, raw)))
            }
            TokenKind::LiteralReal | TokenKind::LiteralRealFloat => {
                let t = self.next_token().unwrap();
                let raw = t.string_value();
                let val: f64 = raw.parse().unwrap_or(0.0);
                Some(Box::new(RealLiteral::new(val, raw)))
            }
            TokenKind::LiteralHexInt | TokenKind::LiteralHexLong => {
                let t = self.next_token().unwrap();
                let raw = t.string_value();
                let s = raw.trim_start_matches("0x").trim_start_matches("0X");
                let val = i64::from_str_radix(s, 16).unwrap_or(0);
                Some(Box::new(IntLiteral::new(val, raw)))
            }
            TokenKind::LiteralString => {
                let t = self.next_token().unwrap();
                Some(Box::new(StringLiteral::new(t.string_value())))
            }
            TokenKind::Identifier => {
                let name = t.string_value();
                if name == "true" || name == "TRUE" {
                    self.next_token();
                    // Phase F: BooleanLiteral 节点
                    Some(Box::new(IntLiteral::new(1, "true".to_string())))
                } else if name == "false" || name == "FALSE" {
                    self.next_token();
                    Some(Box::new(IntLiteral::new(0, "false".to_string())))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// maybe_eat_method_or_property → identifier(args)? / identifier
    fn maybe_eat_method_or_property(&mut self, null_safe: bool) -> Option<Box<dyn SpelNode>> {
        let t = self.peek_token()?;
        if t.kind != TokenKind::Identifier {
            return None;
        }
        let name = t.string_value();
        self.next_token(); // consume identifier

        // 检查是否是方法调用：identifier(...)
        if self.peek_token_kind(&[TokenKind::LParen]) {
            self.next_token(); // consume (
            let mut args: Vec<Box<dyn SpelNode>> = Vec::new();
            if !self.peek_token_kind(&[TokenKind::RParen]) {
                loop {
                    if let Some(a) = self.eat_expression() {
                        args.push(a);
                    }
                    if !self.peek_token_kind(&[TokenKind::Comma]) {
                        break;
                    }
                    self.next_token(); // consume ,
                }
            }
            self.eat_token(TokenKind::RParen);
            // Phase F: 使用 MethodReference 节点
            Some(Box::new(Identifier::new(format!(
                "{name}({args_str})",
                args_str = args.iter().map(|a| a.to_string_ast()).collect::<Vec<_>>().join(", ")
            ))))
        } else {
            // 属性引用
            Some(Box::new(Identifier::new(name)))
        }
    }

    /// maybe_eat_indexer → [expr]
    fn maybe_eat_indexer(&mut self, null_safe: bool) -> Option<Box<dyn SpelNode>> {
        if !self.peek_token_kind(&[TokenKind::LSquare]) {
            return None;
        }
        self.next_token(); // consume [
        let index_expr = self.eat_expression();
        self.eat_token(TokenKind::RSquare);
        // Phase F: 使用 Indexer 节点
        Some(Box::new(Identifier::new(format!(
            "[{}]",
            index_expr.as_ref().map(|e| e.to_string_ast()).unwrap_or_default()
        ))))
    }

    /// maybe_eat_relational_operator → == != < > <= >= instanceof matches between
    fn maybe_eat_relational_operator(&mut self) -> Option<Token> {
        let t = self.peek_token()?;
        match t.kind {
            TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::Lt
            | TokenKind::Le
            | TokenKind::Gt
            | TokenKind::Ge => self.next_token(),
            TokenKind::Identifier => {
                let name = t.string_value().to_lowercase();
                match name.as_str() {
                    "instanceof" => {
                        let mut t = self.next_token().unwrap();
                        Some(t.as_instanceof_token())
                    }
                    "matches" => {
                        let mut t = self.next_token().unwrap();
                        Some(t.as_matches_token())
                    }
                    "between" => {
                        let mut t = self.next_token().unwrap();
                        Some(t.as_between_token())
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    // ─── Token 工具 ──────────────────────────────────────────────

    fn peek_token(&self) -> Option<Token> {
        self.token_stream.get(self.token_stream_pointer).cloned()
    }

    fn peek_token_kind(&self, kinds: &[TokenKind]) -> bool {
        self.peek_token().map_or(false, |t| kinds.contains(&t.kind))
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.token_stream_pointer < self.token_stream_length {
            let t = self.token_stream[self.token_stream_pointer].clone();
            self.token_stream_pointer += 1;
            Some(t)
        } else {
            None
        }
    }

    fn eat_token(&mut self, kind: TokenKind) -> bool {
        let t = self.next_token();
        match t {
            None => false,
            Some(t) if t.kind == kind => true,
            _ => false,
        }
    }
}

impl Default for InternalSpelExpressionParser {
    fn default() -> Self {
        Self::new()
    }
}
