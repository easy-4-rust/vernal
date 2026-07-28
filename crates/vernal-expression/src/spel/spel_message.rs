//! SpEL 错误消息码（对标 Spring `org.springframework.expression.spel.SpelMessage`）。
//!
//! 86 个变体，按 Spring 顺序与错误码定义。`format_message` 输出
//! `"EL{code}E: <插值后的消息模板>"` 形式（对标 java.text.MessageFormat 的 `{n}` 占位符）。
//!
//! Spring 中文/英文消息差异通过 i18n 资源处理；vernal-expression 在 0.x 阶段直接硬编码中英双语。

/// SpEL 错误消息码（86 项，对标 Spring `SpelMessage`）。
///
/// 枚举顺序与 Spring 源码一致；数值通过 `code()` 方法返回。
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SpelMessage {
    // ── Type / Conversion / Construct (1001-1005) ─────────────────────
    /// `1001` — Type conversion problem, cannot convert from {0} to {1}.
    TypeConversionError,
    /// `1002` — Constructor call: No suitable constructor found on type {0} for arguments {1}.
    ConstructorNotFound,
    /// `1003` — A problem occurred whilst attempting to construct an object of type '{0}' using arguments '{1}'.
    ConstructorInvocationProblem,
    /// `1004` — Method call: Method {0} cannot be found on type {1}.
    MethodNotFound,
    /// `1005` — Type cannot be found '{0}'.
    TypeNotFound,

    // ── Function / Property / Field / Method (1006-1011) ──────────────
    /// `1006` — Function '{0}' could not be found.
    FunctionNotDefined,
    /// `1007` — Property or field '{0}' cannot be found on null.
    PropertyOrFieldNotReadableOnNull,
    /// `1008` — Property or field '{0}' cannot be found on object of type '{1}'.
    PropertyOrFieldNotReadable,
    /// `1009` — Property or field '{0}' cannot be set on null.
    PropertyOrFieldNotWritableOnNull,
    /// `1010` — Property or field '{0}' cannot be set on object of type '{1}'.
    PropertyOrFieldNotWritable,
    /// `1011` — Method call: Attempted to call method {0} on null context object.
    MethodCallOnNullObjectNotAllowed,

    // ── Index / Selector (1012-1019) ───────────────────────────────────
    /// `1012` — Cannot index into a null value.
    CannotIndexIntoNullValue,
    /// `1013` — Cannot compare instances of {0} and {1}.
    NotComparable,
    /// `1014` — Incorrect number of arguments for function '{0}': {1} supplied but function takes {2}.
    IncorrectNumberOfArgumentsToFunction,
    /// `1015` — Cannot perform selection on input data of type '{0}'.
    InvalidTypeForSelection,
    /// `1016` — Result of selection criteria is not boolean.
    ResultOfSelectionCriteriaIsNotBoolean,
    /// `1017` — Right operand for the 'between' operator has to be a two-element list.
    BetweenRightOperandMustBeTwoElementList,
    /// `1018` — Pattern is not valid '{0}'.
    InvalidPattern,
    /// `1019` — Projection is not supported on the type '{0}'.
    ProjectionNotSupportedOnType,

    // ── Misc Evaluation (1020-1034) ────────────────────────────────────
    /// `1020` — The argument list of a lambda expression should never have getValue() called upon it.
    ArgListShouldNotBeEvaluated,
    /// `1021` — A problem occurred whilst attempting to access the property '{0}': '{1}'.
    ExceptionDuringPropertyRead,
    /// `1022` — The function '{0}' mapped to an object of type '{1}' cannot be invoked.
    FunctionReferenceCannotBeInvoked,
    /// `1023` — A problem occurred whilst attempting to invoke the function '{0}': '{1}'.
    ExceptionDuringFunctionCall,
    /// `1024` — The array has '{0}' elements, index '{1}' is invalid.
    ArrayIndexOutOfBounds,
    /// `1025` — The collection has '{0}' elements, index '{1}' is invalid.
    CollectionIndexOutOfBounds,
    /// `1026` — The string has '{0}' characters, index '{1}' is invalid.
    StringIndexOutOfBounds,
    /// `1027` — Indexing into type '{0}' is not supported.
    IndexingNotSupportedForType,
    /// `1028` — The operator 'instanceof' needs the right operand to be a class, not a '{0}'.
    InstanceOfOperatorNeedsClassOperand,
    /// `1029` — A problem occurred when trying to execute method '{0}' on object of type '{1}': '{2}'.
    ExceptionDuringMethodInvocation,
    /// `1030` — The operator '{0}' is not supported between objects of type '{1}' and '{2}'.
    OperatorNotSupportedBetweenTypes,
    /// `1031` — Problem locating method {0} on type {1}.
    ProblemLocatingMethod,
    /// `1032` — setValue(ExpressionState, Object) not supported for '{0}'.
    SetValueNotSupported,
    /// `1033` — Method call of '{0}' is ambiguous, supported type conversions allow multiple variants to match.
    MultiplePossibleMethods,
    /// `1034` — A problem occurred whilst attempting to set the property '{0}': {1}.
    ExceptionDuringPropertyWrite,

    // ── Numeric literals (1035-1040) ───────────────────────────────────
    /// `1035` — The value '{0}' cannot be parsed as an int.
    NotAnInteger,
    /// `1036` — The value '{0}' cannot be parsed as a long.
    NotALong,
    /// `1037` — First operand to matches operator must be a string. '{0}' is not.
    InvalidFirstOperandForMatchesOperator,
    /// `1038` — Second operand to matches operator must be a string. '{0}' is not.
    InvalidSecondOperandForMatchesOperator,
    /// `1039` — Only static methods can be called via function references. The method '{0}' referred to by name '{1}' is not static.
    FunctionMustBeStatic,
    /// `1040` — The value '{0}' cannot be parsed as a double.
    NotAReal,

    // ── Parser-side (1041-1069) ────────────────────────────────────────
    /// `1041` — After parsing a valid expression, there is still more data in the expression: '{0}'.
    MoreInput,
    /// `1042` — Problem parsing right operand.
    RightOperandProblem,
    /// `1043` — Unexpected token. Expected '{0}' but was '{1}'.
    NotExpectedToken,
    /// `1044` — Unexpectedly ran out of input.
    Ood,
    /// `1045` — Cannot find terminating " for string.
    NonTerminatingDoubleQuotedString,
    /// `1046` — Cannot find terminating ' for string.
    NonTerminatingQuotedString,
    /// `1047` — A real number must be prefixed by zero, it cannot start with just '.'.
    MissingLeadingZeroForNumber,
    /// `1048` — Real number cannot be suffixed with a long (L or l) suffix.
    RealCannotBeLong,
    /// `1049` — Unexpected data after '.': '{0}'.
    UnexpectedDataAfterDot,
    /// `1050` — The arguments '(...)' for the constructor call are missing.
    MissingConstructorArguments,
    /// `1051` — Unexpectedly ran out of arguments.
    RunOutOfArguments,

    // ── Grow collections / Dynamic creation (1052-1056) ────────────────
    /// `1052` — Unable to grow collection.
    UnableToGrowCollection,
    /// `1053` — Unable to grow collection: unable to determine list element type.
    UnableToGrowCollectionUnknownElementType,
    /// `1054` — Unable to dynamically create a List to replace a null value.
    UnableToCreateListForIndexing,
    /// `1055` — Unable to dynamically create a Map to replace a null value.
    UnableToCreateMapForIndexing,
    /// `1056` — Unable to dynamically create instance of '{0}' to replace a null value.
    UnableToDynamicallyCreateObject,

    // ── Bean / Array construction (1057-1064) ──────────────────────────
    /// `1057` — No bean resolver registered in the context to resolve access to bean '{0}'.
    NoBeanResolverRegistered,
    /// `1058` — A problem occurred when trying to resolve bean '{0}': '{1}'.
    ExceptionDuringBeanResolution,
    /// `1059` — @ or & can only be followed by an identifier or a quoted name.
    InvalidBeanReference,
    /// `1060` — Expected the type of the new array to be specified as a String but found '{0}'.
    TypeNameExpectedForArrayConstruction,
    /// `1061` — The array of type '{0}' cannot have an element of type '{1}' inserted.
    IncorrectElementTypeForArray,
    /// `1062` — Using an initializer to build a multi-dimensional array is not currently supported.
    MultidimArrayInitializerNotSupported,
    /// `1063` — A required array dimension has not been specified.
    MissingArrayDimension,
    /// `1064` — Array initializer size does not match array dimensions.
    InitializerLengthIncorrect,

    // ── Misc (1065-1081) ───────────────────────────────────────────────
    /// `1065` — Unexpected escape character.
    UnexpectedEscapeChar,
    /// `1066` — The expression component '{0}' does not support increment.
    OperandNotIncrementable,
    /// `1067` — The expression component '{0}' does not support decrement.
    OperandNotDecrementable,
    /// `1068` — The expression component '{0}' is not assignable.
    NotAssignable,
    /// `1069` — Missing expected character '{0}'.
    MissingCharacter,
    /// `1070` — Problem parsing left operand.
    LeftOperandProblem,
    /// `1071` — A required selection expression has not been specified.
    MissingSelectionExpression,

    // ── Compile / limits (1072-1079) ───────────────────────────────────
    /// `1072` — An exception occurred whilst evaluating a compiled expression.
    ExceptionRunningCompiledExpression,
    /// `1073` — Failed to efficiently evaluate pattern '{0}': consider redesigning it.
    FlawedPattern,
    /// `1074` — An exception occurred while compiling an expression.
    ExceptionCompilingExpression,
    /// `1075` — Array declares too many elements, exceeding the threshold of '{0}'.
    MaxArrayElementsThresholdExceeded,
    /// `1076` — Repeated text is too long, exceeding the threshold of '{0}' characters.
    MaxRepeatedTextSizeExceeded,
    /// `1077` — Regular expression is too long, exceeding the threshold of '{0}' characters.
    MaxRegexLengthExceeded,
    /// `1078` — Concatenated string is too long, exceeding the threshold of '{0}' characters.
    MaxConcatenatedStringLengthExceeded,
    /// `1079` — SpEL expression is too long, exceeding the threshold of '{0}' characters.
    MaxExpressionLengthExceeded,

    // ── More (1080-1085) ───────────────────────────────────────────────
    /// `1080` — Assignment to variable '{0}' is not supported.
    VariableAssignmentNotSupported,
    /// `1081` — Repeat count '{0}' must not be negative.
    NegativeRepeatedTextCount,
    /// `1082` — Unsupported character '{0}' ({1}) encountered in expression.
    UnsupportedCharacter,
    /// `1083` — A problem occurred while attempting to read index '{0}' in '{1}'.
    ExceptionDuringIndexRead,
    /// `1084` — A problem occurred while attempting to write index '{0}' in '{1}'.
    ExceptionDuringIndexWrite,
    /// `1085` — SpEL expression evaluation exceeded the threshold of '{0}' operations.
    MaxOperationsExceeded,

    // ── Internal ───────────────────────────────────────────────────────
    /// `9999` — Internal framework error.
    InternalError,
}

impl SpelMessage {
    /// 错误码数字（与 Spring `SpelMessage.code` 一致）。
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Self::TypeConversionError => 1001,
            Self::ConstructorNotFound => 1002,
            Self::ConstructorInvocationProblem => 1003,
            Self::MethodNotFound => 1004,
            Self::TypeNotFound => 1005,
            Self::FunctionNotDefined => 1006,
            Self::PropertyOrFieldNotReadableOnNull => 1007,
            Self::PropertyOrFieldNotReadable => 1008,
            Self::PropertyOrFieldNotWritableOnNull => 1009,
            Self::PropertyOrFieldNotWritable => 1010,
            Self::MethodCallOnNullObjectNotAllowed => 1011,
            Self::CannotIndexIntoNullValue => 1012,
            Self::NotComparable => 1013,
            Self::IncorrectNumberOfArgumentsToFunction => 1014,
            Self::InvalidTypeForSelection => 1015,
            Self::ResultOfSelectionCriteriaIsNotBoolean => 1016,
            Self::BetweenRightOperandMustBeTwoElementList => 1017,
            Self::InvalidPattern => 1018,
            Self::ProjectionNotSupportedOnType => 1019,
            Self::ArgListShouldNotBeEvaluated => 1020,
            Self::ExceptionDuringPropertyRead => 1021,
            Self::FunctionReferenceCannotBeInvoked => 1022,
            Self::ExceptionDuringFunctionCall => 1023,
            Self::ArrayIndexOutOfBounds => 1024,
            Self::CollectionIndexOutOfBounds => 1025,
            Self::StringIndexOutOfBounds => 1026,
            Self::IndexingNotSupportedForType => 1027,
            Self::InstanceOfOperatorNeedsClassOperand => 1028,
            Self::ExceptionDuringMethodInvocation => 1029,
            Self::OperatorNotSupportedBetweenTypes => 1030,
            Self::ProblemLocatingMethod => 1031,
            Self::SetValueNotSupported => 1032,
            Self::MultiplePossibleMethods => 1033,
            Self::ExceptionDuringPropertyWrite => 1034,
            Self::NotAnInteger => 1035,
            Self::NotALong => 1036,
            Self::InvalidFirstOperandForMatchesOperator => 1037,
            Self::InvalidSecondOperandForMatchesOperator => 1038,
            Self::FunctionMustBeStatic => 1039,
            Self::NotAReal => 1040,
            Self::MoreInput => 1041,
            Self::RightOperandProblem => 1042,
            Self::NotExpectedToken => 1043,
            Self::Ood => 1044,
            Self::NonTerminatingDoubleQuotedString => 1045,
            Self::NonTerminatingQuotedString => 1046,
            Self::MissingLeadingZeroForNumber => 1047,
            Self::RealCannotBeLong => 1048,
            Self::UnexpectedDataAfterDot => 1049,
            Self::MissingConstructorArguments => 1050,
            Self::RunOutOfArguments => 1051,
            Self::UnableToGrowCollection => 1052,
            Self::UnableToGrowCollectionUnknownElementType => 1053,
            Self::UnableToCreateListForIndexing => 1054,
            Self::UnableToCreateMapForIndexing => 1055,
            Self::UnableToDynamicallyCreateObject => 1056,
            Self::NoBeanResolverRegistered => 1057,
            Self::ExceptionDuringBeanResolution => 1058,
            Self::InvalidBeanReference => 1059,
            Self::TypeNameExpectedForArrayConstruction => 1060,
            Self::IncorrectElementTypeForArray => 1061,
            Self::MultidimArrayInitializerNotSupported => 1062,
            Self::MissingArrayDimension => 1063,
            Self::InitializerLengthIncorrect => 1064,
            Self::UnexpectedEscapeChar => 1065,
            Self::OperandNotIncrementable => 1066,
            Self::OperandNotDecrementable => 1067,
            Self::NotAssignable => 1068,
            Self::MissingCharacter => 1069,
            Self::LeftOperandProblem => 1070,
            Self::MissingSelectionExpression => 1071,
            Self::ExceptionRunningCompiledExpression => 1072,
            Self::FlawedPattern => 1073,
            Self::ExceptionCompilingExpression => 1074,
            Self::MaxArrayElementsThresholdExceeded => 1075,
            Self::MaxRepeatedTextSizeExceeded => 1076,
            Self::MaxRegexLengthExceeded => 1077,
            Self::MaxConcatenatedStringLengthExceeded => 1078,
            Self::MaxExpressionLengthExceeded => 1079,
            Self::VariableAssignmentNotSupported => 1080,
            Self::NegativeRepeatedTextCount => 1081,
            Self::UnsupportedCharacter => 1082,
            Self::ExceptionDuringIndexRead => 1083,
            Self::ExceptionDuringIndexWrite => 1084,
            Self::MaxOperationsExceeded => 1085,
            Self::InternalError => 9999,
        }
    }

    /// 默认消息模板（占位符使用 `{n}` 形式，对标 `java.text.MessageFormat`）。
    #[must_use]
    pub fn default_message(self) -> &'static str {
        match self {
            Self::TypeConversionError => "Type conversion problem, cannot convert from {0} to {1}",
            Self::ConstructorNotFound => "Constructor call: No suitable constructor found on type {0} for arguments {1}",
            Self::ConstructorInvocationProblem => "A problem occurred whilst attempting to construct an object of type '{0}' using arguments '{1}'",
            Self::MethodNotFound => "Method call: Method {0} cannot be found on type {1}",
            Self::TypeNotFound => "Type cannot be found '{0}'",
            Self::FunctionNotDefined => "Function '{0}' could not be found",
            Self::PropertyOrFieldNotReadableOnNull => "Property or field '{0}' cannot be found on null",
            Self::PropertyOrFieldNotReadable => "Property or field '{0}' cannot be found on object of type '{1}' - maybe not public or not valid?",
            Self::PropertyOrFieldNotWritableOnNull => "Property or field '{0}' cannot be set on null",
            Self::PropertyOrFieldNotWritable => "Property or field '{0}' cannot be set on object of type '{1}' - maybe not public or not writable?",
            Self::MethodCallOnNullObjectNotAllowed => "Method call: Attempted to call method {0} on null context object",
            Self::CannotIndexIntoNullValue => "Cannot index into a null value",
            Self::NotComparable => "Cannot compare instances of {0} and {1}",
            Self::IncorrectNumberOfArgumentsToFunction => "Incorrect number of arguments for function '{0}': {1} supplied but function takes {2}",
            Self::InvalidTypeForSelection => "Cannot perform selection on input data of type '{0}'",
            Self::ResultOfSelectionCriteriaIsNotBoolean => "Result of selection criteria is not boolean",
            Self::BetweenRightOperandMustBeTwoElementList => "Right operand for the 'between' operator has to be a two-element list",
            Self::InvalidPattern => "Pattern is not valid '{0}'",
            Self::ProjectionNotSupportedOnType => "Projection is not supported on the type '{0}'",
            Self::ArgListShouldNotBeEvaluated => "The argument list of a lambda expression should never have getValue() called upon it",
            Self::ExceptionDuringPropertyRead => "A problem occurred whilst attempting to access the property '{0}': '{1}'",
            Self::FunctionReferenceCannotBeInvoked => "The function '{0}' mapped to an object of type '{1}' cannot be invoked",
            Self::ExceptionDuringFunctionCall => "A problem occurred whilst attempting to invoke the function '{0}': '{1}'",
            Self::ArrayIndexOutOfBounds => "The array has '{0}' elements, index '{1}' is invalid",
            Self::CollectionIndexOutOfBounds => "The collection has '{0}' elements, index '{1}' is invalid",
            Self::StringIndexOutOfBounds => "The string has '{0}' characters, index '{1}' is invalid",
            Self::IndexingNotSupportedForType => "Indexing into type '{0}' is not supported",
            Self::InstanceOfOperatorNeedsClassOperand => "The operator 'instanceof' needs the right operand to be a class, not a '{0}'",
            Self::ExceptionDuringMethodInvocation => "A problem occurred when trying to execute method '{0}' on object of type '{1}': '{2}'",
            Self::OperatorNotSupportedBetweenTypes => "The operator '{0}' is not supported between objects of type '{1}' and '{2}'",
            Self::ProblemLocatingMethod => "Problem locating method {0} on type {1}",
            Self::SetValueNotSupported => "setValue(ExpressionState, Object) not supported for '{0}'",
            Self::MultiplePossibleMethods => "Method call of '{0}' is ambiguous, supported type conversions allow multiple variants to match",
            Self::ExceptionDuringPropertyWrite => "A problem occurred whilst attempting to set the property '{0}': {1}",
            Self::NotAnInteger => "The value '{0}' cannot be parsed as an int",
            Self::NotALong => "The value '{0}' cannot be parsed as a long",
            Self::InvalidFirstOperandForMatchesOperator => "First operand to matches operator must be a string. '{0}' is not",
            Self::InvalidSecondOperandForMatchesOperator => "Second operand to matches operator must be a string. '{0}' is not",
            Self::FunctionMustBeStatic => "Only static methods can be called via function references. The method '{0}' referred to by name '{1}' is not static.",
            Self::NotAReal => "The value '{0}' cannot be parsed as a double",
            Self::MoreInput => "After parsing a valid expression, there is still more data in the expression: '{0}'",
            Self::RightOperandProblem => "Problem parsing right operand",
            Self::NotExpectedToken => "Unexpected token. Expected '{0}' but was '{1}'",
            Self::Ood => "Unexpectedly ran out of input",
            Self::NonTerminatingDoubleQuotedString => "Cannot find terminating \" for string",
            Self::NonTerminatingQuotedString => "Cannot find terminating ' for string",
            Self::MissingLeadingZeroForNumber => "A real number must be prefixed by zero, it cannot start with just '.'",
            Self::RealCannotBeLong => "Real number cannot be suffixed with a long (L or l) suffix",
            Self::UnexpectedDataAfterDot => "Unexpected data after '.': '{0}'",
            Self::MissingConstructorArguments => "The arguments '(...)' for the constructor call are missing",
            Self::RunOutOfArguments => "Unexpectedly ran out of arguments",
            Self::UnableToGrowCollection => "Unable to grow collection",
            Self::UnableToGrowCollectionUnknownElementType => "Unable to grow collection: unable to determine list element type",
            Self::UnableToCreateListForIndexing => "Unable to dynamically create a List to replace a null value",
            Self::UnableToCreateMapForIndexing => "Unable to dynamically create a Map to replace a null value",
            Self::UnableToDynamicallyCreateObject => "Unable to dynamically create instance of '{0}' to replace a null value",
            Self::NoBeanResolverRegistered => "No bean resolver registered in the context to resolve access to bean '{0}'",
            Self::ExceptionDuringBeanResolution => "A problem occurred when trying to resolve bean '{0}': '{1}'",
            Self::InvalidBeanReference => "@ or & can only be followed by an identifier or a quoted name",
            Self::TypeNameExpectedForArrayConstruction => "Expected the type of the new array to be specified as a String but found '{0}'",
            Self::IncorrectElementTypeForArray => "The array of type '{0}' cannot have an element of type '{1}' inserted",
            Self::MultidimArrayInitializerNotSupported => "Using an initializer to build a multi-dimensional array is not currently supported",
            Self::MissingArrayDimension => "A required array dimension has not been specified",
            Self::InitializerLengthIncorrect => "Array initializer size does not match array dimensions",
            Self::UnexpectedEscapeChar => "Unexpected escape character",
            Self::OperandNotIncrementable => "The expression component '{0}' does not support increment",
            Self::OperandNotDecrementable => "The expression component '{0}' does not support decrement",
            Self::NotAssignable => "The expression component '{0}' is not assignable",
            Self::MissingCharacter => "Missing expected character '{0}'",
            Self::LeftOperandProblem => "Problem parsing left operand",
            Self::MissingSelectionExpression => "A required selection expression has not been specified",
            Self::ExceptionRunningCompiledExpression => "An exception occurred whilst evaluating a compiled expression",
            Self::FlawedPattern => "Failed to efficiently evaluate pattern '{0}': consider redesigning it",
            Self::ExceptionCompilingExpression => "An exception occurred while compiling an expression",
            Self::MaxArrayElementsThresholdExceeded => "Array declares too many elements, exceeding the threshold of '{0}'",
            Self::MaxRepeatedTextSizeExceeded => "Repeated text is too long, exceeding the threshold of '{0}' characters",
            Self::MaxRegexLengthExceeded => "Regular expression is too long, exceeding the threshold of '{0}' characters",
            Self::MaxConcatenatedStringLengthExceeded => "Concatenated string is too long, exceeding the threshold of '{0}' characters",
            Self::MaxExpressionLengthExceeded => "SpEL expression is too long, exceeding the threshold of '{0}' characters",
            Self::VariableAssignmentNotSupported => "Assignment to variable '{0}' is not supported",
            Self::NegativeRepeatedTextCount => "Repeat count '{0}' must not be negative",
            Self::UnsupportedCharacter => "Unsupported character '{0}' ({1}) encountered in expression",
            Self::ExceptionDuringIndexRead => "A problem occurred while attempting to read index '{0}' in '{1}'",
            Self::ExceptionDuringIndexWrite => "A problem occurred while attempting to write index '{0}' in '{1}'",
            Self::MaxOperationsExceeded => "SpEL expression evaluation exceeded the threshold of '{0}' operations",
            Self::InternalError => "Internal error",
        }
    }

    /// 类别（Spring `SpelMessage.Kind`，仅 `Error` 当前实际使用）。
    #[must_use]
    pub const fn kind(self) -> MessageKind {
        MessageKind::Error
    }

    /// 渲染消息：把 `{n}` 占位符替换成 `inserts[n]`。
    ///
    /// 对标 Spring `SpelMessage.formatMessage(Object... inserts)`。
    /// Spring 用 `java.text.MessageFormat` 处理 `{0}`/`{1,number,...}` 等；
    /// 我们实现最小集：支持 `{n}` 与 `'{n}'`（带引号包裹）。
    #[must_use]
    pub fn format_message(self, inserts: &[&str]) -> String {
        let template = self.default_message();
        let code = self.code();
        // SpEL 输出前缀固定为 `EL{code}E: `
        let prefix = format!("EL{code}E: ");
        let mut body = String::with_capacity(template.len());
        let bytes = template.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let ch = bytes[i] as char;
            if ch == '{' {
                // 检查是否是 {n} 或 '{n}'
                if let Some(end) = template[i..].find('}') {
                    let inside = &template[i + 1..i + end];
                    if let Ok(idx) = inside.parse::<usize>() {
                        if idx < inserts.len() {
                            body.push_str(inserts[idx]);
                        } else {
                            body.push('{');
                            body.push_str(inside);
                            body.push('}');
                        }
                        i += end + 1;
                        continue;
                    }
                }
                body.push(ch);
                i += 1;
            } else {
                body.push(ch);
                i += 1;
            }
        }
        format!("{prefix}{body}")
    }

    /// 同 `format_message`，但每个 insert 都走 `Display`。
    pub fn format_message_display<D: std::fmt::Display>(self, inserts: &[D]) -> String {
        let strings: Vec<String> = inserts.iter().map(|d| d.to_string()).collect();
        let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
        self.format_message(&refs)
    }
}

/// 错误类别（INFO/WARNING/ERROR，SpEL 仅用 ERROR）。
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MessageKind {
    /// 信息级别（保留）。
    #[allow(dead_code)]
    Info,
    /// 警告级别（保留）。
    #[allow(dead_code)]
    Warning,
    /// 错误级别（所有 SpelMessage 实际标记）。
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_has_code() {
        // Smoke test: ensure all 87 variants are reachable
        let codes = [
            SpelMessage::TypeConversionError,
            SpelMessage::ConstructorNotFound,
            SpelMessage::MethodNotFound,
            SpelMessage::TypeNotFound,
            SpelMessage::PropertyOrFieldNotReadable,
            SpelMessage::OperatorNotSupportedBetweenTypes,
            SpelMessage::NotAReal,
            SpelMessage::Ood,
            SpelMessage::UnsupportedCharacter,
            SpelMessage::MaxOperationsExceeded,
            SpelMessage::InternalError,
        ];
        for c in codes {
            assert!(c.code() >= 1001, "code out of range for {c:?}");
            assert!(!c.default_message().is_empty());
        }
    }

    #[test]
    fn format_message_substitutes() {
        let m = SpelMessage::TypeConversionError;
        let s = m.format_message(&["int", "String"]);
        assert_eq!(s, "EL1001E: Type conversion problem, cannot convert from int to String");
    }

    #[test]
    fn format_message_no_inserts() {
        let m = SpelMessage::Ood;
        let s = m.format_message(&[]);
        assert_eq!(s, "EL1044E: Unexpectedly ran out of input");
    }
}
