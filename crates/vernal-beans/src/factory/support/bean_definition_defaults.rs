//! BeanDefinitionDefaults — Spring 风格 Bean 定义默认值。

#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionDefaults {
    pub lazyInit: bool,
    pub autowire: bool,
    pub dependencyCheck: bool,
    pub autowireCandidate: bool,
    pub primary: bool,
}

impl BeanDefinitionDefaults {
    pub fn new() -> Self { Self::default() }
    pub fn set_lazy_init(&mut self, v: bool) { self.lazyInit = v; }
    pub fn set_autowire(&mut self, v: bool) { self.autowire = v; }
    pub fn set_dependency_check(&mut self, v: bool) { self.dependencyCheck = v; }
    pub fn set_autowire_candidate(&mut self, v: bool) { self.autowireCandidate = v; }
    pub fn set_primary(&mut self, v: bool) { self.primary = v; }
}
