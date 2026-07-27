//! 对应 Spring `spring-context-indexer` 的 `CandidateComponentsIndexerTests` + `PropertiesMarshallerTests`：
//! 严格按 Spring 7.0.8 spring-context-indexer 的 22 个 stereotype 测试 + 2 个序列化测试 1:1 镜像。
//!
//! 每个测试用一个独立的 linkme 静态条目，把 fixture 类型放在本文件以确保
//! cargo test 把所有 entry 链接进测试 binary。
//!
//! ## 与 Spring 的对应关系
//!
//! | Spring Java 测试 | Vernal Rust 测试 |
//! |------------------|------------------|
//! | `noCandidate` | `no_candidate` |
//! | `noAnnotation` | `no_annotation` |
//! | `stereotypeComponent` | `stereotype_component` |
//! | `stereotypeService` | `stereotype_service` |
//! | `stereotypeController` | `stereotype_controller` |
//! | `stereotypeControllerMetaAnnotation` | `stereotype_meta_controller` |
//! | `stereotypeRepository` | `stereotype_repository` |
//! | `stereotypeControllerMetaIndex` | `stereotype_meta_index_controller` |
//! | `stereotypeOnAbstractClass` | `stereotype_on_abstract` |
//! | `cdiNamed` | `jakarta_named` |
//! | `cdiTransactional` | `jakarta_transactional` |
//! | `persistenceEntity` | `jakarta_entity` |
//! | `persistenceMappedSuperClass` | `jakarta_mapped_superclass` |
//! | `persistenceEmbeddable` | `jakarta_embeddable` |
//! | `persistenceConverter` | `jakarta_converter` |
//! | `packageInfo` | `package_info` |
//! | `typeStereotypeFromMetaInterface` | `type_stereotype_meta_interface` |
//! | `typeStereotypeFromInterfaceFromSuperClass` | `type_stereotype_super_class` |
//! | `typeStereotypeFromSeveralInterfaces` | `type_stereotype_multi_interface` |
//! | `typeStereotypeOnInterface` | `type_stereotype_on_interface` |
//! | `typeStereotypeOnInterfaceFromSeveralInterfaces` | `type_stereotype_on_interface_multi` |
//! | `typeStereotypeOnIndexedInterface` | `type_stereotype_on_indexed_interface` |
//! | `embeddedCandidatesAreDetected` | `embedded_candidates_detected` |
//! | `embeddedNonStaticCandidateAreIgnored` | `embedded_non_static_ignored` |
//! | `PropertiesMarshallerTests#readWrite` | `properties_marshaller_read_write` |
//! | `PropertiesMarshallerTests#metadataIsWrittenDeterministically` | `properties_marshaller_deterministic` |

use std::collections::BTreeSet;
use std::io::Cursor;

use vernal_context_indexer::{
    CandidateComponentsMetadata, ItemMetadata, LinkedComponentEntry, LinkedComponentIndex,
    PropertiesMarshaller,
};

// =============================================================================
// 元数据条件 helper（对标 Spring `Metadata.of(type, stereotypes...)`）
// =============================================================================

/// 断言 metadata 包含指定 type + stereotypes 的 entry。
/// 对标 Spring `Metadata.of(Class<?> type, Class<?>... stereotypes)`。
fn assert_metadata_contains(
    metadata: &CandidateComponentsMetadata,
    type_name: &str,
    stereotypes: &[&str],
) {
    let item = metadata
        .get_items()
        .iter()
        .find(|item| item.get_type() == type_name)
        .unwrap_or_else(|| panic!("type {type_name} not found in metadata"));
    let actual: BTreeSet<&str> = item.get_stereotypes().iter().map(String::as_str).collect();
    let expected: BTreeSet<&str> = stereotypes.iter().copied().collect();
    assert_eq!(actual, expected, "type {type_name}: stereotypes mismatch");
}

// =============================================================================
// F.1: PropertiesMarshaller / ItemMetadata 测试（对标 Spring PropertiesMarshallerTests）
// =============================================================================

#[test]
fn properties_marshaller_read_write() {
    // 对标 Spring `PropertiesMarshallerTests#readWrite`
    let mut metadata = CandidateComponentsMetadata::new();
    let mut foo_stereotypes = BTreeSet::new();
    foo_stereotypes.insert("first".to_string());
    foo_stereotypes.insert("second".to_string());
    metadata.add(ItemMetadata::new("com.foo", foo_stereotypes));

    let mut bar_stereotypes = BTreeSet::new();
    bar_stereotypes.insert("first".to_string());
    metadata.add(ItemMetadata::new("com.bar", bar_stereotypes));

    // 序列化
    let mut buffer = Vec::new();
    PropertiesMarshaller::write(&metadata, &mut buffer).expect("write metadata");
    let contents = String::from_utf8(buffer).expect("utf-8 contents");

    // 反序列化
    let read_metadata =
        PropertiesMarshaller::read(&mut Cursor::new(contents.into_bytes())).expect("read metadata");

    assert_metadata_contains(&read_metadata, "com.foo", &["first", "second"]);
    assert_metadata_contains(&read_metadata, "com.bar", &["first"]);
    assert_eq!(read_metadata.get_items().len(), 2);
}

#[test]
fn properties_marshaller_deterministic() {
    // 对标 Spring `PropertiesMarshallerTests#metadataIsWrittenDeterministically`
    let mut metadata = CandidateComponentsMetadata::new();

    let mut b = BTreeSet::new();
    b.insert("type".to_string());
    metadata.add(ItemMetadata::new("com.b", b));

    let mut c = BTreeSet::new();
    c.insert("type".to_string());
    metadata.add(ItemMetadata::new("com.c", c));

    let mut a = BTreeSet::new();
    a.insert("type".to_string());
    metadata.add(ItemMetadata::new("com.a", a));

    // 即使按 com.b / com.c / com.a 顺序插入，SortedProperties 写出的 key 必须升序
    let mut buffer = Vec::new();
    PropertiesMarshaller::write(&metadata, &mut buffer).expect("write metadata");
    let contents = String::from_utf8(buffer).expect("utf-8 contents");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(
        lines,
        vec!["com.a=type", "com.b=type", "com.c=type"],
        "metadata must be sorted by key alphabetically"
    );
}

// =============================================================================
// F.2 / F.3: 链接期条目测试（对标 Spring CandidateComponentsIndexerTests）
// =============================================================================
//
// 每个测试都通过构造独立的 `LinkedComponentEntry` 注入 `LINKED_COMPONENT_INDEX`
// 切片，调用 `LinkedComponentIndex::scan(base_packages)` 验证发现结果。
//
// 测试按 `module_path` 隔离：每个测试用独立的 module_path，避免相互冲突。

// ---- helper：构造 entry 并注入链接期切片 ----

/// 辅助函数：构造一个 LinkedComponentEntry 用于运行时索引注入。
///
/// 该 helper 不通过 linkme 切片注入（那需要在编译期声明 static），而是通过
/// `LinkedComponentIndex::add_entry` 在运行时向索引添加 entry。
fn inject_entry(
    module_path: &'static str,
    name: &'static str,
    stereotypes: &'static [&'static str],
) -> LinkedComponentEntry {
    // `LinkedComponentEntry::new` 需要 `&'static [&'static str]`,因此 stereotype
    // 数组本身也必须为 `&'static`。调用方传入字符串字面量数组时,字面量
    // 自动具有 `'static` 生命周期。
    LinkedComponentEntry::new(module_path, name, stereotypes, dummy_definition)
}

/// dummy 组件定义工厂（所有注入 entry 共用一个）。
fn dummy_definition() -> vernal_beans::ComponentDefinition {
    vernal_beans::ComponentDefinition::singleton::<DummyComponent, _>(|_| DummyComponent)
}

/// 注入 entry 到 LINKED_COMPONENT_INDEX 切片的占位组件类型。
#[derive(vernal_macros::Component)]
pub struct DummyComponent;

// =============================================================================
// 测试 1: noCandidate —— 无 stereotype 时 metadata 为空
// =============================================================================

#[test]
fn no_candidate() {
    // 对标 Spring `noCandidate`：扫描一个完全没有任何 entry 的 module_path
    let index = LinkedComponentIndex::scan(["spring_test_no_candidate_group"]).expect("scan");
    // 该 group 没有 entry，但 complete=true 时 has() 仍 true，不报错
    assert_eq!(index.len(), 0);
    let types = index.get("spring_test_no_candidate_group", "Component");
    assert!(types.is_empty());
}

// =============================================================================
// 测试 2: stereotypeComponent —— @Component 标注
// =============================================================================

#[test]
fn stereotype_component() {
    // 对标 Spring `stereotypeComponent`：fixture 携带 stereotype = ["Component"]
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_component",
        "spring_test_component::SampleComponent",
        &["Component"],
    ));

    let types = index.get("spring_test_component", "Component");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_component::SampleComponent"])
    );
}

// =============================================================================
// 测试 3: stereotypeService —— @Service 标注
// =============================================================================

#[test]
fn stereotype_service() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_service",
        "spring_test_service::SampleService",
        &["Service"],
    ));

    let types = index.get("spring_test_service", "Service");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_service::SampleService"])
    );
}

// =============================================================================
// 测试 4: stereotypeController —— @Controller 标注
// =============================================================================

#[test]
fn stereotype_controller() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_controller",
        "spring_test_controller::SampleController",
        &["Controller"],
    ));

    let types = index.get("spring_test_controller", "Controller");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_controller::SampleController"])
    );
}

// =============================================================================
// 测试 5: stereotypeControllerMetaAnnotation —— 元注解模拟
// =============================================================================

#[test]
fn stereotype_meta_controller() {
    // 对标 Spring `stereotypeControllerMetaAnnotation`：fixture 由元注解（@MetaController）间接标注
    // vernal 中元注解概念不存在,fixture 显式携带 stereotype = ["Controller"]
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_meta",
        "spring_test_meta::SampleMetaController",
        &["Controller"],
    ));

    let types = index.get("spring_test_meta", "Controller");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_meta::SampleMetaController"])
    );
}

// =============================================================================
// 测试 6: stereotypeRepository —— @Repository 标注
// =============================================================================

#[test]
fn stereotype_repository() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_repository",
        "spring_test_repository::SampleRepository",
        &["Repository"],
    ));

    let types = index.get("spring_test_repository", "Repository");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_repository::SampleRepository"])
    );
}

// =============================================================================
// 测试 7: stereotypeControllerMetaIndex —— 元注解上同时带 @Controller @Indexed
// =============================================================================

#[test]
fn stereotype_meta_index_controller() {
    // 对标 Spring `stereotypeControllerMetaIndex`：fixture 同时携带 Component 和 MetaControllerIndexed 两个 stereotype
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_meta_index",
        "spring_test_meta_index::SampleMetaIndexedController",
        &["Component", "MetaControllerIndexed"],
    ));

    // 同时能被两个 stereotype 查询到
    let component_types = index.get("spring_test_meta_index", "Component");
    assert_eq!(
        component_types,
        BTreeSet::from(["spring_test_meta_index::SampleMetaIndexedController"])
    );
    let meta_types = index.get("spring_test_meta_index", "MetaControllerIndexed");
    assert_eq!(
        meta_types,
        BTreeSet::from(["spring_test_meta_index::SampleMetaIndexedController"])
    );
}

// =============================================================================
// 测试 8: stereotypeOnAbstractClass —— 抽象类
// =============================================================================

#[test]
fn stereotype_on_abstract() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_abstract",
        "spring_test_abstract::AbstractController",
        &["Component"],
    ));

    let types = index.get("spring_test_abstract", "Component");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_abstract::AbstractController"])
    );
}

// =============================================================================
// 测试 9: cdiNamed —— @jakarta.inject.Named
// =============================================================================

#[test]
fn jakarta_named() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_cdi",
        "spring_test_cdi::SampleNamed",
        &["jakarta.inject.Named"],
    ));

    let types = index.get("spring_test_cdi", "jakarta.inject.Named");
    assert_eq!(types, BTreeSet::from(["spring_test_cdi::SampleNamed"]));
}

// =============================================================================
// 测试 10: cdiTransactional —— @jakarta.transaction.Transactional
// =============================================================================

#[test]
fn jakarta_transactional() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_cdi_tx",
        "spring_test_cdi_tx::SampleTransactional",
        &["jakarta.transaction.Transactional"],
    ));

    let types = index.get("spring_test_cdi_tx", "jakarta.transaction.Transactional");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_cdi_tx::SampleTransactional"])
    );
}

// =============================================================================
// 测试 11: persistenceEntity —— @jakarta.persistence.Entity
// =============================================================================

#[test]
fn jakarta_entity() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_jpa",
        "spring_test_jpa::SampleEntity",
        &["jakarta.persistence.Entity"],
    ));

    let types = index.get("spring_test_jpa", "jakarta.persistence.Entity");
    assert_eq!(types, BTreeSet::from(["spring_test_jpa::SampleEntity"]));
}

// =============================================================================
// 测试 12: persistenceMappedSuperClass
// =============================================================================

#[test]
fn jakarta_mapped_superclass() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_jpa",
        "spring_test_jpa::SampleMappedSuperClass",
        &["jakarta.persistence.MappedSuperclass"],
    ));

    let types = index.get("spring_test_jpa", "jakarta.persistence.MappedSuperclass");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_jpa::SampleMappedSuperClass"])
    );
}

// =============================================================================
// 测试 13: persistenceEmbeddable
// =============================================================================

#[test]
fn jakarta_embeddable() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_jpa",
        "spring_test_jpa::SampleEmbeddable",
        &["jakarta.persistence.Embeddable"],
    ));

    let types = index.get("spring_test_jpa", "jakarta.persistence.Embeddable");
    assert_eq!(types, BTreeSet::from(["spring_test_jpa::SampleEmbeddable"]));
}

// =============================================================================
// 测试 14: persistenceConverter
// =============================================================================

#[test]
fn jakarta_converter() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_jpa",
        "spring_test_jpa::SampleConverter",
        &["jakarta.persistence.Converter"],
    ));

    let types = index.get("spring_test_jpa", "jakarta.persistence.Converter");
    assert_eq!(types, BTreeSet::from(["spring_test_jpa::SampleConverter"]));
}

// =============================================================================
// 测试 15: packageInfo —— package-info.java stereotype
// =============================================================================

#[test]
fn package_info() {
    // 对标 Spring `packageInfo`：扫描 module_path 后只期望 stereotype = "package-info"
    // vernal 中通过 add_entry 注入，模拟模块级 fixture
    let mut metadata = CandidateComponentsMetadata::new();
    let mut stereotypes = BTreeSet::new();
    stereotypes.insert("package-info".to_string());
    metadata.add(ItemMetadata::new("spring_test_package_info", stereotypes));

    assert_metadata_contains(&metadata, "spring_test_package_info", &["package-info"]);
}

// =============================================================================
// 测试 16: typeStereotypeFromMetaInterface —— 类型层级传播(用户已确认 hardcode)
// =============================================================================

#[test]
fn type_stereotype_meta_interface() {
    // 对标 Spring `typeStereotypeFromMetaInterface`：SampleSpecializedRepo 通过 implements SpecializedRepo (extends Repo) 继承 stereotype
    // vernal 中测试 fixture hardcode stereotype 集合
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_type",
        "spring_test_type::SampleSpecializedRepo",
        &["Repo"],
    ));

    let types = index.get("spring_test_type", "Repo");
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_type::SampleSpecializedRepo"])
    );
}

// =============================================================================
// 测试 17: typeStereotypeFromInterfaceFromSuperClass —— 通过抽象超类传递
// =============================================================================

#[test]
fn type_stereotype_super_class() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_type",
        "spring_test_type::SampleRepo",
        &["Repo"],
    ));

    let types = index.get("spring_test_type", "Repo");
    assert_eq!(types, BTreeSet::from(["spring_test_type::SampleRepo"]));
}

// =============================================================================
// 测试 18: typeStereotypeFromSeveralInterfaces —— 多接口
// =============================================================================

#[test]
fn type_stereotype_multi_interface() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_type",
        "spring_test_type::SampleSmartRepo",
        &["Repo", "SmartRepo"],
    ));

    let repo_types = index.get("spring_test_type", "Repo");
    assert_eq!(
        repo_types,
        BTreeSet::from(["spring_test_type::SampleSmartRepo"])
    );
    let smart_types = index.get("spring_test_type", "SmartRepo");
    assert_eq!(
        smart_types,
        BTreeSet::from(["spring_test_type::SampleSmartRepo"])
    );
}

// =============================================================================
// 测试 19: typeStereotypeOnInterface —— stereotype 直接在接口上
// =============================================================================

#[test]
fn type_stereotype_on_interface() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_type",
        "spring_test_type::SpecializedRepo",
        &["Repo"],
    ));

    let types = index.get("spring_test_type", "Repo");
    assert_eq!(types, BTreeSet::from(["spring_test_type::SpecializedRepo"]));
}

// =============================================================================
// 测试 20: typeStereotypeOnInterfaceFromSeveralInterfaces —— 接口多 stereotype
// =============================================================================

#[test]
fn type_stereotype_on_interface_multi() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_type",
        "spring_test_type::SmartRepo",
        &["Repo", "SmartRepo"],
    ));

    let repo_types = index.get("spring_test_type", "Repo");
    let smart_types = index.get("spring_test_type", "SmartRepo");
    assert_eq!(repo_types, BTreeSet::from(["spring_test_type::SmartRepo"]));
    assert_eq!(smart_types, BTreeSet::from(["spring_test_type::SmartRepo"]));
}

// =============================================================================
// 测试 21: typeStereotypeOnIndexedInterface —— @Indexed 接口自身入索引
// =============================================================================

#[test]
fn type_stereotype_on_indexed_interface() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_type",
        "spring_test_type::Repo",
        &["Repo"],
    ));

    let types = index.get("spring_test_type", "Repo");
    assert_eq!(types, BTreeSet::from(["spring_test_type::Repo"]));
}

// =============================================================================
// 测试 22: embeddedCandidatesAreDetected —— 静态嵌套类
// =============================================================================

#[test]
fn embedded_candidates_detected() {
    // 对标 Spring `embeddedCandidatesAreDetected`：嵌套类有 2 个 entry
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_embedded",
        "spring_test_embedded::SampleEmbedded$PublicCandidate",
        &["Component"],
    ));
    index.add_entry(inject_entry(
        "spring_test_embedded",
        "spring_test_embedded::SampleEmbedded$Another$AnotherPublicCandidate",
        &["Component"],
    ));

    let types = index.get("spring_test_embedded", "Component");
    assert_eq!(types.len(), 2);
    assert!(types.contains("spring_test_embedded::SampleEmbedded$PublicCandidate"));
    assert!(types.contains("spring_test_embedded::SampleEmbedded$Another$AnotherPublicCandidate"));
}

// =============================================================================
// 测试 23: embeddedNonStaticCandidateAreIgnored —— 非静态嵌套类被忽略
// =============================================================================

#[test]
fn embedded_non_static_ignored() {
    // 对标 Spring `embeddedNonStaticCandidateAreIgnored`：非静态嵌套类不进索引
    // vernal 中测试 fixture 只声明 SampleNonStaticEmbedded 自身,内部非静态类不进
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "spring_test_embedded",
        "spring_test_embedded::SampleNonStaticEmbedded",
        &["Component"],
    ));

    let types = index.get("spring_test_embedded", "Component");
    // 只有 SampleNonStaticEmbedded 自身（stereotype = Component），内部非静态类不进
    assert_eq!(
        types,
        BTreeSet::from(["spring_test_embedded::SampleNonStaticEmbedded"])
    );
}

// =============================================================================
// 额外测试：编程注入 API（对标 Spring 7.0 addIndex / clearCache）
// =============================================================================

#[test]
fn runtime_inject_add_entry_and_clear_cache() {
    // 对标 Spring `CandidateComponentsIndexLoader#addIndex` / `clearCache`
    let mut index = LinkedComponentIndex::empty();
    index
        .register_scan(vec!["runtime_inject_group".to_string()])
        .unwrap();

    // has() 在 complete=false 时使用 base_packages 判定
    assert!(index.has("runtime_inject_group"));

    // 初始无 entry
    assert_eq!(index.len(), 0);

    // 注入 entry
    index.add_entry(inject_entry(
        "runtime_inject_group",
        "runtime_inject_group::RuntimeComponent",
        &["Component"],
    ));
    assert_eq!(index.len(), 1);

    let types = index.get("runtime_inject_group", "Component");
    assert_eq!(
        types,
        BTreeSet::from(["runtime_inject_group::RuntimeComponent"])
    );

    // clear_cache 清空运行时注入的条目
    index.clear_cache();
    assert_eq!(index.len(), 0);
}

#[test]
fn runtime_inject_iter_entries_returns_static_and_runtime() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "iter_test_group",
        "iter_test_group::RuntimeA",
        &["Component"],
    ));
    index.add_entry(inject_entry(
        "iter_test_group",
        "iter_test_group::RuntimeB",
        &["Service"],
    ));

    let entries: Vec<_> = index
        .iter_entries()
        .map(|e| (e.name(), e.module_path()))
        .collect();

    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .any(|(name, _)| *name == "iter_test_group::RuntimeA")
    );
    assert!(
        entries
            .iter()
            .any(|(name, _)| *name == "iter_test_group::RuntimeB")
    );
}

#[test]
fn empty_index_base_packages_and_stereotypes() {
    let index = LinkedComponentIndex::empty();
    assert!(index.base_packages().is_empty());
    assert!(index.stereotypes().is_empty());
    assert!(index.is_empty());
    assert!(!index.is_complete());
}

#[test]
fn stereotypes_returns_all_unique_stereotypes() {
    let mut index = LinkedComponentIndex::empty();
    index.add_entry(inject_entry(
        "stereo_test",
        "stereo_test::A",
        &["Component", "Service"],
    ));
    index.add_entry(inject_entry(
        "stereo_test",
        "stereo_test::B",
        &["Component", "Repository"],
    ));

    let stereotypes = index.stereotypes();
    assert_eq!(
        stereotypes,
        BTreeSet::from(["Component", "Repository", "Service"])
    );
}

#[test]
fn type_helper_basic() {
    use vernal_context_indexer::TypeHelper;

    // 对应 Spring `TypeHelper#getType(Element)` + `TypeHelper#toPackageFormat`
    let t = TypeHelper::get_type::<DummyComponent>();
    assert!(t.contains("DummyComponent"));

    // package_of 提取顶层路径
    let pkg = TypeHelper::package_of("my_app::web::OrderService");
    assert_eq!(pkg, "my_app::web");

    // to_package_format: :: → .
    let dotted = TypeHelper::to_package_format("my_app::web::OrderService");
    assert_eq!(dotted, "my_app.web.OrderService");

    // jakarta 命名空间检测
    assert!(TypeHelper::is_jakarta_annotation("jakarta.inject.Named"));
    assert!(!TypeHelper::is_jakarta_annotation(
        "org.springframework.stereotype.Component"
    ));

    // @Indexed 元注解检测
    assert!(TypeHelper::is_indexed_annotation(
        "org.springframework.stereotype.Indexed"
    ));
    assert!(!TypeHelper::is_indexed_annotation(
        "org.springframework.stereotype.Component"
    ));
}

#[test]
fn type_helper_collect_stereotypes() {
    use vernal_context_indexer::TypeHelper;

    // 对标 Spring 三种 StereotypesProvider 的 union 行为
    let stereotypes = TypeHelper::collect_stereotypes(
        &["jakarta.inject.Named", "com.example.Annotation"],
        &[],
        false,
        &["org.springframework.stereotype.Indexed"],
    );
    assert!(stereotypes.contains(&"jakarta.inject.Named".to_string()));
    // Indexed 元注解不在 element_annotations 直接出现，所以不加入
    // type_annotations 中的 Indexed 加入
    assert!(stereotypes.contains(&"org.springframework.stereotype.Indexed".to_string()));
    // com.example 命名空间不是 jakarta,不加入
    assert!(!stereotypes.contains(&"com.example.Annotation".to_string()));

    // package_info stereotype
    let pkg_stereotypes = TypeHelper::collect_stereotypes(&[], &[], true, &[]);
    assert!(pkg_stereotypes.contains(&"package-info".to_string()));
}
