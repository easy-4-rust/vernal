<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->

# 迁移文档门禁执行证据

> 执行日期：2026-07-30（Asia/Shanghai）
>
> Vernal HEAD：`dd20300d16a09200bd8a379ff14db1e2da99b67c`，并包含执行时尚未提交的
> `vernal-beans` 用户工作树修改。Spring HEAD：
> `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；aspect-rs HEAD：
> `89beaa9057b3f2b73093fc31d3219420e0bb6182`。

| 命令 | 结果 | 未清理证据 |
|---|---|---|
| `python3 -m unittest discover -s scripts/tests` | 16 passed | 无失败 |
| `python3 scripts/audit_migration_docs.py --module vernal-beans --check` | 通过 | 对象台账仍含未完成状态 |
| `python3 scripts/audit_migration_docs.py --module vernal-aop --check` | 通过 | 对象台账仍含未完成状态 |
| `python3 scripts/audit_migration_docs.py --all --check` | 23 个源码模块通过；27 个文档目录五件套完整 | 只证明报告未漂移和文档集完整 |
| `cargo test -p vernal-beans` | 退出码 0；doctest 5 passed、7 ignored | 编译器输出大量 missing-docs、unused 等警告；未视为质量完成 |
| `cargo test -p vernal-aop` | 376 个非文档测试通过；doctest 1 passed、13 ignored | `vernal-core` 17 组警告、`vernal-aop` lib 9 组警告，测试目标另有 unused-imports；未视为质量完成 |

本证据不把测试通过换算为对象完成。ignored doctest、编译警告、`AspectRsAdapter`
的 `PARTIAL` 事实以及对象台账中的未完成状态仍须在后续源码迁移中处理。
