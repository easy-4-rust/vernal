<!-- migration-doc: authority=support canonical=../迁移验收规范.md -->

> 迁移文档治理：本文级别为 **support**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。

# spring-core 中不迁移到 vernal-core 的模块

> 版本：v1.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core
> 目标：明确列出 spring-core 中 **JVM 生态特有**、**不在 vernal-core 范围内**的模块及其处置理由

本文档按 rust-java-migration 技能的 `JAVA_ONLY_EXEMPT` 状态分类,记录 spring-core 中 6 个顶级包(共 **245 个 .java 文件**)不迁移到 vernal-core 的理由。

---

## 一、不迁移模块总览

| Java 包 | 文件数 | 性质 | 不迁移理由 | Rust 替代方案 |
|---|---|---|---|---|
| `org.springframework.asm` | 33 | **第三方库重打包** | Spring 重打包的 ASM 9.x 字节码操作库 | Rust 不需要 JVM 字节码;过程宏 + syn/quote 替代 |
| `org.springframework.cglib` | 131 | **第三方库重打包** | Spring 重打包的 CGLIB 动态代理库 | Rust 用 trait object + 过程宏;vernal-aop 提供运行时拦截 |
| `org.springframework.aot` | 74 | **JVM AOT 支持** | Spring 为 GraalVM native-image 提供的 AOT 提示与代码生成 | Rust 本身就是 AOT 编译,不需要 GraalVM 适配 |
| `org.springframework.objenesis` | 1 | **第三方库重打包** | Spring 重打包的 Objenesis(绕过构造器创建对象) | Rust 没有 JVM 类加载机制;`MaybeUninit::uninit()` 或显式构造 |
| `org.springframework.javapoet` | 0 | **第三方库重打包** | JavaPoet 占位(Rust 用 `quote` 替代) | `proc-macro2` + `quote` 生成 Rust 代码 |
| `org.springframework.lang` | 6 | **nullability 注解** | `@NonNull` / `@Nullable` 等 JSR-305 风格注解 | Rust 类型系统:`Option<T>` + `&T` 默认非空 |
| **合计** | **245** | — | — | — |

> spring-core 总文件数 680,其中 245 个属于上述"不迁移"模块(占 36%),剩余 435 个分布在 `core/*`(327)与 `util/*`(105+3 backoff 等)子包,是 vernal-core 迁移的真正范围。

---

## 二、`org.springframework.asm` — ASM 字节码库

### 2.1 性质

ASM 是一个独立的 Java 字节码操作与解析库(<https://asm.ow2.io/>),Spring 将其重打包到 `org.springframework.asm` 命名空间,用于:

- `spring-core` 的 `core.type.classreading` 子包:不加载类即可读取字节码注解元数据
- `spring-aop` / `spring-aspects` 的代理生成
- `spring-orm` 等模块的类扫描

### 2.2 关键类(前 10)

| ASM 类 | 职责 |
|---|---|
| `ClassReader` | 读取 .class 文件的字节流 visitor |
| `ClassVisitor` | 访问类信息的 visitor 基类 |
| `MethodVisitor` | 访问方法字节码的 visitor |
| `AnnotationVisitor` | 访问注解的 visitor |
| `Type` | JVM 类型描述符(`Ljava/lang/String;`) |
| `Opcodes` | JVM 字节码操作码常量 |
| `ClassWriter` | 生成 .class 文件字节 |
| `MethodWriter` | 生成方法字节码 |
| `Label` | 跳转标签 |
| `SpringClassWriter` | Spring 扩展的 ClassWriter |

### 2.3 不迁移理由

1. **Rust 没有 JVM 字节码**:Rust 编译为原生机器码,没有 `.class` 文件、没有 JVM 操作码、没有字节码 visitor 模式
2. **类型元数据来源不同**:Spring 通过 ASM 读取字节码避免类加载;Rust 通过 `TypeId` + 过程宏在编译期生成元数据
3. **代理生成机制不同**:Spring CGLIB 通过 ASM 生成子类;Rust 用 trait object + 过程宏(`#[derive(Component)]`)在编译期生成注册代码

### 2.4 Rust 替代方案

| ASM 能力 | Rust 替代 |
|---|---|
| 不加载类读取注解 | `vernal-macros` 过程宏在编译期提取 `#[derive(...)]` 属性 |
| 类型描述符 `Type` | `std::any::TypeId` + `std::any::type_name::<T>()` |
| 字节码 visitor | `syn::Item` visitor(过程宏) |
| 运行时类扫描 | `linkme` 分布式切片(编译期收集)+ `inventory`(备选) |

---

## 三、`org.springframework.cglib` — CGLIB 动态代理库

### 3.1 性质

CGLIB(<https://github.com/cglib/cglib>)通过 ASM 生成目标类的子类,实现运行时动态代理。Spring 重打包用于:

- `@Configuration` 类的 full 模式增强(确保 `@Bean` 方法返回单例)
- `@Transactional` / `@Async` / `@Cacheable` 等 AOP 切面的代理创建
- `MethodInterceptor` 风格的拦截器

### 3.2 关键类(前 10)

| CGLIB 类 | 职责 |
|---|---|
| `Enhancer` | 创建动态代理子类的入口 |
| `MethodInterceptor` | 方法拦截器接口 |
| `Callback` | 所有回调的基接口 |
| `Proxy` | 代理对象基类 |
| `FastClass` | 通过索引加速方法分发 |
| `MethodProxy` | 方法代理,支持 `invokeSuper()` |
| `LazyLoader` | 延迟加载回调 |
| `Dispatcher` | 分发回调 |
| `NoOp` | 空操作回调 |
| `CallbackGenerator` | 回调代码生成器 |

### 3.3 不迁移理由

1. **Rust 没有 JVM 类加载 + 字节码增强**:CGLIB 依赖 ASM 生成 `.class` 文件并由 JVM 加载,Rust 没有等价机制
2. **Rust 类型系统更强**:trait + blanket impl + 过程宏可以表达 CGLIB 的绝大部分用例
3. **vernal 已有 vernal-aop**:`vernal-aop` 提供 `Interceptor` + `Next` 运行时环绕拦截,无需子类增强

### 3.4 Rust 替代方案

| CGLIB 能力 | Rust 替代 |
|---|---|
| `Enhancer.create()` 创建代理 | `Arc<dyn Trait>` + `vernal-aop::Interceptor` |
| `MethodInterceptor.intercept()` | `vernal_aop::Interceptor::intercept(Arc<Invocation>, Next)` |
| `@Configuration` full 增强 | `#[derive(Component)]` 过程宏在编译期生成 `Bean` 工厂 |
| `MethodProxy.invokeSuper()` | `Next::run()`(调用链下一个节点) |
| `LazyLoader` 延迟加载 | `OnceLock<T>` 或 `vernal_beans::ComponentProvider<T>` |
| `FastClass` 加速分发 | Rust 静态分发(零开销) |

---

## 四、`org.springframework.aot` — AOT 与 native-image 支持

### 4.1 性质

Spring 6 引入的 AOT(Ahead-Of-Time)处理,用于 GraalVM native-image:

- `aot/hint/`(44 个文件):运行时反射 / 资源 / 代理 / 序列化提示
- `aot/generate/`(21 个文件):AOT 代码生成
- `aot/nativex/`(8 个文件):native-image 配置生成(`native-image.json` 等)

### 4.2 关键类

| AOT 类 | 职责 |
|---|---|
| `RuntimeHints` | 运行时提示注册中心 |
| `ReflectionHints` | 反射提示 |
| `ResourceHints` | 资源提示 |
| `SerializationHints` | 序列化提示 |
| `ProxyHints` | 代理提示 |
| `JdkProxyHints` | JDK 动态代理提示 |
| `HintPredicate` | 提示谓词 |
| `RuntimeHintsRegistrar` | 提示注册器接口 |
| `GeneratedClass` | AOT 生成的类 |
| `GeneratedMethod` | AOT 生成的方法 |
| `NativeConfiguration` | native-image 配置 |
| `BeanFactoryInitializationAotContribution` | BeanFactory 初始化 AOT 贡献 |

### 4.3 不迁移理由

1. **Rust 本身就是 AOT**:`rustc` 编译期生成机器码,所有"运行时反射"在 Rust 中都是编译期静态分发
2. **GraalVM 是 JVM 特有**:native-image 是 JVM 生态的优化,Rust 不需要这种间接层
3. **vernal 不需要运行时提示注册**:`linkme` 在链接期收集所有组件,过程宏在编译期生成注册代码

### 4.4 Rust 替代方案

| AOT 能力 | Rust 替代 |
|---|---|
| `RuntimeHints`(反射提示) | 不需要;过程宏在编译期生成所有反射代码 |
| `GeneratedClass`(AOT 代码) | `proc-macro2::TokenStream` + `quote!` |
| `native-image.json` | 不需要;`cargo build` 直接生成原生二进制 |
| `RuntimeHintsRegistrar` | 不需要;`#[derive(Component)]` 在编译期完成注册 |

---

## 五、`org.springframework.objenesis` — 绕过构造器创建对象

### 5.1 性质

Objenesis(<http://objenesis.org/>)是一个绕过 Java 构造器创建对象的库,Spring 重打包用于:

- `spring-core` 的 `core.serializer`:反序列化时创建对象实例
- CGLIB 代理创建

### 5.2 不迁移理由

1. **Rust 没有等价的"绕过构造器"概念**:Rust 的对象必须通过 `struct` 字面量构造
2. **Rust 反序列化由 serde 处理**:`serde::Deserialize` 通过 `derive` 宏生成 `deserialize` 方法,不需要绕过构造器
3. **如果真的需要未初始化内存**:`std::mem::MaybeUninit::<T>::uninit()` + 后续初始化

### 5.3 Rust 替代方案

| Objenesis 能力 | Rust 替代 |
|---|---|
| `ObjectInstantiator.newInstance()` | `Default::default()` 或 `serde::Deserialize` |
| 绕过构造器创建 | `MaybeUninit::<T>::uninit()`(unsafe,需手动初始化) |

---

## 六、`org.springframework.javapoet` — Java 源码生成

### 6.1 性质

JavaPoet(<https://github.com/square/javapoet>)是 Square 的 Java 源码生成 API,Spring 用于 AOT 代码生成。

### 6.2 不迁移理由

JavaPoet 是 Java 特有的源码生成 API,Rust 有更原生的方案。

### 6.3 Rust 替代方案

| JavaPoet 能力 | Rust 替代 |
|---|---|
| `JavaFile.builder().build()` | `proc_macro2::TokenStream` + `quote::quote!{}` |
| `MethodSpec.builder()` | `quote::quote! { fn #name() -> #ret { #body } }` |
| `TypeSpec.classBuilder()` | `quote::quote! { struct #name { #fields } }` |

---

## 七、`org.springframework.lang` — nullability 注解

### 7.1 性质

Spring 的 6 个 nullability 注解(基于 JSR-305):

- `@NonNull`、`@Nullable`、`@NonNullApi`、`@NonNullFields`、`@NullableApi`、@TLSafe

### 7.2 不迁移理由

Rust 类型系统天然支持 nullability:

- 默认非空(无 `Option<T>` 即非空)
- 可空用 `Option<T>` 显式表达
- 不需要注解

### 7.3 Rust 替代方案

| Java 注解 | Rust 替代 |
|---|---|
| `@NonNull T` | `T`(默认非空) |
| `@Nullable T` | `Option<T>` |
| `@NonNullApi` / `@NonNullFields` | `#![deny(missing_docs)]` + 类型签名约束 |

---

## 八、汇总统计

| 模块 | 文件数 | 状态 | 处置 |
|---|---|---|---|
| `org.springframework.asm` | 33 | `JAVA_ONLY_EXEMPT` | 不迁移,使用 Rust 过程宏替代 |
| `org.springframework.cglib` | 131 | `JAVA_ONLY_EXEMPT` | 不迁移,使用 vernal-aop + 过程宏替代 |
| `org.springframework.aot` | 74 | `JAVA_ONLY_EXEMPT` | 不迁移,Rust 本身就是 AOT |
| `org.springframework.objenesis` | 1 | `JAVA_ONLY_EXEMPT` | 不迁移,使用 serde / Default 替代 |
| `org.springframework.javapoet` | 0 | `JAVA_ONLY_EXEMPT` | 不迁移(实际无文件),使用 quote 替代 |
| `org.springframework.lang` | 6 | `JAVA_ONLY_EXEMPT` | 不迁移,Rust 类型系统天然支持 |
| **合计** | **245** | — | **全部 `JAVA_ONLY_EXEMPT`** |

---

## 九、与 vernal-core 迁移范围的边界划分

| spring-core 子模块 | 文件数 | 是否在 vernal-core 范围 | 归属 crate |
|---|---|---|---|
| `org.springframework.asm` | 33 | ❌ 不在 | (无 Rust 对应) |
| `org.springframework.cglib` | 131 | ❌ 不在 | (vernal-aop 提供运行时替代) |
| `org.springframework.aot` | 74 | ❌ 不在 | (Rust 本身就是 AOT) |
| `org.springframework.objenesis` | 1 | ❌ 不在 | (无 Rust 对应) |
| `org.springframework.lang` | 6 | ❌ 不在 | (Rust 类型系统天然支持) |
| `org.springframework.core.*` | 327 | ✅ **在 vernal-core 范围** | vernal-core |
| `org.springframework.util.*` | 108 | ✅ **在 vernal-core 范围** | vernal-core(部分归 vernal-beans / vernal-context) |

**vernal-core 实际迁移范围**:435 个 Java 类(327 core + 108 util),按子包分别制定迁移路线图(见配套的 14 份子包文档)。

---

## 十、参考

- ASM 官网:<https://asm.ow2.io/>
- CGLIB GitHub:<https://github.com/cglib/cglib>
- Spring AOT 文档:<https://docs.spring.io/spring-framework/reference/core/aot.html>
- GraalVM native-image:<https://www.graalvm.org/native-image/>
- Objenesis:<http://objenesis.org/>
- JavaPoet:<https://github.com/square/javapoet>
- JSR-305 nullability:<https://jcp.org/en/jsr/detail?id=305>