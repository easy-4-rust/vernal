<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-rbdc 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> Spring Framework 不存在 `spring-rbdc` 模块。本目录描述未来 rbdc driver adapter；`spring-jdbc` 的正式目标是 `vernal-jdbc`。

目标 crate 尚未建立，rbdc 来源仓库/提交也未进入迁移清单，因此不得把 rbdc 的 282 个文件或概念近似标为 Vernal 完成。

## 职责

- 把 rbdc connection/pool/driver/result/error 接口桥接到 Vernal 数据访问合同。
- 向 `vernal-rbatis` 暴露稳定 driver adapter。
- 不重复定义 `JdbcTemplate`、Spring datasource 或 ORM 事务对象。
- 保持异步资源获取、取消、释放和错误 cause。

目录保留来源末两层；一对象一文件；外部复用需精确符号和集成测试。遵循[迁移验收规范](../迁移验收规范.md)。
