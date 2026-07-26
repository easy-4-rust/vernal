//! AST 节点层。
//!
//! 包含所有 SpEL AST 节点的实现：字面量、运算符、表达式节点。

// ─── 基础设施 ───
pub mod spel_node;
pub mod literal;
pub mod operator;

// ─── 字面量（7 个） ───
pub mod boolean_literal;
pub mod float_literal;
pub mod int_literal;
pub mod long_literal;
pub mod null_literal;
pub mod real_literal;
pub mod string_literal;

// ─── 算术运算符（6 个） ───
pub mod op_divide;
pub mod op_minus;
pub mod op_modulus;
pub mod op_multiply;
pub mod op_plus;
pub mod operator_power;

// ─── 比较运算符（6 个） ───
pub mod op_eq;
pub mod op_ge;
pub mod op_gt;
pub mod op_le;
pub mod op_lt;
pub mod op_ne;

// ─── 逻辑运算符（3 个） ───
pub mod op_and;
pub mod op_or;
pub mod operator_not;
