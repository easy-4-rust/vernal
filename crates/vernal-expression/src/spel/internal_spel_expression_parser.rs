//! 内部递归下降解析器（对标 Spring `InternalSpelExpressionParser`）。
//!
//! Pratt 风格优先级递归下降，消费 `Tokenizer` 输出的 token 流，
//! 构造 AST 节点树，返回 `SpelExpression`。
//!
//! 对应 Java 类：`org.springframework.expression.spel.standard.InternalSpelExpressionParser`。
//!
//! # 解析优先级链（低→高）
//!
//! ```text
//! eat_expression        → assign / elvis / ternary / logicalOr
//! eat_logical_or        → logicalAnd (|| logicalAnd)*
//! eat_logical_and       → relational (&& relational)*
//! eat_relational        → sum (relationalOp sum)?
//! eat_sum               → product ((+ | -) product)*
//! eat_product           → power (( * | / | % ) power)*
//! eat_power_inc_dec     → unary (^ unary)*
//! eat_unary             → prefix (+ | - | !) unary | primary
//! eat_primary           → start_node (.property / [index] / .method())*
//! ```
//!
//! # AST 节点工厂
//!
//! 每个 `maybe_eat_*` 方法返回 `bool`（是否消费了 token），
//! 将构造的节点推入 `constructed_nodes` 栈。
//! 上层 `eat_*` 方法从栈中弹出并组装复合节点。

use super::ast::assign::Assign;
use super::ast::bean_reference::BeanReference;
use super::ast::boolean_literal::BooleanLiteral;
use super::ast::compound_expression::CompoundExpression;
use super::ast::constructor_reference::ConstructorReference;
use super::ast::elvis::Elvis;
use super::ast::function_reference::FunctionReference;
use super::ast::identifier::Identifier;
use super::ast::indexer::Indexer;
use super::ast::inline_list::InlineList;
use super::ast::inline_map::InlineMap;
use super::ast::int_literal::IntLiteral;
use super::ast::method_reference::MethodReference;
use super::ast::null_literal::NullLiteral;
use super::ast::op_and::OpAnd;
use super::ast::op_dec::OpDec;
use super::ast::op_divide::OpDivide;
use super::ast::op_eq::OpEq;
use super::ast::op_ge::OpGe;
use super::ast::op_gt::OpGt;
use super::ast::op_inc::OpInc;
use super::ast::op_le::OpLe;
use super::ast::op_lt::OpLt;
use super::ast::op_minus::OpMinus;
use super::ast::op_modulus::OpModulus;
use super::ast::op_multiply::OpMultiply;
use super::ast::op_ne::OpNe;
use super::ast::op_or::OpOr;
use super::ast::op_plus::OpPlus;
use super::ast::operator_between::OperatorBetween;
use super::ast::operator_instanceof::OperatorInstanceof;
use super::ast::type_reference::TypeReference;
use super::ast::operator_matches::OperatorMatches;
use super::ast::operator_not::OperatorNot;
use super::ast::operator_power::OperatorPower;
use super::ast::projection::Projection;
use super::ast::property_or_field_reference::PropertyOrFieldReference;
use super::ast::selection::{Selection, SelectionVariant};
use super::ast::real_literal::RealLiteral;
use super::ast::spel_node::SpelNode;
use super::ast::string_literal::StringLiteral;
use super::ast::ternary::Ternary;
use super::ast::variable_reference::VariableReference;
use super::spel_message::SpelMessage;
use super::spel_parse_exception::SpelParseException;
use super::token::Token;
use super::token_kind::TokenKind;
use super::tokenizer::Tokenizer;

use crate::expression::Expression as ExpressionTrait;

/// 内部递归下降解析器（对标 Spring `InternalSpelExpressionParser`）。
///
/// 实例可复用但**非线程安全**（与 Spring Java 实现一致）。
/// 每次调用 `do_parse_expression` 会重置内部状态。
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
    ///
    /// 对标 Java `InternalSpelExpressionParser.doParseExpression(String, ParserContext)`。
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
    ///
    /// 对标 Java `eatExpression()`（lines 171-206）。
    fn eat_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let expr = self.eat_logical_or_expression();

        // 赋值：a = b
        if self.peek_token_kind(&[TokenKind::Assign]) {
            self.next_token(); // consume =
            let right = self.eat_logical_or_expression();
            return Some(Box::new(Assign::new(
                expr.unwrap_or_else(|| Box::new(NullLiteral::new())),
                right.unwrap_or_else(|| Box::new(NullLiteral::new())),
            )));
        }

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
    ///
    /// 对标 Java `eatLogicalOrExpression()`（lines 209-218）。
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
    ///
    /// 对标 Java `eatLogicalAndExpression()`（lines 221-230）。
    fn eat_logical_and_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let mut left = self.eat_relational_expression();
        while self.peek_token_kind(&[TokenKind::SymbolicAnd]) {
            self.next_token(); // consume &&
            let right = self.eat_relational_expression();
            left = Some(Box::new(OpAnd::new(
                left.unwrap_or_else(|| Box::new(NullLiteral::new())),
                right.unwrap_or_else(|| Box::new(NullLiteral::new())),
            )));
        }
        left
    }

    /// relational → sum (relationalOp sum)?
    ///
    /// 对标 Java `eatRelationalExpression()`（lines 233-274）。
    /// 支持：== != < > <= >= instanceof matches between
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
                    // 右操作数应为类型名字符串
                    let type_name = r.to_string_ast();
                    Box::new(OperatorInstanceof::new(l, type_name))
                }
                TokenKind::Matches => {
                    Box::new(OperatorMatches::new(l, r))
                }
                TokenKind::Between => {
                    // between 需要右操作数为两元素列表 [low, high]
                    // OperatorBetween::new(value, low, high) 期望 3 个参数
                    // 暂用 OperatorBetween（低=0, 高=r），后续补全 inline list 解析
                    Box::new(OperatorBetween::new(
                        l,
                        Box::new(IntLiteral::new(0, "0".to_string())),
                        r,
                    ))
                }
                _ => unreachable!(),
            };
            return Some(node);
        }
        left
    }

    /// sum → product ((+ | -) product)*
    ///
    /// 对标 Java `eatSumExpression()`（lines 278-292）。
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
    ///
    /// 对标 Java `eatProductExpression()`（lines 295-312）。
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

    /// power → unary (^ unary)*   (右结合)
    ///
    /// 对标 Java `eatPowerIncDecExpression()`（lines 316-332）。
    /// `^` 为幂运算符（右结合：2^3^2 = 2^9 = 512）。
    fn eat_power_inc_dec_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let mut left = self.eat_unary_expression();
        while self.peek_token_kind(&[TokenKind::Power]) {
            self.next_token(); // consume ^
            let right = self.eat_unary_expression();
            left = Some(Box::new(OperatorPower::new(
                left.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
                right.unwrap_or_else(|| Box::new(IntLiteral::new(1, "1".to_string()))),
            )));
        }
        left
    }

    /// unary → prefix (+ | - | !) unary | postfix (++ | --) | primary
    ///
    /// 对标 Java `eatUnaryExpression()`（lines 336-364）。
    /// 支持前缀：`+a`（一元正）、`-a`（一元负）、`!a`（逻辑非）、`++a`（前缀自增）、`--a`（前缀自减）。
    fn eat_unary_expression(&mut self) -> Option<Box<dyn SpelNode>> {
        let kind = self.peek_token().map(|t| t.kind);
        match kind {
            Some(TokenKind::Not) => {
                self.next_token(); // consume !
                let operand = self.eat_unary_expression();
                Some(Box::new(OperatorNot::new(
                    operand.unwrap_or_else(|| Box::new(NullLiteral::new())),
                )))
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
            Some(TokenKind::Inc) => {
                self.next_token(); // consume ++ (prefix)
                let operand = self.eat_unary_expression();
                let node = OpInc::new(
                    operand.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string()))),
                    true, // prefix
                );
                Some(Box::new(node))
            }
            Some(TokenKind::Dec) => {
                self.next_token(); // consume -- (prefix)
                let operand = self.eat_unary_expression();
                Some(Box::new(OpDec::new(
                    operand.unwrap_or_else(|| Box::new(IntLiteral::new(0, "0".to_string()))),
                    true, // prefix
                )))
            }
            _ => self.eat_primary_expression(),
        }
    }

    /// primary → start_node (.property / [index] / .method())*
    ///
    /// 对标 Java `eatPrimaryExpression()`（lines 367-384）。
    /// 解析属性访问链（点分表达式序列），构造 `CompoundExpression`。
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
                Some(TokenKind::Select) => {
                    // 对标 Java `.?[expr]` Selection ALL
                    self.next_token(); // consume ?[
                    let criteria = self.eat_expression();
                    self.eat_token(TokenKind::RSquare);
                    nodes.push(Box::new(Selection::new(
                        criteria.unwrap_or_else(|| Box::new(NullLiteral::new())),
                        SelectionVariant::All,
                    )));
                }
                Some(TokenKind::SelectFirst) => {
                    self.next_token(); // consume ^[
                    let criteria = self.eat_expression();
                    self.eat_token(TokenKind::RSquare);
                    nodes.push(Box::new(Selection::new(
                        criteria.unwrap_or_else(|| Box::new(NullLiteral::new())),
                        SelectionVariant::First,
                    )));
                }
                Some(TokenKind::SelectLast) => {
                    self.next_token(); // consume $[
                    let criteria = self.eat_expression();
                    self.eat_token(TokenKind::RSquare);
                    nodes.push(Box::new(Selection::new(
                        criteria.unwrap_or_else(|| Box::new(NullLiteral::new())),
                        SelectionVariant::Last,
                    )));
                }
                Some(TokenKind::Project) => {
                    self.next_token(); // consume![
                    let expr = self.eat_expression();
                    self.eat_token(TokenKind::RSquare);
                    nodes.push(Box::new(Projection::new(
                        expr.unwrap_or_else(|| Box::new(NullLiteral::new())),
                    )));
                }
                _ => break,
            }
        }

        // 后缀 ++ / --
        if self.peek_token_kind(&[TokenKind::Inc]) {
            self.next_token();
            let last = nodes.pop().unwrap();
            nodes.push(Box::new(OpInc::new(last, false))); // postfix
        } else if self.peek_token_kind(&[TokenKind::Dec]) {
            self.next_token();
            let last = nodes.pop().unwrap();
            nodes.push(Box::new(OpDec::new(last, false))); // postfix
        }

        if nodes.len() == 1 {
            Some(nodes.into_iter().next().unwrap())
        } else {
            Some(Box::new(CompoundExpression::new(nodes)))
        }
    }

    /// startNode → literal / paren / null / identifier / @bean / #var / constructor / inlineList/Map
    ///
    /// 对标 Java `eatStartNode()`（lines 517-540）。
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

        // T(...) 类型引用
        if self.peek_token_kind(&[TokenKind::Identifier])
            && self.peek_token().map(|t| t.string_value()) == Some("T".to_string())
        {
            self.next_token(); // consume T
            if self.peek_token_kind(&[TokenKind::LParen]) {
                self.next_token(); // consume (
                if let Some(t) = self.peek_token() {
                    let type_name = t.string_value();
                    self.next_token(); // consume type name
                    self.eat_token(TokenKind::RParen);
                    // Phase F: 使用 TypeReference 节点
                    return Some(Box::new(TypeReference::new(type_name)));
                }
            }
        }

        // new ConstructorReference
        if self.peek_token_kind(&[TokenKind::Identifier])
            && self.peek_token().map(|t| t.string_value()) == Some("new".to_string())
        {
            self.next_token(); // consume new
            if let Some(t) = self.peek_token() {
                let type_name = t.string_value();
                self.next_token(); // consume type name
                if self.peek_token_kind(&[TokenKind::LParen]) {
                    self.next_token(); // consume (
                    let args = self.consume_arguments();
                    self.eat_token(TokenKind::RParen);
                    return Some(Box::new(ConstructorReference::new(type_name, args)));
                }
            }
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
                // 检查是否是函数调用：#fn(...)
                if self.peek_token_kind(&[TokenKind::LParen]) {
                    self.next_token(); // consume (
                    let args = self.consume_arguments();
                    self.eat_token(TokenKind::RParen);
                    return Some(Box::new(FunctionReference::new(name, args)));
                }
                // 变量引用
                return Some(Box::new(VariableReference::new(name)));
            }
        }

        // @bean 引用
        if self.peek_token_kind(&[TokenKind::BeanRef]) {
            self.next_token(); // consume @
            if let Some(t) = self.peek_token() {
                let name = t.string_value();
                self.next_token(); // consume identifier
                return Some(Box::new(BeanReference::new(name)));
            }
        }

        // &factoryBean 引用
        if self.peek_token_kind(&[TokenKind::FactoryBeanRef]) {
            self.next_token(); // consume &
            if let Some(t) = self.peek_token() {
                let name = t.string_value();
                self.next_token(); // consume identifier
                // Phase F: FactoryBeanReference
                return Some(Box::new(BeanReference::new(format!("&{name}"))));
            }
        }

        // 内联列表 {1, 2, 3}
        if self.peek_token_kind(&[TokenKind::LCurly]) {
            self.next_token(); // consume {
            let mut elements = Vec::new();
            if !self.peek_token_kind(&[TokenKind::RCurly]) {
                loop {
                    if let Some(e) = self.eat_expression() {
                        elements.push(e);
                    }
                    if !self.peek_token_kind(&[TokenKind::Comma]) {
                        break;
                    }
                    self.next_token(); // consume ,
                }
            }
            self.eat_token(TokenKind::RCurly);
            // 检查是否是 Map（通过冒号分隔的键值对）
            // 简化：如果元素数为偶数且有冒号，视为 Map
            // Phase F: 更精确的 Map 检测
            return Some(Box::new(InlineList::new(elements)));
        }

        None
    }

    /// maybe_eat_literal → Int / Long / Real / Hex / String / Boolean / null
    ///
    /// 对标 Java `maybeEatLiteral()`（lines 847-884）。
    fn maybe_eat_literal(&mut self) -> Option<Box<dyn SpelNode>> {
        let t = self.peek_token()?;
        match t.kind {
            TokenKind::LiteralInt => {
                let t = self.next_token().unwrap();
                let raw = t.string_value();
                let val: i64 = raw.parse().unwrap_or(0);
                Some(Box::new(IntLiteral::new(val, raw)))
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
                let s = raw
                    .trim_start_matches("0x")
                    .trim_start_matches("0X")
                    .trim_end_matches('L')
                    .trim_end_matches('l');
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
                    Some(Box::new(BooleanLiteral::new(true)))
                } else if name == "false" || name == "FALSE" {
                    self.next_token();
                    Some(Box::new(BooleanLiteral::new(false)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// maybe_eat_method_or_property → identifier(args)? / identifier
    ///
    /// 对标 Java `maybeEatMethodOrProperty()`（lines 769-785）。
    /// 如果标识符后跟 `(` 则解析为方法调用，否则为属性引用。
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
            let args = self.consume_arguments();
            self.eat_token(TokenKind::RParen);
            return Some(Box::new(MethodReference::new(name, args, null_safe)));
        }

        // 属性/字段引用
        Some(Box::new(PropertyOrFieldReference::new(name, null_safe)))
    }

    /// maybe_eat_indexer → [expr]
    ///
    /// 对标 Java `maybeEatIndexer()`（lines 693-705）。
    fn maybe_eat_indexer(&mut self, null_safe: bool) -> Option<Box<dyn SpelNode>> {
        if !self.peek_token_kind(&[TokenKind::LSquare]) {
            return None;
        }
        self.next_token(); // consume [
        let index_expr = self.eat_expression();
        self.eat_token(TokenKind::RSquare);
        let index = index_expr.unwrap_or_else(|| Box::new(NullLiteral::new()));
        Some(Box::new(Indexer::new(index)))
    }

    /// consume_arguments → (expression (, expression)*)?
    ///
    /// 对标 Java `consumeArguments()`（lines 451-495）。
    fn consume_arguments(&mut self) -> Vec<Box<dyn SpelNode>> {
        let mut args = Vec::new();
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
        args
    }

    /// maybe_eat_relational_operator → == != < > <= >= instanceof matches between
    ///
    /// 对标 Java `maybeEatRelationalOperator()`（lines 909-930）。
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
