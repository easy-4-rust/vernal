<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-rbatis 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> Spring Framework 不存在 `spring-rbatis` 模块；本目录描述未来 RBatis adapter，不属于 Spring 对象一对一迁移。目标 crate 尚未建立。

## 边界

- Spring 的事务合同由 `vernal-tx` 承担。
- Spring JDBC 对象由 `vernal-jdbc` 承担。
- Spring ORM 对象由 `vernal-orm` 承担。
- 本 crate 只负责 RBatis executor、transaction、interceptor、page 和缓存能力接入 Vernal。

在 RBatis 仓库、提交和精确符号进入清单前，所有对象只可标 `PLANNED`，不能标 `DEPENDENCY_REUSED`。

## 目标目录

末两层目录用于 adapter 自身的来源结构，例如 `intercept/cache/Foo` → `intercept/cache/foo.rs`；每个公开 adapter 对象独立文件，`mod.rs` 仅重导出。

验收遵循[迁移验收规范](../迁移验收规范.md)，并要求真实数据库事务、拦截顺序和错误映射集成测试。
