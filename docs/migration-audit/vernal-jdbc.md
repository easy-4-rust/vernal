<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-jdbc 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 236 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 236 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 0 |
| `MISSING` | 236 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 0 |

## 结构红线

- 未发现 `lib.rs`/`mod.rs` 类型定义或生产 wildcard import。

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.jdbc.BadSqlGrammarException` | `BadSqlGrammarException.java` | `bad_sql_grammar_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.CannotGetJdbcConnectionException` | `CannotGetJdbcConnectionException.java` | `cannot_get_jdbc_connection_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.IncorrectResultSetColumnCountException` | `IncorrectResultSetColumnCountException.java` | `incorrect_result_set_column_count_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.InvalidResultSetAccessException` | `InvalidResultSetAccessException.java` | `invalid_result_set_access_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.JdbcUpdateAffectedIncorrectNumberOfRowsException` | `JdbcUpdateAffectedIncorrectNumberOfRowsException.java` | `jdbc_update_affected_incorrect_number_of_rows_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.LobRetrievalFailureException` | `LobRetrievalFailureException.java` | `lob_retrieval_failure_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.SQLWarningException` | `SQLWarningException.java` | `sql_warning_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.UncategorizedSQLException` | `UncategorizedSQLException.java` | `uncategorized_sql_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.config.DatabasePopulatorConfigUtils` | `config/DatabasePopulatorConfigUtils.java` | `config/database_populator_config_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.config.EmbeddedDatabaseBeanDefinitionParser` | `config/EmbeddedDatabaseBeanDefinitionParser.java` | `config/embedded_database_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.config.InitializeDatabaseBeanDefinitionParser` | `config/InitializeDatabaseBeanDefinitionParser.java` | `config/initialize_database_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.config.JdbcNamespaceHandler` | `config/JdbcNamespaceHandler.java` | `config/jdbc_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.config.SortedResourcesFactoryBean` | `config/SortedResourcesFactoryBean.java` | `config/sorted_resources_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.AggregatedBatchUpdateException` | `core/AggregatedBatchUpdateException.java` | `core/aggregated_batch_update_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ArgumentPreparedStatementSetter` | `core/ArgumentPreparedStatementSetter.java` | `core/argument_prepared_statement_setter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ArgumentTypePreparedStatementSetter` | `core/ArgumentTypePreparedStatementSetter.java` | `core/argument_type_prepared_statement_setter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.BatchPreparedStatementSetter` | `core/BatchPreparedStatementSetter.java` | `core/batch_prepared_statement_setter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.BeanPropertyRowMapper` | `core/BeanPropertyRowMapper.java` | `core/bean_property_row_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.CallableStatementCallback` | `core/CallableStatementCallback.java` | `core/callable_statement_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.CallableStatementCreator` | `core/CallableStatementCreator.java` | `core/callable_statement_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.CallableStatementCreatorFactory` | `core/CallableStatementCreatorFactory.java` | `core/callable_statement_creator_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ColumnMapRowMapper` | `core/ColumnMapRowMapper.java` | `core/column_map_row_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ConnectionCallback` | `core/ConnectionCallback.java` | `core/connection_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.DataClassRowMapper` | `core/DataClassRowMapper.java` | `core/data_class_row_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.DisposableSqlTypeValue` | `core/DisposableSqlTypeValue.java` | `core/disposable_sql_type_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.InterruptibleBatchPreparedStatementSetter` | `core/InterruptibleBatchPreparedStatementSetter.java` | `core/interruptible_batch_prepared_statement_setter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.JdbcOperations` | `core/JdbcOperations.java` | `core/jdbc_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.JdbcTemplate` | `core/JdbcTemplate.java` | `core/jdbc_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ParameterDisposer` | `core/ParameterDisposer.java` | `core/parameter_disposer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ParameterMapper` | `core/ParameterMapper.java` | `core/parameter_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ParameterizedPreparedStatementSetter` | `core/ParameterizedPreparedStatementSetter.java` | `core/parameterized_prepared_statement_setter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.PreparedStatementCallback` | `core/PreparedStatementCallback.java` | `core/prepared_statement_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.PreparedStatementCreator` | `core/PreparedStatementCreator.java` | `core/prepared_statement_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.PreparedStatementCreatorFactory` | `core/PreparedStatementCreatorFactory.java` | `core/prepared_statement_creator_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.PreparedStatementSetter` | `core/PreparedStatementSetter.java` | `core/prepared_statement_setter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ResultSetExtractor` | `core/ResultSetExtractor.java` | `core/result_set_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.ResultSetSupportingSqlParameter` | `core/ResultSetSupportingSqlParameter.java` | `core/result_set_supporting_sql_parameter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.RowCallbackHandler` | `core/RowCallbackHandler.java` | `core/row_callback_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.RowCountCallbackHandler` | `core/RowCountCallbackHandler.java` | `core/row_count_callback_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.RowMapper` | `core/RowMapper.java` | `core/row_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.RowMapperResultSetExtractor` | `core/RowMapperResultSetExtractor.java` | `core/row_mapper_result_set_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SimplePropertyRowMapper` | `core/SimplePropertyRowMapper.java` | `core/simple_property_row_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SingleColumnRowMapper` | `core/SingleColumnRowMapper.java` | `core/single_column_row_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlInOutParameter` | `core/SqlInOutParameter.java` | `core/sql_in_out_parameter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlOutParameter` | `core/SqlOutParameter.java` | `core/sql_out_parameter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlParameter` | `core/SqlParameter.java` | `core/sql_parameter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlParameterValue` | `core/SqlParameterValue.java` | `core/sql_parameter_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlProvider` | `core/SqlProvider.java` | `core/sql_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlReturnResultSet` | `core/SqlReturnResultSet.java` | `core/sql_return_result_set.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlReturnType` | `core/SqlReturnType.java` | `core/sql_return_type.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlReturnUpdateCount` | `core/SqlReturnUpdateCount.java` | `core/sql_return_update_count.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlRowSetResultSetExtractor` | `core/SqlRowSetResultSetExtractor.java` | `core/sql_row_set_result_set_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.SqlTypeValue` | `core/SqlTypeValue.java` | `core/sql_type_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.StatementCallback` | `core/StatementCallback.java` | `core/statement_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.StatementCreatorUtils` | `core/StatementCreatorUtils.java` | `core/statement_creator_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.CallMetaDataContext` | `core/metadata/CallMetaDataContext.java` | `core/metadata/call_meta_data_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.CallMetaDataProvider` | `core/metadata/CallMetaDataProvider.java` | `core/metadata/call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.CallMetaDataProviderFactory` | `core/metadata/CallMetaDataProviderFactory.java` | `core/metadata/call_meta_data_provider_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.CallParameterMetaData` | `core/metadata/CallParameterMetaData.java` | `core/metadata/call_parameter_meta_data.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.Db2CallMetaDataProvider` | `core/metadata/Db2CallMetaDataProvider.java` | `core/metadata/db2_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.DerbyCallMetaDataProvider` | `core/metadata/DerbyCallMetaDataProvider.java` | `core/metadata/derby_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.DerbyTableMetaDataProvider` | `core/metadata/DerbyTableMetaDataProvider.java` | `core/metadata/derby_table_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.GenericCallMetaDataProvider` | `core/metadata/GenericCallMetaDataProvider.java` | `core/metadata/generic_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.GenericTableMetaDataProvider` | `core/metadata/GenericTableMetaDataProvider.java` | `core/metadata/generic_table_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.HanaCallMetaDataProvider` | `core/metadata/HanaCallMetaDataProvider.java` | `core/metadata/hana_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.HsqlTableMetaDataProvider` | `core/metadata/HsqlTableMetaDataProvider.java` | `core/metadata/hsql_table_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.MySQLTableMetaDataProvider` | `core/metadata/MySQLTableMetaDataProvider.java` | `core/metadata/my_sql_table_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.OracleCallMetaDataProvider` | `core/metadata/OracleCallMetaDataProvider.java` | `core/metadata/oracle_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.OracleTableMetaDataProvider` | `core/metadata/OracleTableMetaDataProvider.java` | `core/metadata/oracle_table_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.PostgresCallMetaDataProvider` | `core/metadata/PostgresCallMetaDataProvider.java` | `core/metadata/postgres_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.PostgresTableMetaDataProvider` | `core/metadata/PostgresTableMetaDataProvider.java` | `core/metadata/postgres_table_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.SqlServerCallMetaDataProvider` | `core/metadata/SqlServerCallMetaDataProvider.java` | `core/metadata/sql_server_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.SybaseCallMetaDataProvider` | `core/metadata/SybaseCallMetaDataProvider.java` | `core/metadata/sybase_call_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.TableMetaDataContext` | `core/metadata/TableMetaDataContext.java` | `core/metadata/table_meta_data_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.TableMetaDataProvider` | `core/metadata/TableMetaDataProvider.java` | `core/metadata/table_meta_data_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.TableMetaDataProviderFactory` | `core/metadata/TableMetaDataProviderFactory.java` | `core/metadata/table_meta_data_provider_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.metadata.TableParameterMetaData` | `core/metadata/TableParameterMetaData.java` | `core/metadata/table_parameter_meta_data.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.AbstractSqlParameterSource` | `core/namedparam/AbstractSqlParameterSource.java` | `core/namedparam/abstract_sql_parameter_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.BeanPropertySqlParameterSource` | `core/namedparam/BeanPropertySqlParameterSource.java` | `core/namedparam/bean_property_sql_parameter_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.EmptySqlParameterSource` | `core/namedparam/EmptySqlParameterSource.java` | `core/namedparam/empty_sql_parameter_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.MapSqlParameterSource` | `core/namedparam/MapSqlParameterSource.java` | `core/namedparam/map_sql_parameter_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.NamedParameterJdbcDaoSupport` | `core/namedparam/NamedParameterJdbcDaoSupport.java` | `core/namedparam/named_parameter_jdbc_dao_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.NamedParameterJdbcOperations` | `core/namedparam/NamedParameterJdbcOperations.java` | `core/namedparam/named_parameter_jdbc_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.NamedParameterJdbcTemplate` | `core/namedparam/NamedParameterJdbcTemplate.java` | `core/namedparam/named_parameter_jdbc_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.NamedParameterUtils` | `core/namedparam/NamedParameterUtils.java` | `core/namedparam/named_parameter_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.ParsedSql` | `core/namedparam/ParsedSql.java` | `core/namedparam/parsed_sql.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.SimplePropertySqlParameterSource` | `core/namedparam/SimplePropertySqlParameterSource.java` | `core/namedparam/simple_property_sql_parameter_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.SqlParameterSource` | `core/namedparam/SqlParameterSource.java` | `core/namedparam/sql_parameter_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.namedparam.SqlParameterSourceUtils` | `core/namedparam/SqlParameterSourceUtils.java` | `core/namedparam/sql_parameter_source_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.AbstractJdbcCall` | `core/simple/AbstractJdbcCall.java` | `core/simple/abstract_jdbc_call.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.AbstractJdbcInsert` | `core/simple/AbstractJdbcInsert.java` | `core/simple/abstract_jdbc_insert.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.DefaultJdbcClient` | `core/simple/DefaultJdbcClient.java` | `core/simple/default_jdbc_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.JdbcClient` | `core/simple/JdbcClient.java` | `core/simple/jdbc_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.SimpleJdbcCall` | `core/simple/SimpleJdbcCall.java` | `core/simple/simple_jdbc_call.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.SimpleJdbcCallOperations` | `core/simple/SimpleJdbcCallOperations.java` | `core/simple/simple_jdbc_call_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.SimpleJdbcInsert` | `core/simple/SimpleJdbcInsert.java` | `core/simple/simple_jdbc_insert.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.simple.SimpleJdbcInsertOperations` | `core/simple/SimpleJdbcInsertOperations.java` | `core/simple/simple_jdbc_insert_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.AbstractInterruptibleBatchPreparedStatementSetter` | `core/support/AbstractInterruptibleBatchPreparedStatementSetter.java` | `core/support/abstract_interruptible_batch_prepared_statement_setter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.AbstractLobCreatingPreparedStatementCallback` | `core/support/AbstractLobCreatingPreparedStatementCallback.java` | `core/support/abstract_lob_creating_prepared_statement_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.AbstractLobStreamingResultSetExtractor` | `core/support/AbstractLobStreamingResultSetExtractor.java` | `core/support/abstract_lob_streaming_result_set_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.AbstractSqlTypeValue` | `core/support/AbstractSqlTypeValue.java` | `core/support/abstract_sql_type_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.JdbcBeanDefinitionReader` | `core/support/JdbcBeanDefinitionReader.java` | `core/support/jdbc_bean_definition_reader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.JdbcDaoSupport` | `core/support/JdbcDaoSupport.java` | `core/support/jdbc_dao_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.SqlBinaryValue` | `core/support/SqlBinaryValue.java` | `core/support/sql_binary_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.SqlCharacterValue` | `core/support/SqlCharacterValue.java` | `core/support/sql_character_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.core.support.SqlLobValue` | `core/support/SqlLobValue.java` | `core/support/sql_lob_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.AbstractDataSource` | `datasource/AbstractDataSource.java` | `datasource/abstract_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.AbstractDriverBasedDataSource` | `datasource/AbstractDriverBasedDataSource.java` | `datasource/abstract_driver_based_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.ConnectionHandle` | `datasource/ConnectionHandle.java` | `datasource/connection_handle.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.ConnectionHolder` | `datasource/ConnectionHolder.java` | `datasource/connection_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.ConnectionProxy` | `datasource/ConnectionProxy.java` | `datasource/connection_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.DataSourceTransactionManager` | `datasource/DataSourceTransactionManager.java` | `datasource/data_source_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.DataSourceUtils` | `datasource/DataSourceUtils.java` | `datasource/data_source_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.DelegatingDataSource` | `datasource/DelegatingDataSource.java` | `datasource/delegating_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.DriverManagerDataSource` | `datasource/DriverManagerDataSource.java` | `datasource/driver_manager_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.IsolationLevelDataSourceAdapter` | `datasource/IsolationLevelDataSourceAdapter.java` | `datasource/isolation_level_data_source_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.JdbcTransactionObjectSupport` | `datasource/JdbcTransactionObjectSupport.java` | `datasource/jdbc_transaction_object_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.LazyConnectionDataSourceProxy` | `datasource/LazyConnectionDataSourceProxy.java` | `datasource/lazy_connection_data_source_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.ShardingKeyDataSourceAdapter` | `datasource/ShardingKeyDataSourceAdapter.java` | `datasource/sharding_key_data_source_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.ShardingKeyProvider` | `datasource/ShardingKeyProvider.java` | `datasource/sharding_key_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.SimpleConnectionHandle` | `datasource/SimpleConnectionHandle.java` | `datasource/simple_connection_handle.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.SimpleDriverDataSource` | `datasource/SimpleDriverDataSource.java` | `datasource/simple_driver_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.SingleConnectionDataSource` | `datasource/SingleConnectionDataSource.java` | `datasource/single_connection_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.SmartDataSource` | `datasource/SmartDataSource.java` | `datasource/smart_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.TransactionAwareDataSourceProxy` | `datasource/TransactionAwareDataSourceProxy.java` | `datasource/transaction_aware_data_source_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.UserCredentialsDataSourceAdapter` | `datasource/UserCredentialsDataSourceAdapter.java` | `datasource/user_credentials_data_source_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.AbstractEmbeddedDatabaseConfigurer` | `datasource/embedded/AbstractEmbeddedDatabaseConfigurer.java` | `datasource/embedded/abstract_embedded_database_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.ConnectionProperties` | `datasource/embedded/ConnectionProperties.java` | `datasource/embedded/connection_properties.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.DataSourceFactory` | `datasource/embedded/DataSourceFactory.java` | `datasource/embedded/data_source_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.DerbyEmbeddedDatabaseConfigurer` | `datasource/embedded/DerbyEmbeddedDatabaseConfigurer.java` | `datasource/embedded/derby_embedded_database_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabase` | `datasource/embedded/EmbeddedDatabase.java` | `datasource/embedded/embedded_database.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseBuilder` | `datasource/embedded/EmbeddedDatabaseBuilder.java` | `datasource/embedded/embedded_database_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseConfigurer` | `datasource/embedded/EmbeddedDatabaseConfigurer.java` | `datasource/embedded/embedded_database_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseConfigurerDelegate` | `datasource/embedded/EmbeddedDatabaseConfigurerDelegate.java` | `datasource/embedded/embedded_database_configurer_delegate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseConfigurers` | `datasource/embedded/EmbeddedDatabaseConfigurers.java` | `datasource/embedded/embedded_database_configurers.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseFactory` | `datasource/embedded/EmbeddedDatabaseFactory.java` | `datasource/embedded/embedded_database_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseFactoryBean` | `datasource/embedded/EmbeddedDatabaseFactoryBean.java` | `datasource/embedded/embedded_database_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseFactoryRuntimeHints` | `datasource/embedded/EmbeddedDatabaseFactoryRuntimeHints.java` | `datasource/embedded/embedded_database_factory_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.EmbeddedDatabaseType` | `datasource/embedded/EmbeddedDatabaseType.java` | `datasource/embedded/embedded_database_type.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.H2EmbeddedDatabaseConfigurer` | `datasource/embedded/H2EmbeddedDatabaseConfigurer.java` | `datasource/embedded/h2_embedded_database_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.HsqlEmbeddedDatabaseConfigurer` | `datasource/embedded/HsqlEmbeddedDatabaseConfigurer.java` | `datasource/embedded/hsql_embedded_database_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.OutputStreamFactory` | `datasource/embedded/OutputStreamFactory.java` | `datasource/embedded/output_stream_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.embedded.SimpleDriverDataSourceFactory` | `datasource/embedded/SimpleDriverDataSourceFactory.java` | `datasource/embedded/simple_driver_data_source_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.CannotReadScriptException` | `datasource/init/CannotReadScriptException.java` | `datasource/init/cannot_read_script_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.CompositeDatabasePopulator` | `datasource/init/CompositeDatabasePopulator.java` | `datasource/init/composite_database_populator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.DataSourceInitializer` | `datasource/init/DataSourceInitializer.java` | `datasource/init/data_source_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.DatabasePopulator` | `datasource/init/DatabasePopulator.java` | `datasource/init/database_populator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.DatabasePopulatorUtils` | `datasource/init/DatabasePopulatorUtils.java` | `datasource/init/database_populator_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.ResourceDatabasePopulator` | `datasource/init/ResourceDatabasePopulator.java` | `datasource/init/resource_database_populator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.ScriptException` | `datasource/init/ScriptException.java` | `datasource/init/script_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.ScriptParseException` | `datasource/init/ScriptParseException.java` | `datasource/init/script_parse_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.ScriptStatementFailedException` | `datasource/init/ScriptStatementFailedException.java` | `datasource/init/script_statement_failed_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.ScriptUtils` | `datasource/init/ScriptUtils.java` | `datasource/init/script_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.init.UncategorizedScriptException` | `datasource/init/UncategorizedScriptException.java` | `datasource/init/uncategorized_script_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.AbstractRoutingDataSource` | `datasource/lookup/AbstractRoutingDataSource.java` | `datasource/lookup/abstract_routing_data_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.BeanFactoryDataSourceLookup` | `datasource/lookup/BeanFactoryDataSourceLookup.java` | `datasource/lookup/bean_factory_data_source_lookup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.DataSourceLookup` | `datasource/lookup/DataSourceLookup.java` | `datasource/lookup/data_source_lookup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.DataSourceLookupFailureException` | `datasource/lookup/DataSourceLookupFailureException.java` | `datasource/lookup/data_source_lookup_failure_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.IsolationLevelDataSourceRouter` | `datasource/lookup/IsolationLevelDataSourceRouter.java` | `datasource/lookup/isolation_level_data_source_router.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.JndiDataSourceLookup` | `datasource/lookup/JndiDataSourceLookup.java` | `datasource/lookup/jndi_data_source_lookup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.MapDataSourceLookup` | `datasource/lookup/MapDataSourceLookup.java` | `datasource/lookup/map_data_source_lookup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.datasource.lookup.SingleDataSourceLookup` | `datasource/lookup/SingleDataSourceLookup.java` | `datasource/lookup/single_data_source_lookup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.BatchSqlUpdate` | `object/BatchSqlUpdate.java` | `object/batch_sql_update.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.GenericSqlQuery` | `object/GenericSqlQuery.java` | `object/generic_sql_query.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.GenericStoredProcedure` | `object/GenericStoredProcedure.java` | `object/generic_stored_procedure.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.MappingSqlQuery` | `object/MappingSqlQuery.java` | `object/mapping_sql_query.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.MappingSqlQueryWithParameters` | `object/MappingSqlQueryWithParameters.java` | `object/mapping_sql_query_with_parameters.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.RdbmsOperation` | `object/RdbmsOperation.java` | `object/rdbms_operation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.SqlCall` | `object/SqlCall.java` | `object/sql_call.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.SqlFunction` | `object/SqlFunction.java` | `object/sql_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.SqlOperation` | `object/SqlOperation.java` | `object/sql_operation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.SqlQuery` | `object/SqlQuery.java` | `object/sql_query.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.SqlUpdate` | `object/SqlUpdate.java` | `object/sql_update.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.StoredProcedure` | `object/StoredProcedure.java` | `object/stored_procedure.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.object.UpdatableSqlQuery` | `object/UpdatableSqlQuery.java` | `object/updatable_sql_query.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.AbstractFallbackSQLExceptionTranslator` | `support/AbstractFallbackSQLExceptionTranslator.java` | `support/abstract_fallback_sql_exception_translator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.CustomSQLErrorCodesTranslation` | `support/CustomSQLErrorCodesTranslation.java` | `support/custom_sql_error_codes_translation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.CustomSQLExceptionTranslatorRegistrar` | `support/CustomSQLExceptionTranslatorRegistrar.java` | `support/custom_sql_exception_translator_registrar.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.CustomSQLExceptionTranslatorRegistry` | `support/CustomSQLExceptionTranslatorRegistry.java` | `support/custom_sql_exception_translator_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.DatabaseMetaDataCallback` | `support/DatabaseMetaDataCallback.java` | `support/database_meta_data_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.DatabaseStartupValidator` | `support/DatabaseStartupValidator.java` | `support/database_startup_validator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.GeneratedKeyHolder` | `support/GeneratedKeyHolder.java` | `support/generated_key_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.JdbcAccessor` | `support/JdbcAccessor.java` | `support/jdbc_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.JdbcTransactionManager` | `support/JdbcTransactionManager.java` | `support/jdbc_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.JdbcUtils` | `support/JdbcUtils.java` | `support/jdbc_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.JdbcUtilsRuntimeHints` | `support/JdbcUtilsRuntimeHints.java` | `support/jdbc_utils_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.KeyHolder` | `support/KeyHolder.java` | `support/key_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.MetaDataAccessException` | `support/MetaDataAccessException.java` | `support/meta_data_access_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SQLErrorCodeSQLExceptionTranslator` | `support/SQLErrorCodeSQLExceptionTranslator.java` | `support/sql_error_code_sql_exception_translator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SQLErrorCodes` | `support/SQLErrorCodes.java` | `support/sql_error_codes.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SQLErrorCodesFactory` | `support/SQLErrorCodesFactory.java` | `support/sql_error_codes_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SQLExceptionSubclassTranslator` | `support/SQLExceptionSubclassTranslator.java` | `support/sql_exception_subclass_translator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SQLExceptionTranslator` | `support/SQLExceptionTranslator.java` | `support/sql_exception_translator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SQLStateSQLExceptionTranslator` | `support/SQLStateSQLExceptionTranslator.java` | `support/sql_state_sql_exception_translator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SqlArrayValue` | `support/SqlArrayValue.java` | `support/sql_array_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.SqlValue` | `support/SqlValue.java` | `support/sql_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.AbstractColumnMaxValueIncrementer` | `support/incrementer/AbstractColumnMaxValueIncrementer.java` | `support/incrementer/abstract_column_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.AbstractDataFieldMaxValueIncrementer` | `support/incrementer/AbstractDataFieldMaxValueIncrementer.java` | `support/incrementer/abstract_data_field_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.AbstractIdentityColumnMaxValueIncrementer` | `support/incrementer/AbstractIdentityColumnMaxValueIncrementer.java` | `support/incrementer/abstract_identity_column_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.AbstractSequenceMaxValueIncrementer` | `support/incrementer/AbstractSequenceMaxValueIncrementer.java` | `support/incrementer/abstract_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.DataFieldMaxValueIncrementer` | `support/incrementer/DataFieldMaxValueIncrementer.java` | `support/incrementer/data_field_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.Db2LuwMaxValueIncrementer` | `support/incrementer/Db2LuwMaxValueIncrementer.java` | `support/incrementer/db2_luw_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.Db2MainframeMaxValueIncrementer` | `support/incrementer/Db2MainframeMaxValueIncrementer.java` | `support/incrementer/db2_mainframe_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.DerbyMaxValueIncrementer` | `support/incrementer/DerbyMaxValueIncrementer.java` | `support/incrementer/derby_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.H2SequenceMaxValueIncrementer` | `support/incrementer/H2SequenceMaxValueIncrementer.java` | `support/incrementer/h2_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.HanaSequenceMaxValueIncrementer` | `support/incrementer/HanaSequenceMaxValueIncrementer.java` | `support/incrementer/hana_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.HsqlMaxValueIncrementer` | `support/incrementer/HsqlMaxValueIncrementer.java` | `support/incrementer/hsql_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.HsqlSequenceMaxValueIncrementer` | `support/incrementer/HsqlSequenceMaxValueIncrementer.java` | `support/incrementer/hsql_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.MariaDBSequenceMaxValueIncrementer` | `support/incrementer/MariaDBSequenceMaxValueIncrementer.java` | `support/incrementer/maria_db_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.MySQLIdentityColumnMaxValueIncrementer` | `support/incrementer/MySQLIdentityColumnMaxValueIncrementer.java` | `support/incrementer/my_sql_identity_column_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.MySQLMaxValueIncrementer` | `support/incrementer/MySQLMaxValueIncrementer.java` | `support/incrementer/my_sql_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.OracleSequenceMaxValueIncrementer` | `support/incrementer/OracleSequenceMaxValueIncrementer.java` | `support/incrementer/oracle_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.PostgresSequenceMaxValueIncrementer` | `support/incrementer/PostgresSequenceMaxValueIncrementer.java` | `support/incrementer/postgres_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.SqlServerMaxValueIncrementer` | `support/incrementer/SqlServerMaxValueIncrementer.java` | `support/incrementer/sql_server_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.SqlServerSequenceMaxValueIncrementer` | `support/incrementer/SqlServerSequenceMaxValueIncrementer.java` | `support/incrementer/sql_server_sequence_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.SqliteMaxValueIncrementer` | `support/incrementer/SqliteMaxValueIncrementer.java` | `support/incrementer/sqlite_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.SybaseAnywhereMaxValueIncrementer` | `support/incrementer/SybaseAnywhereMaxValueIncrementer.java` | `support/incrementer/sybase_anywhere_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.incrementer.SybaseMaxValueIncrementer` | `support/incrementer/SybaseMaxValueIncrementer.java` | `support/incrementer/sybase_max_value_incrementer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.lob.AbstractLobHandler` | `support/lob/AbstractLobHandler.java` | `support/lob/abstract_lob_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.lob.DefaultLobHandler` | `support/lob/DefaultLobHandler.java` | `support/lob/default_lob_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.lob.LobCreator` | `support/lob/LobCreator.java` | `support/lob/lob_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.lob.LobHandler` | `support/lob/LobHandler.java` | `support/lob/lob_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.lob.PassThroughBlob` | `support/lob/PassThroughBlob.java` | `support/lob/pass_through_blob.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.lob.PassThroughClob` | `support/lob/PassThroughClob.java` | `support/lob/pass_through_clob.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.lob.TemporaryLobCreator` | `support/lob/TemporaryLobCreator.java` | `support/lob/temporary_lob_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.rowset.ResultSetWrappingSqlRowSet` | `support/rowset/ResultSetWrappingSqlRowSet.java` | `support/rowset/result_set_wrapping_sql_row_set.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.rowset.ResultSetWrappingSqlRowSetMetaData` | `support/rowset/ResultSetWrappingSqlRowSetMetaData.java` | `support/rowset/result_set_wrapping_sql_row_set_meta_data.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.rowset.SqlRowSet` | `support/rowset/SqlRowSet.java` | `support/rowset/sql_row_set.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.rowset.SqlRowSetMetaData` | `support/rowset/SqlRowSetMetaData.java` | `support/rowset/sql_row_set_meta_data.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.xml.Jdbc4SqlXmlHandler` | `support/xml/Jdbc4SqlXmlHandler.java` | `support/xml/jdbc4_sql_xml_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.xml.SqlXmlFeatureNotImplementedException` | `support/xml/SqlXmlFeatureNotImplementedException.java` | `support/xml/sql_xml_feature_not_implemented_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.xml.SqlXmlHandler` | `support/xml/SqlXmlHandler.java` | `support/xml/sql_xml_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.xml.SqlXmlValue` | `support/xml/SqlXmlValue.java` | `support/xml/sql_xml_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.xml.XmlBinaryStreamProvider` | `support/xml/XmlBinaryStreamProvider.java` | `support/xml/xml_binary_stream_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.xml.XmlCharacterStreamProvider` | `support/xml/XmlCharacterStreamProvider.java` | `support/xml/xml_character_stream_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.jdbc.support.xml.XmlResultProvider` | `support/xml/XmlResultProvider.java` | `support/xml/xml_result_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
