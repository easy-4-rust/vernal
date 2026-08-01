<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-expression 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 117 个 class/interface/enum/record；`package-info.java` 不计入 |
| 目录算法 | 去掉组织和模块根包，保留末 2 层包目录 |
| 文件边界 | 一个 Java 对象对应一个 snake_case `.rs` 文件；内部类/Builder 可随主对象 |
| 模块文件 | `lib.rs`/`mod.rs` 只允许模块文档、声明和显式重导出 |
| 完成状态 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |
| 未完成状态 | `MISSING`、`MISPLACED`、`STUB`、`PARTIAL`、`UNVERIFIED` |
| 注释与测试 | 中文 Java 来源注释；正常、失败、边界和生命周期语义测试 |

本文件顶部事实区始终按当前源码重新生成；下方历史设计附录不得覆盖这里的对象数量、路径、状态或证据。
<!-- current-migration-contract-end -->

## 汇总

| 指标 | 数量 |
|---|---:|
| Java 业务对象 | 117 |
| 已处理（严格三类） | 7 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 7 |
| `MISPLACED` | 7 |
| `MISSING` | 11 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 92 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `method_filter.rs`：`MethodFilter`、`MethodFilterRegistry`
- 单文件多个公开对象位于 `property_accessor.rs`：`PropertyAccessor`、`IndexAccessor`
- 单文件多个公开对象位于 `parser_context.rs`：`ParserContext`、`TemplateParserContext`
- 单文件多个公开对象位于 `type_descriptor.rs`：`PrimitiveKind`、`TypeDescriptor`
- 单文件多个公开对象位于 `spel/spel_message.rs`：`SpelMessage`、`MessageKind`
- 单文件多个公开对象位于 `spel/support/reflective_method_resolver.rs`：`ArcReflectiveMethodExecutor`、`ReflectiveMethodResolver`、`ReflectiveMethodExecutor`
- 单文件多个公开对象位于 `spel/support/reflective_property_accessor.rs`：`PropertyBinding`、`ReflectivePropertyAccessor`
- 单文件多个公开对象位于 `spel/support/reflective_constructor_resolver.rs`：`ReflectiveConstructorExecutor`、`ReflectiveConstructorResolver`
- 单文件多个公开对象位于 `spel/ast/selection.rs`：`SelectionVariant`、`Selection`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.expression.AccessException` | `AccessException.java` | `access_exception.rs` | `access_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.BeanResolver` | `BeanResolver.java` | `bean_resolver.rs` | `bean_resolver.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.ConstructorExecutor` | `ConstructorExecutor.java` | `constructor_executor.rs` | `constructor_executor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.ConstructorResolver` | `ConstructorResolver.java` | `constructor_resolver.rs` | `constructor_resolver.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.EvaluationContext` | `EvaluationContext.java` | `evaluation_context.rs` | `evaluation_context.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/selection_projection_tests.rs` |
| `org.springframework.expression.EvaluationException` | `EvaluationException.java` | `evaluation_exception.rs` | `evaluation_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.Expression` | `Expression.java` | `expression.rs` | `expression.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/selection_projection_tests.rs` |
| `org.springframework.expression.ExpressionException` | `ExpressionException.java` | `expression_exception.rs` | `expression_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.ExpressionInvocationTargetException` | `ExpressionInvocationTargetException.java` | `expression_invocation_target_exception.rs` | `expression_invocation_target_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.expression.ExpressionParser` | `ExpressionParser.java` | `expression_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.IndexAccessor` | `IndexAccessor.java` | `index_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.MethodExecutor` | `MethodExecutor.java` | `method_executor.rs` | `method_executor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.MethodFilter` | `MethodFilter.java` | `method_filter.rs` | `method_filter.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.expression.MethodResolver` | `MethodResolver.java` | `method_resolver.rs` | `method_resolver.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.Operation` | `Operation.java` | `operation.rs` | `operation.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.OperatorOverloader` | `OperatorOverloader.java` | `operator_overloader.rs` | `operator_overloader.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.ParseException` | `ParseException.java` | `parse_exception.rs` | `parse_exception.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/common_module_tests.rs` |
| `org.springframework.expression.ParserContext` | `ParserContext.java` | `parser_context.rs` | `parser_context.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.PropertyAccessor` | `PropertyAccessor.java` | `property_accessor.rs` | `property_accessor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.TargetedAccessor` | `TargetedAccessor.java` | `targeted_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.TypeComparator` | `TypeComparator.java` | `type_comparator.rs` | `type_comparator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.TypeConverter` | `TypeConverter.java` | `type_converter.rs` | `type_converter.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.TypeLocator` | `TypeLocator.java` | `type_locator.rs` | `type_locator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.TypedValue` | `TypedValue.java` | `typed_value.rs` | `typed_value.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/selection_projection_tests.rs` |
| `org.springframework.expression.common.CompositeStringExpression` | `common/CompositeStringExpression.java` | `common/composite_string_expression.rs` | `common/composite_string_expression.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.common.ExpressionUtils` | `common/ExpressionUtils.java` | `common/expression_utils.rs` | `common/expression_utils.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.common.LiteralExpression` | `common/LiteralExpression.java` | `common/literal_expression.rs` | `common/literal_expression.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.common.TemplateAwareExpressionParser` | `common/TemplateAwareExpressionParser.java` | `common/template_aware_expression_parser.rs` | `common/template_aware_expression_parser.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.common.TemplateParserContext` | `common/TemplateParserContext.java` | `common/template_parser_context.rs` | `common/template_parser_context.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.CodeFlow` | `spel/CodeFlow.java` | `spel/code_flow.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.CompilableIndexAccessor` | `spel/CompilableIndexAccessor.java` | `spel/compilable_index_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.CompilablePropertyAccessor` | `spel/CompilablePropertyAccessor.java` | `spel/compilable_property_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.CompiledExpression` | `spel/CompiledExpression.java` | `spel/compiled_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.ExpressionState` | `spel/ExpressionState.java` | `spel/expression_state.rs` | `spel/expression_state.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.InternalParseException` | `spel/InternalParseException.java` | `spel/internal_parse_exception.rs` | `spel/internal_parse_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.SpelCompilerMode` | `spel/SpelCompilerMode.java` | `spel/spel_compiler_mode.rs` | `spel/spel_compiler_mode.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.expression.spel.SpelEvaluationException` | `spel/SpelEvaluationException.java` | `spel/spel_evaluation_exception.rs` | `spel/spel_evaluation_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.SpelMessage` | `spel/SpelMessage.java` | `spel/spel_message.rs` | `spel/spel_message.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.SpelNode` | `spel/SpelNode.java` | `spel/spel_node.rs` | `spel/ast/spel_node.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.expression.spel.SpelParseException` | `spel/SpelParseException.java` | `spel/spel_parse_exception.rs` | `spel/spel_parse_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.SpelParserConfiguration` | `spel/SpelParserConfiguration.java` | `spel/spel_parser_configuration.rs` | `spel/spel_parser_configuration.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.AccessorUtils` | `spel/ast/AccessorUtils.java` | `spel/ast/accessor_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.ast.Assign` | `spel/ast/Assign.java` | `spel/ast/assign.rs` | `spel/ast/assign.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.BeanReference` | `spel/ast/BeanReference.java` | `spel/ast/bean_reference.rs` | `spel/ast/bean_reference.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.BooleanLiteral` | `spel/ast/BooleanLiteral.java` | `spel/ast/boolean_literal.rs` | `spel/ast/boolean_literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.CompoundExpression` | `spel/ast/CompoundExpression.java` | `spel/ast/compound_expression.rs` | `spel/ast/compound_expression.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.ConstructorReference` | `spel/ast/ConstructorReference.java` | `spel/ast/constructor_reference.rs` | `spel/ast/constructor_reference.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Elvis` | `spel/ast/Elvis.java` | `spel/ast/elvis.rs` | `spel/ast/elvis.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.FloatLiteral` | `spel/ast/FloatLiteral.java` | `spel/ast/float_literal.rs` | `spel/ast/float_literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.FormatHelper` | `spel/ast/FormatHelper.java` | `spel/ast/format_helper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.ast.FunctionReference` | `spel/ast/FunctionReference.java` | `spel/ast/function_reference.rs` | `spel/ast/function_reference.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Identifier` | `spel/ast/Identifier.java` | `spel/ast/identifier.rs` | `spel/ast/identifier.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Indexer` | `spel/ast/Indexer.java` | `spel/ast/indexer.rs` | `spel/ast/indexer.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.InlineList` | `spel/ast/InlineList.java` | `spel/ast/inline_list.rs` | `spel/ast/inline_list.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.InlineMap` | `spel/ast/InlineMap.java` | `spel/ast/inline_map.rs` | `spel/ast/inline_map.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.IntLiteral` | `spel/ast/IntLiteral.java` | `spel/ast/int_literal.rs` | `spel/ast/int_literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Literal` | `spel/ast/Literal.java` | `spel/ast/literal.rs` | `spel/ast/literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.LongLiteral` | `spel/ast/LongLiteral.java` | `spel/ast/long_literal.rs` | `spel/ast/long_literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.MethodReference` | `spel/ast/MethodReference.java` | `spel/ast/method_reference.rs` | `spel/ast/method_reference.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.NullLiteral` | `spel/ast/NullLiteral.java` | `spel/ast/null_literal.rs` | `spel/ast/null_literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpAnd` | `spel/ast/OpAnd.java` | `spel/ast/op_and.rs` | `spel/ast/op_and.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpDec` | `spel/ast/OpDec.java` | `spel/ast/op_dec.rs` | `spel/ast/op_dec.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpDivide` | `spel/ast/OpDivide.java` | `spel/ast/op_divide.rs` | `spel/ast/op_divide.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpEQ` | `spel/ast/OpEQ.java` | `spel/ast/op_eq.rs` | `spel/ast/op_eq.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.OpGE` | `spel/ast/OpGE.java` | `spel/ast/op_ge.rs` | `spel/ast/op_ge.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.OpGT` | `spel/ast/OpGT.java` | `spel/ast/op_gt.rs` | `spel/ast/op_gt.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.OpInc` | `spel/ast/OpInc.java` | `spel/ast/op_inc.rs` | `spel/ast/op_inc.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpLE` | `spel/ast/OpLE.java` | `spel/ast/op_le.rs` | `spel/ast/op_le.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.OpLT` | `spel/ast/OpLT.java` | `spel/ast/op_lt.rs` | `spel/ast/op_lt.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.OpMinus` | `spel/ast/OpMinus.java` | `spel/ast/op_minus.rs` | `spel/ast/op_minus.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpModulus` | `spel/ast/OpModulus.java` | `spel/ast/op_modulus.rs` | `spel/ast/op_modulus.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpMultiply` | `spel/ast/OpMultiply.java` | `spel/ast/op_multiply.rs` | `spel/ast/op_multiply.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpNE` | `spel/ast/OpNE.java` | `spel/ast/op_ne.rs` | `spel/ast/op_ne.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.OpOr` | `spel/ast/OpOr.java` | `spel/ast/op_or.rs` | `spel/ast/op_or.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OpPlus` | `spel/ast/OpPlus.java` | `spel/ast/op_plus.rs` | `spel/ast/op_plus.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Operator` | `spel/ast/Operator.java` | `spel/ast/operator.rs` | `spel/ast/operator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型 |
| `org.springframework.expression.spel.ast.OperatorBetween` | `spel/ast/OperatorBetween.java` | `spel/ast/operator_between.rs` | `spel/ast/operator_between.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OperatorInstanceof` | `spel/ast/OperatorInstanceof.java` | `spel/ast/operator_instanceof.rs` | `spel/ast/operator_instanceof.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OperatorMatches` | `spel/ast/OperatorMatches.java` | `spel/ast/operator_matches.rs` | `spel/ast/operator_matches.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OperatorNot` | `spel/ast/OperatorNot.java` | `spel/ast/operator_not.rs` | `spel/ast/operator_not.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.OperatorPower` | `spel/ast/OperatorPower.java` | `spel/ast/operator_power.rs` | `spel/ast/operator_power.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Projection` | `spel/ast/Projection.java` | `spel/ast/projection.rs` | `spel/ast/projection.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.PropertyOrFieldReference` | `spel/ast/PropertyOrFieldReference.java` | `spel/ast/property_or_field_reference.rs` | `spel/ast/property_or_field_reference.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.QualifiedIdentifier` | `spel/ast/QualifiedIdentifier.java` | `spel/ast/qualified_identifier.rs` | `spel/ast/qualified_identifier.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.RealLiteral` | `spel/ast/RealLiteral.java` | `spel/ast/real_literal.rs` | `spel/ast/real_literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Selection` | `spel/ast/Selection.java` | `spel/ast/selection.rs` | `spel/ast/selection.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.SpelNodeImpl` | `spel/ast/SpelNodeImpl.java` | `spel/ast/spel_node_impl.rs` | `spel/ast/spel_node_impl.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.StringLiteral` | `spel/ast/StringLiteral.java` | `spel/ast/string_literal.rs` | `spel/ast/string_literal.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.Ternary` | `spel/ast/Ternary.java` | `spel/ast/ternary.rs` | `spel/ast/ternary.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.TypeCode` | `spel/ast/TypeCode.java` | `spel/ast/type_code.rs` | `spel/ast/type_code.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.TypeReference` | `spel/ast/TypeReference.java` | `spel/ast/type_reference.rs` | `spel/ast/type_reference.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.ast.ValueRef` | `spel/ast/ValueRef.java` | `spel/ast/value_ref.rs` | `spel/ast/value_ref.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.expression.spel.ast.VariableReference` | `spel/ast/VariableReference.java` | `spel/ast/variable_reference.rs` | `spel/ast/variable_reference.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.standard.InternalSpelExpressionParser` | `spel/standard/InternalSpelExpressionParser.java` | `spel/standard/internal_spel_expression_parser.rs` | `spel/internal_spel_expression_parser.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.expression.spel.standard.SpelCompiler` | `spel/standard/SpelCompiler.java` | `spel/standard/spel_compiler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.standard.SpelExpression` | `spel/standard/SpelExpression.java` | `spel/standard/spel_expression.rs` | `spel/spel_expression.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.expression.spel.standard.SpelExpressionParser` | `spel/standard/SpelExpressionParser.java` | `spel/standard/spel_expression_parser.rs` | `spel/spel_expression_parser.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.expression.spel.standard.Token` | `spel/standard/Token.java` | `spel/standard/token.rs` | `spel/token.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.expression.spel.standard.TokenKind` | `spel/standard/TokenKind.java` | `spel/standard/token_kind.rs` | `spel/token_kind.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.expression.spel.standard.Tokenizer` | `spel/standard/Tokenizer.java` | `spel/standard/tokenizer.rs` | `spel/tokenizer.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.expression.spel.support.BooleanTypedValue` | `spel/support/BooleanTypedValue.java` | `spel/support/boolean_typed_value.rs` | `spel/support/boolean_typed_value.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.DataBindingMethodResolver` | `spel/support/DataBindingMethodResolver.java` | `spel/support/data_binding_method_resolver.rs` | `spel/support/data_binding_method_resolver.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/spel/support/data_binding_method_resolver.rs#[cfg(test)]` |
| `org.springframework.expression.spel.support.DataBindingPropertyAccessor` | `spel/support/DataBindingPropertyAccessor.java` | `spel/support/data_binding_property_accessor.rs` | `spel/support/data_binding_property_accessor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/spel/support/data_binding_property_accessor.rs#[cfg(test)]` |
| `org.springframework.expression.spel.support.MapAccessor` | `spel/support/MapAccessor.java` | `spel/support/map_accessor.rs` | `spel/support/map_accessor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.ReflectionHelper` | `spel/support/ReflectionHelper.java` | `spel/support/reflection_helper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.expression.spel.support.ReflectiveConstructorExecutor` | `spel/support/ReflectiveConstructorExecutor.java` | `spel/support/reflective_constructor_executor.rs` | `spel/support/reflective_constructor_executor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.ReflectiveConstructorResolver` | `spel/support/ReflectiveConstructorResolver.java` | `spel/support/reflective_constructor_resolver.rs` | `spel/support/reflective_constructor_resolver.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/spel/support/reflective_constructor_resolver.rs#[cfg(test)]` |
| `org.springframework.expression.spel.support.ReflectiveIndexAccessor` | `spel/support/ReflectiveIndexAccessor.java` | `spel/support/reflective_index_accessor.rs` | `spel/support/reflective_index_accessor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.ReflectiveMethodExecutor` | `spel/support/ReflectiveMethodExecutor.java` | `spel/support/reflective_method_executor.rs` | `spel/support/reflective_method_executor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.ReflectiveMethodResolver` | `spel/support/ReflectiveMethodResolver.java` | `spel/support/reflective_method_resolver.rs` | `spel/support/reflective_method_resolver.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.ReflectivePropertyAccessor` | `spel/support/ReflectivePropertyAccessor.java` | `spel/support/reflective_property_accessor.rs` | `spel/support/reflective_property_accessor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.SimpleEvaluationContext` | `spel/support/SimpleEvaluationContext.java` | `spel/support/simple_evaluation_context.rs` | `spel/support/simple_evaluation_context.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.StandardEvaluationContext` | `spel/support/StandardEvaluationContext.java` | `spel/support/standard_evaluation_context.rs` | `spel/support/standard_evaluation_context.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.StandardOperatorOverloader` | `spel/support/StandardOperatorOverloader.java` | `spel/support/standard_operator_overloader.rs` | `spel/support/standard_operator_overloader.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.StandardTypeComparator` | `spel/support/StandardTypeComparator.java` | `spel/support/standard_type_comparator.rs` | `spel/support/standard_type_comparator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.StandardTypeConverter` | `spel/support/StandardTypeConverter.java` | `spel/support/standard_type_converter.rs` | `spel/support/standard_type_converter.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.expression.spel.support.StandardTypeLocator` | `spel/support/StandardTypeLocator.java` | `spel/support/standard_type_locator.rs` | `spel/support/standard_type_locator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
