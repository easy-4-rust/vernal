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

    /// 验证所有 86 个变体都有有效 code 和非空 default_message。
    #[test]
    fn all_variants_have_valid_code_and_message() {
        let all = [
            SpelMessage::TypeConversionError,
            SpelMessage::ConstructorNotFound,
            SpelMessage::ConstructorInvocationProblem,
            SpelMessage::MethodNotFound,
            SpelMessage::TypeNotFound,
            SpelMessage::FunctionNotDefined,
            SpelMessage::PropertyOrFieldNotReadableOnNull,
            SpelMessage::PropertyOrFieldNotReadable,
            SpelMessage::PropertyOrFieldNotWritableOnNull,
            SpelMessage::PropertyOrFieldNotWritable,
            SpelMessage::MethodCallOnNullObjectNotAllowed,
            SpelMessage::CannotIndexIntoNullValue,
            SpelMessage::NotComparable,
            SpelMessage::IncorrectNumberOfArgumentsToFunction,
            SpelMessage::InvalidTypeForSelection,
            SpelMessage::ResultOfSelectionCriteriaIsNotBoolean,
            SpelMessage::BetweenRightOperandMustBeTwoElementList,
            SpelMessage::InvalidPattern,
            SpelMessage::ProjectionNotSupportedOnType,
            SpelMessage::ArgListShouldNotBeEvaluated,
            SpelMessage::ExceptionDuringPropertyRead,
            SpelMessage::FunctionReferenceCannotBeInvoked,
            SpelMessage::ExceptionDuringFunctionCall,
            SpelMessage::ArrayIndexOutOfBounds,
            SpelMessage::CollectionIndexOutOfBounds,
            SpelMessage::StringIndexOutOfBounds,
            SpelMessage::IndexingNotSupportedForType,
            SpelMessage::InstanceOfOperatorNeedsClassOperand,
            SpelMessage::ExceptionDuringMethodInvocation,
            SpelMessage::OperatorNotSupportedBetweenTypes,
            SpelMessage::ProblemLocatingMethod,
            SpelMessage::SetValueNotSupported,
            SpelMessage::MultiplePossibleMethods,
            SpelMessage::ExceptionDuringPropertyWrite,
            SpelMessage::NotAnInteger,
            SpelMessage::NotALong,
            SpelMessage::InvalidFirstOperandForMatchesOperator,
            SpelMessage::InvalidSecondOperandForMatchesOperator,
            SpelMessage::FunctionMustBeStatic,
            SpelMessage::NotAReal,
            SpelMessage::MoreInput,
            SpelMessage::RightOperandProblem,
            SpelMessage::NotExpectedToken,
            SpelMessage::Ood,
            SpelMessage::NonTerminatingDoubleQuotedString,
            SpelMessage::NonTerminatingQuotedString,
            SpelMessage::MissingLeadingZeroForNumber,
            SpelMessage::RealCannotBeLong,
            SpelMessage::UnexpectedDataAfterDot,
            SpelMessage::MissingConstructorArguments,
            SpelMessage::RunOutOfArguments,
            SpelMessage::UnableToGrowCollection,
            SpelMessage::UnableToGrowCollectionUnknownElementType,
            SpelMessage::UnableToCreateListForIndexing,
            SpelMessage::UnableToCreateMapForIndexing,
            SpelMessage::UnableToDynamicallyCreateObject,
            SpelMessage::NoBeanResolverRegistered,
            SpelMessage::ExceptionDuringBeanResolution,
            SpelMessage::InvalidBeanReference,
            SpelMessage::TypeNameExpectedForArrayConstruction,
            SpelMessage::IncorrectElementTypeForArray,
            SpelMessage::MultidimArrayInitializerNotSupported,
            SpelMessage::MissingArrayDimension,
            SpelMessage::InitializerLengthIncorrect,
            SpelMessage::UnexpectedEscapeChar,
            SpelMessage::OperandNotIncrementable,
            SpelMessage::OperandNotDecrementable,
            SpelMessage::NotAssignable,
            SpelMessage::MissingCharacter,
            SpelMessage::LeftOperandProblem,
            SpelMessage::MissingSelectionExpression,
            SpelMessage::ExceptionRunningCompiledExpression,
            SpelMessage::FlawedPattern,
            SpelMessage::ExceptionCompilingExpression,
            SpelMessage::MaxArrayElementsThresholdExceeded,
            SpelMessage::MaxRepeatedTextSizeExceeded,
            SpelMessage::MaxRegexLengthExceeded,
            SpelMessage::MaxConcatenatedStringLengthExceeded,
            SpelMessage::MaxExpressionLengthExceeded,
            SpelMessage::VariableAssignmentNotSupported,
            SpelMessage::NegativeRepeatedTextCount,
            SpelMessage::UnsupportedCharacter,
            SpelMessage::ExceptionDuringIndexRead,
            SpelMessage::ExceptionDuringIndexWrite,
            SpelMessage::MaxOperationsExceeded,
            SpelMessage::InternalError,
        ];
        assert_eq!(all.len(), 86, "expected 86 SpelMessage variants");
        for m in all {
            assert!(m.code() >= 1001, "code out of range for {m:?}");
            assert!(!m.default_message().is_empty(), "empty message for {m:?}");
            assert_eq!(m.kind(), MessageKind::Error);
        }
    }

    /// 验证 code 唯一性（无冲突）。
    #[test]
    fn codes_are_unique() {
        let all_codes = [
            SpelMessage::TypeConversionError.code(),
            SpelMessage::ConstructorNotFound.code(),
            SpelMessage::ConstructorInvocationProblem.code(),
            SpelMessage::MethodNotFound.code(),
            SpelMessage::TypeNotFound.code(),
            SpelMessage::FunctionNotDefined.code(),
            SpelMessage::PropertyOrFieldNotReadableOnNull.code(),
            SpelMessage::PropertyOrFieldNotReadable.code(),
            SpelMessage::PropertyOrFieldNotWritableOnNull.code(),
            SpelMessage::PropertyOrFieldNotWritable.code(),
            SpelMessage::MethodCallOnNullObjectNotAllowed.code(),
            SpelMessage::CannotIndexIntoNullValue.code(),
            SpelMessage::NotComparable.code(),
            SpelMessage::IncorrectNumberOfArgumentsToFunction.code(),
            SpelMessage::InvalidTypeForSelection.code(),
            SpelMessage::ResultOfSelectionCriteriaIsNotBoolean.code(),
            SpelMessage::BetweenRightOperandMustBeTwoElementList.code(),
            SpelMessage::InvalidPattern.code(),
            SpelMessage::ProjectionNotSupportedOnType.code(),
            SpelMessage::ArgListShouldNotBeEvaluated.code(),
            SpelMessage::ExceptionDuringPropertyRead.code(),
            SpelMessage::FunctionReferenceCannotBeInvoked.code(),
            SpelMessage::ExceptionDuringFunctionCall.code(),
            SpelMessage::ArrayIndexOutOfBounds.code(),
            SpelMessage::CollectionIndexOutOfBounds.code(),
            SpelMessage::StringIndexOutOfBounds.code(),
            SpelMessage::IndexingNotSupportedForType.code(),
            SpelMessage::InstanceOfOperatorNeedsClassOperand.code(),
            SpelMessage::ExceptionDuringMethodInvocation.code(),
            SpelMessage::OperatorNotSupportedBetweenTypes.code(),
            SpelMessage::ProblemLocatingMethod.code(),
            SpelMessage::SetValueNotSupported.code(),
            SpelMessage::MultiplePossibleMethods.code(),
            SpelMessage::ExceptionDuringPropertyWrite.code(),
            SpelMessage::NotAnInteger.code(),
            SpelMessage::NotALong.code(),
            SpelMessage::InvalidFirstOperandForMatchesOperator.code(),
            SpelMessage::InvalidSecondOperandForMatchesOperator.code(),
            SpelMessage::FunctionMustBeStatic.code(),
            SpelMessage::NotAReal.code(),
            SpelMessage::MoreInput.code(),
            SpelMessage::RightOperandProblem.code(),
            SpelMessage::NotExpectedToken.code(),
            SpelMessage::Ood.code(),
            SpelMessage::NonTerminatingDoubleQuotedString.code(),
            SpelMessage::NonTerminatingQuotedString.code(),
            SpelMessage::MissingLeadingZeroForNumber.code(),
            SpelMessage::RealCannotBeLong.code(),
            SpelMessage::UnexpectedDataAfterDot.code(),
            SpelMessage::MissingConstructorArguments.code(),
            SpelMessage::RunOutOfArguments.code(),
            SpelMessage::UnableToGrowCollection.code(),
            SpelMessage::UnableToGrowCollectionUnknownElementType.code(),
            SpelMessage::UnableToCreateListForIndexing.code(),
            SpelMessage::UnableToCreateMapForIndexing.code(),
            SpelMessage::UnableToDynamicallyCreateObject.code(),
            SpelMessage::NoBeanResolverRegistered.code(),
            SpelMessage::ExceptionDuringBeanResolution.code(),
            SpelMessage::InvalidBeanReference.code(),
            SpelMessage::TypeNameExpectedForArrayConstruction.code(),
            SpelMessage::IncorrectElementTypeForArray.code(),
            SpelMessage::MultidimArrayInitializerNotSupported.code(),
            SpelMessage::MissingArrayDimension.code(),
            SpelMessage::InitializerLengthIncorrect.code(),
            SpelMessage::UnexpectedEscapeChar.code(),
            SpelMessage::OperandNotIncrementable.code(),
            SpelMessage::OperandNotDecrementable.code(),
            SpelMessage::NotAssignable.code(),
            SpelMessage::MissingCharacter.code(),
            SpelMessage::LeftOperandProblem.code(),
            SpelMessage::MissingSelectionExpression.code(),
            SpelMessage::ExceptionRunningCompiledExpression.code(),
            SpelMessage::FlawedPattern.code(),
            SpelMessage::ExceptionCompilingExpression.code(),
            SpelMessage::MaxArrayElementsThresholdExceeded.code(),
            SpelMessage::MaxRepeatedTextSizeExceeded.code(),
            SpelMessage::MaxRegexLengthExceeded.code(),
            SpelMessage::MaxConcatenatedStringLengthExceeded.code(),
            SpelMessage::MaxExpressionLengthExceeded.code(),
            SpelMessage::VariableAssignmentNotSupported.code(),
            SpelMessage::NegativeRepeatedTextCount.code(),
            SpelMessage::UnsupportedCharacter.code(),
            SpelMessage::ExceptionDuringIndexRead.code(),
            SpelMessage::ExceptionDuringIndexWrite.code(),
            SpelMessage::MaxOperationsExceeded.code(),
            SpelMessage::InternalError.code(),
        ];
        let mut sorted = all_codes.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(all_codes.len(), sorted.len(), "duplicate codes detected");
    }

    // ── format_message 测试 ──────────────────────────────────────────

    #[test]
    fn format_type_conversion_error() {
        let s = SpelMessage::TypeConversionError.format_message(&["int", "String"]);
        assert_eq!(s, "EL1001E: Type conversion problem, cannot convert from int to String");
    }

    #[test]
    fn format_constructor_not_found() {
        let s = SpelMessage::ConstructorNotFound.format_message(&["MyClass", "[String, int]"]);
        assert!(s.starts_with("EL1002E:"));
        assert!(s.contains("MyClass"));
        assert!(s.contains("[String, int]"));
    }

    #[test]
    fn format_method_not_found() {
        let s = SpelMessage::MethodNotFound.format_message(&["foo", "Bar"]);
        assert!(s.starts_with("EL1004E:"));
        assert!(s.contains("foo"));
        assert!(s.contains("Bar"));
    }

    #[test]
    fn format_type_not_found() {
        let s = SpelMessage::TypeNotFound.format_message(&["com.example.Missing"]);
        assert!(s.starts_with("EL1005E:"));
        assert!(s.contains("com.example.Missing"));
    }

    #[test]
    fn format_function_not_defined() {
        let s = SpelMessage::FunctionNotDefined.format_message(&["myFunc"]);
        assert!(s.starts_with("EL1006E:"));
        assert!(s.contains("myFunc"));
    }

    #[test]
    fn format_property_on_null() {
        let s = SpelMessage::PropertyOrFieldNotReadableOnNull.format_message(&["name"]);
        assert!(s.starts_with("EL1007E:"));
        assert!(s.contains("name"));
    }

    #[test]
    fn format_property_not_readable() {
        let s = SpelMessage::PropertyOrFieldNotReadable.format_message(&["age", "Person"]);
        assert!(s.starts_with("EL1008E:"));
        assert!(s.contains("age"));
        assert!(s.contains("Person"));
    }

    #[test]
    fn format_property_not_writable_on_null() {
        let s = SpelMessage::PropertyOrFieldNotWritableOnNull.format_message(&["x"]);
        assert!(s.starts_with("EL1009E:"));
    }

    #[test]
    fn format_property_not_writable() {
        let s = SpelMessage::PropertyOrFieldNotWritable.format_message(&["x", "Foo"]);
        assert!(s.starts_with("EL1010E:"));
    }

    #[test]
    fn format_method_call_on_null() {
        let s = SpelMessage::MethodCallOnNullObjectNotAllowed.format_message(&["toString"]);
        assert!(s.starts_with("EL1011E:"));
    }

    #[test]
    fn format_not_comparable() {
        let s = SpelMessage::NotComparable.format_message(&["String", "Integer"]);
        assert!(s.starts_with("EL1013E:"));
        assert!(s.contains("String"));
        assert!(s.contains("Integer"));
    }

    #[test]
    fn format_between_right_operand() {
        let s = SpelMessage::BetweenRightOperandMustBeTwoElementList.format_message(&[]);
        assert!(s.starts_with("EL1017E:"));
        assert!(s.contains("two-element"));
    }

    #[test]
    fn format_invalid_pattern() {
        let s = SpelMessage::InvalidPattern.format_message(&["[invalid"]);
        assert!(s.starts_with("EL1018E:"));
        assert!(s.contains("[invalid"));
    }

    #[test]
    fn format_array_index_out_of_bounds() {
        let s = SpelMessage::ArrayIndexOutOfBounds.format_message(&["5", "10"]);
        assert!(s.starts_with("EL1024E:"));
        assert!(s.contains("5"));
        assert!(s.contains("10"));
    }

    #[test]
    fn format_collection_index_out_of_bounds() {
        let s = SpelMessage::CollectionIndexOutOfBounds.format_message(&["3", "7"]);
        assert!(s.starts_with("EL1025E:"));
    }

    #[test]
    fn format_string_index_out_of_bounds() {
        let s = SpelMessage::StringIndexOutOfBounds.format_message(&["4", "20"]);
        assert!(s.starts_with("EL1026E:"));
    }

    #[test]
    fn format_indexing_not_supported() {
        let s = SpelMessage::IndexingNotSupportedForType.format_message(&["MyType"]);
        assert!(s.starts_with("EL1027E:"));
    }

    #[test]
    fn format_instanceof_needs_class() {
        let s = SpelMessage::InstanceOfOperatorNeedsClassOperand.format_message(&["String"]);
        assert!(s.starts_with("EL1028E:"));
    }

    #[test]
    fn format_operator_not_supported() {
        let s = SpelMessage::OperatorNotSupportedBetweenTypes.format_message(&["+", "String", "Boolean"]);
        assert!(s.starts_with("EL1030E:"));
        assert!(s.contains("+"));
    }

    #[test]
    fn format_not_an_integer() {
        let s = SpelMessage::NotAnInteger.format_message(&["abc"]);
        assert!(s.starts_with("EL1035E:"));
        assert!(s.contains("abc"));
    }

    #[test]
    fn format_not_a_long() {
        let s = SpelMessage::NotALong.format_message(&["xyz"]);
        assert!(s.starts_with("EL1036E:"));
    }

    #[test]
    fn format_not_a_real() {
        let s = SpelMessage::NotAReal.format_message(&["abc"]);
        assert!(s.starts_with("EL1040E:"));
    }

    #[test]
    fn format_more_input() {
        let s = SpelMessage::MoreInput.format_message(&["extra stuff"]);
        assert!(s.starts_with("EL1041E:"));
        assert!(s.contains("extra stuff"));
    }

    #[test]
    fn format_not_expected_token() {
        let s = SpelMessage::NotExpectedToken.format_message(&["'+'", "'*'"]);
        assert!(s.starts_with("EL1043E:"));
        assert!(s.contains("'+''"));
    }

    #[test]
    fn format_ood() {
        let s = SpelMessage::Ood.format_message(&[]);
        assert_eq!(s, "EL1044E: Unexpectedly ran out of input");
    }

    #[test]
    fn format_non_terminating_double_quoted() {
        let s = SpelMessage::NonTerminatingDoubleQuotedString.format_message(&[]);
        assert!(s.starts_with("EL1045E:"));
    }

    #[test]
    fn format_non_terminating_quoted() {
        let s = SpelMessage::NonTerminatingQuotedString.format_message(&[]);
        assert!(s.starts_with("EL1046E:"));
    }

    #[test]
    fn format_missing_leading_zero() {
        let s = SpelMessage::MissingLeadingZeroForNumber.format_message(&[]);
        assert!(s.starts_with("EL1047E:"));
    }

    #[test]
    fn format_real_cannot_be_long() {
        let s = SpelMessage::RealCannotBeLong.format_message(&[]);
        assert!(s.starts_with("EL1048E:"));
    }

    #[test]
    fn format_unexpected_data_after_dot() {
        let s = SpelMessage::UnexpectedDataAfterDot.format_message(&["abc"]);
        assert!(s.starts_with("EL1049E:"));
    }

    #[test]
    fn format_no_bean_resolver() {
        let s = SpelMessage::NoBeanResolverRegistered.format_message(&["myBean"]);
        assert!(s.starts_with("EL1057E:"));
        assert!(s.contains("myBean"));
    }

    #[test]
    fn format_operand_not_incrementable() {
        let s = SpelMessage::OperandNotIncrementable.format_message(&["literal"]);
        assert!(s.starts_with("EL1066E:"));
    }

    #[test]
    fn format_operand_not_decrementable() {
        let s = SpelMessage::OperandNotDecrementable.format_message(&["literal"]);
        assert!(s.starts_with("EL1067E:"));
    }

    #[test]
    fn format_not_assignable() {
        let s = SpelMessage::NotAssignable.format_message(&["constant"]);
        assert!(s.starts_with("EL1068E:"));
    }

    #[test]
    fn format_missing_character() {
        let s = SpelMessage::MissingCharacter.format_message(&["')'"]);
        assert!(s.starts_with("EL1069E:"));
    }

    #[test]
    fn format_flawed_pattern() {
        let s = SpelMessage::FlawedPattern.format_message(&["(a+)+b"]);
        assert!(s.starts_with("EL1073E:"));
    }

    #[test]
    fn format_max_operations_exceeded() {
        let s = SpelMessage::MaxOperationsExceeded.format_message(&["10000"]);
        assert!(s.starts_with("EL1085E:"));
        assert!(s.contains("10000"));
    }

    #[test]
    fn format_internal_error() {
        let s = SpelMessage::InternalError.format_message(&[]);
        assert_eq!(s, "EL9999E: Internal error");
    }

    #[test]
    fn format_message_display_variant() {
        let s = SpelMessage::TypeConversionError.format_message_display(&["42", "String"]);
        assert!(s.contains("42"));
        assert!(s.contains("String"));
    }

    #[test]
    fn format_message_with_missing_insert_preserves_placeholder() {
        let s = SpelMessage::TypeConversionError.format_message(&["int"]);
        // {1} not provided → kept as {1}
        assert!(s.contains("{1}"));
    }

    #[test]
    fn format_variable_assignment_not_supported() {
        let s = SpelMessage::VariableAssignmentNotSupported.format_message(&["x"]);
        assert!(s.starts_with("EL1080E:"));
    }

    #[test]
    fn format_unsupported_character() {
        let s = SpelMessage::UnsupportedCharacter.format_message(&["@", "0x40"]);
        assert!(s.starts_with("EL1082E:"));
    }

    #[test]
    fn format_exception_during_index_read() {
        let s = SpelMessage::ExceptionDuringIndexRead.format_message(&["0", "list"]);
        assert!(s.starts_with("EL1083E:"));
    }

    #[test]
    fn format_exception_during_index_write() {
        let s = SpelMessage::ExceptionDuringIndexWrite.format_message(&["0", "list"]);
        assert!(s.starts_with("EL1084E:"));
    }

    #[test]
    fn format_incorrect_number_of_arguments() {
        let s = SpelMessage::IncorrectNumberOfArgumentsToFunction.format_message(&["foo", "2", "3"]);
        assert!(s.starts_with("EL1014E:"));
    }

    #[test]
    fn format_invalid_type_for_selection() {
        let s = SpelMessage::InvalidTypeForSelection.format_message(&["Integer"]);
        assert!(s.starts_with("EL1015E:"));
    }

    #[test]
    fn format_result_of_selection_not_boolean() {
        let s = SpelMessage::ResultOfSelectionCriteriaIsNotBoolean.format_message(&[]);
        assert!(s.starts_with("EL1016E:"));
    }

    #[test]
    fn format_projection_not_supported() {
        let s = SpelMessage::ProjectionNotSupportedOnType.format_message(&["Integer"]);
        assert!(s.starts_with("EL1019E:"));
    }

    #[test]
    fn format_exception_during_property_read() {
        let s = SpelMessage::ExceptionDuringPropertyRead.format_message(&["name", "NPE"]);
        assert!(s.starts_with("EL1021E:"));
    }

    #[test]
    fn format_function_reference_cannot_be_invoked() {
        let s = SpelMessage::FunctionReferenceCannotBeInvoked.format_message(&["foo", "String"]);
        assert!(s.starts_with("EL1022E:"));
    }

    #[test]
    fn format_exception_during_function_call() {
        let s = SpelMessage::ExceptionDuringFunctionCall.format_message(&["foo", "error"]);
        assert!(s.starts_with("EL1023E:"));
    }

    #[test]
    fn format_exception_during_method_invocation() {
        let s = SpelMessage::ExceptionDuringMethodInvocation.format_message(&["foo", "Bar", "NPE"]);
        assert!(s.starts_with("EL1029E:"));
    }

    #[test]
    fn format_problem_locating_method() {
        let s = SpelMessage::ProblemLocatingMethod.format_message(&["foo", "Bar"]);
        assert!(s.starts_with("EL1031E:"));
    }

    #[test]
    fn format_set_value_not_supported() {
        let s = SpelMessage::SetValueNotSupported.format_message(&["Indexer"]);
        assert!(s.starts_with("EL1032E:"));
    }

    #[test]
    fn format_multiple_possible_methods() {
        let s = SpelMessage::MultiplePossibleMethods.format_message(&["foo"]);
        assert!(s.starts_with("EL1033E:"));
    }

    #[test]
    fn format_exception_during_property_write() {
        let s = SpelMessage::ExceptionDuringPropertyWrite.format_message(&["name", "read-only"]);
        assert!(s.starts_with("EL1034E:"));
    }

    #[test]
    fn format_invalid_first_operand_for_matches() {
        let s = SpelMessage::InvalidFirstOperandForMatchesOperator.format_message(&["42"]);
        assert!(s.starts_with("EL1037E:"));
    }

    #[test]
    fn format_invalid_second_operand_for_matches() {
        let s = SpelMessage::InvalidSecondOperandForMatchesOperator.format_message(&["42"]);
        assert!(s.starts_with("EL1038E:"));
    }

    #[test]
    fn format_function_must_be_static() {
        let s = SpelMessage::FunctionMustBeStatic.format_message(&["foo", "bar"]);
        assert!(s.starts_with("EL1039E:"));
    }

    #[test]
    fn format_right_operand_problem() {
        let s = SpelMessage::RightOperandProblem.format_message(&[]);
        assert!(s.starts_with("EL1042E:"));
    }

    #[test]
    fn format_missing_constructor_arguments() {
        let s = SpelMessage::MissingConstructorArguments.format_message(&[]);
        assert!(s.starts_with("EL1050E:"));
    }

    #[test]
    fn format_run_out_of_arguments() {
        let s = SpelMessage::RunOutOfArguments.format_message(&[]);
        assert!(s.starts_with("EL1051E:"));
    }

    #[test]
    fn format_unable_to_grow_collection() {
        let s = SpelMessage::UnableToGrowCollection.format_message(&[]);
        assert!(s.starts_with("EL1052E:"));
    }

    #[test]
    fn format_unable_to_grow_collection_unknown_element() {
        let s = SpelMessage::UnableToGrowCollectionUnknownElementType.format_message(&[]);
        assert!(s.starts_with("EL1053E:"));
    }

    #[test]
    fn format_unable_to_create_list() {
        let s = SpelMessage::UnableToCreateListForIndexing.format_message(&[]);
        assert!(s.starts_with("EL1054E:"));
    }

    #[test]
    fn format_unable_to_create_map() {
        let s = SpelMessage::UnableToCreateMapForIndexing.format_message(&[]);
        assert!(s.starts_with("EL1055E:"));
    }

    #[test]
    fn format_unable_to_dynamically_create_object() {
        let s = SpelMessage::UnableToDynamicallyCreateObject.format_message(&["Foo"]);
        assert!(s.starts_with("EL1056E:"));
    }

    #[test]
    fn format_exception_during_bean_resolution() {
        let s = SpelMessage::ExceptionDuringBeanResolution.format_message(&["myBean", "not found"]);
        assert!(s.starts_with("EL1058E:"));
    }

    #[test]
    fn format_invalid_bean_reference() {
        let s = SpelMessage::InvalidBeanReference.format_message(&[]);
        assert!(s.starts_with("EL1059E:"));
    }

    #[test]
    fn format_type_name_expected_for_array() {
        let s = SpelMessage::TypeNameExpectedForArrayConstruction.format_message(&["null"]);
        assert!(s.starts_with("EL1060E:"));
    }

    #[test]
    fn format_incorrect_element_type_for_array() {
        let s = SpelMessage::IncorrectElementTypeForArray.format_message(&["int[]", "String"]);
        assert!(s.starts_with("EL1061E:"));
    }

    #[test]
    fn format_multidim_array_not_supported() {
        let s = SpelMessage::MultidimArrayInitializerNotSupported.format_message(&[]);
        assert!(s.starts_with("EL1062E:"));
    }

    #[test]
    fn format_missing_array_dimension() {
        let s = SpelMessage::MissingArrayDimension.format_message(&[]);
        assert!(s.starts_with("EL1063E:"));
    }

    #[test]
    fn format_initializer_length_incorrect() {
        let s = SpelMessage::InitializerLengthIncorrect.format_message(&[]);
        assert!(s.starts_with("EL1064E:"));
    }

    #[test]
    fn format_unexpected_escape_char() {
        let s = SpelMessage::UnexpectedEscapeChar.format_message(&[]);
        assert!(s.starts_with("EL1065E:"));
    }

    #[test]
    fn format_missing_selection_expression() {
        let s = SpelMessage::MissingSelectionExpression.format_message(&[]);
        assert!(s.starts_with("EL1071E:"));
    }

    #[test]
    fn format_max_array_elements() {
        let s = SpelMessage::MaxArrayElementsThresholdExceeded.format_message(&["1000"]);
        assert!(s.starts_with("EL1075E:"));
    }

    #[test]
    fn format_max_repeated_text_size() {
        let s = SpelMessage::MaxRepeatedTextSizeExceeded.format_message(&["10000"]);
        assert!(s.starts_with("EL1076E:"));
    }

    #[test]
    fn format_max_regex_length() {
        let s = SpelMessage::MaxRegexLengthExceeded.format_message(&["5000"]);
        assert!(s.starts_with("EL1077E:"));
    }

    #[test]
    fn format_max_concatenated_string_length() {
        let s = SpelMessage::MaxConcatenatedStringLengthExceeded.format_message(&["10000"]);
        assert!(s.starts_with("EL1078E:"));
    }

    #[test]
    fn format_max_expression_length() {
        let s = SpelMessage::MaxExpressionLengthExceeded.format_message(&["10000"]);
        assert!(s.starts_with("EL1079E:"));
    }

    #[test]
    fn format_negative_repeated_text_count() {
        let s = SpelMessage::NegativeRepeatedTextCount.format_message(&["-5"]);
        assert!(s.starts_with("EL1081E:"));
    }

    // ── Debug / Copy / Clone / PartialEq ──────────────────────────────

    #[test]
    fn debug_format() {
        assert_eq!(format!("{:?}", SpelMessage::Ood), "Ood");
    }

    #[test]
    fn copy_clone() {
        let m = SpelMessage::TypeConversionError;
        let m2 = m;
        let m3 = m.clone();
        assert_eq!(m, m2);
        assert_eq!(m, m3);
    }

    #[test]
    fn partial_eq() {
        assert_eq!(SpelMessage::Ood, SpelMessage::Ood);
        assert_ne!(SpelMessage::Ood, SpelMessage::InternalError);
    }

    #[test]
    fn message_kind_is_error() {
        assert_eq!(SpelMessage::TypeConversionError.kind(), MessageKind::Error);
        assert_eq!(SpelMessage::InternalError.kind(), MessageKind::Error);
    }
}
