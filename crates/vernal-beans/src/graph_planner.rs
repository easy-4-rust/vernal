//! 确定性依赖图规划器对象。

use std::{any::TypeId, collections::HashMap, sync::Arc};

use crate::{ComponentDefinition, Dependency, GraphError, TraitBinding};

const STATE_NEW: u8 = 0;
const STATE_VISITING: u8 = 1;
const STATE_VISITED: u8 = 2;

/// 对组件定义执行候选选择、环检测和稳定拓扑排序。
///
/// 规划器无运行时状态，只在注册表冻结阶段工作。使用数值状态而非额外公开对象，
/// 保持”一文件一核心对象”，同时将 DFS 的临时状态限制在单次调用中。
///
/// ## 初始化排序
///
/// DFS 拓扑排序保证依赖先于被依赖者。同一拓扑深度的组件按 `init_order`
/// 升序排列（越小越早），相同 `init_order` 保持原始注册顺序。
/// 对标 tx_di 的 `BinaryHeap<Reverse<(i32, usize)>>` 优先队列模式。
pub(crate) struct GraphPlanner;

impl GraphPlanner {
    /// 生成依赖优先的定义索引。
    ///
    /// DFS 拓扑排序保证依赖先于被依赖者。排序后对同一拓扑深度的组件按
    /// `init_order` 升序排列，相同 `init_order` 保持 DFS 产出的稳定顺序。
    pub(crate) fn plan(
        definitions: &[Arc<ComponentDefinition>],
        bindings: &[Arc<TraitBinding>],
    ) -> Result<Vec<usize>, GraphError> {
        let mut candidates: HashMap<TypeId, Vec<usize>> = HashMap::new();
        for (index, definition) in definitions.iter().enumerate() {
            candidates
                .entry(definition.key().type_id)
                .or_default()
                .push(index);
        }
        Self::validate_binding_targets(definitions, bindings)?;

        let mut states = vec![STATE_NEW; definitions.len()];
        let mut stack = Vec::new();
        let mut ordered = Vec::with_capacity(definitions.len());

        for index in 0..definitions.len() {
            Self::visit(
                index,
                definitions,
                bindings,
                &candidates,
                &mut states,
                &mut stack,
                &mut ordered,
            )?;
        }

        // ─── init_order 二次排序 ───
        // 计算每个节点的拓扑深度（最长路径长度），然后按 (depth, init_order, dfs_position) 排序。
        // 这保证：1) 依赖先于被依赖者；2) 同层按 init_order 升序；3) 相同 init_order 保持稳定。
        let depths = Self::compute_depths(definitions, &ordered);
        let mut indexed: Vec<(usize, usize, i32, usize)> = ordered
            .iter()
            .enumerate()
            .map(|(dfs_pos, &idx)| {
                let order = definitions[idx].init_order();
                (idx, depths[idx], order, dfs_pos)
            })
            .collect();
        indexed.sort_by(|a, b| {
            a.1.cmp(&b.1) // 深度优先
                .then(a.2.cmp(&b.2)) // init_order 升序
                .then(a.3.cmp(&b.3)) // DFS 位置稳定
        });
        Ok(indexed.into_iter().map(|(idx, ..)| idx).collect())
    }

    /// 深度优先访问一个定义并在退出节点时写入拓扑顺序。
    fn visit(
        index: usize,
        definitions: &[Arc<ComponentDefinition>],
        bindings: &[Arc<TraitBinding>],
        candidates: &HashMap<TypeId, Vec<usize>>,
        states: &mut [u8],
        stack: &mut Vec<usize>,
        ordered: &mut Vec<usize>,
    ) -> Result<(), GraphError> {
        match states[index] {
            STATE_VISITED => return Ok(()),
            STATE_VISITING => return Err(Self::cycle_error(index, definitions, stack)),
            _ => {}
        }

        states[index] = STATE_VISITING;
        stack.push(index);

        for dependency in definitions[index].dependencies() {
            let dependency_indices =
                Self::select_dependency(dependency, definitions, bindings, candidates, stack)?;
            // Provider 目标只在真正调用 get/get_in 时构造。这里已经完成候选存在性与
            // 唯一性校验，但不能把它误当作 eager 构造边，否则 Provider 无法按次
            // 创建 Transient，也无法打破仅由延迟访问形成的循环。
            if dependency.is_deferred() {
                continue;
            }
            for dependency_index in dependency_indices {
                if states[dependency_index] == STATE_VISITING {
                    return Err(Self::cycle_error(dependency_index, definitions, stack));
                }
                Self::visit(
                    dependency_index,
                    definitions,
                    bindings,
                    candidates,
                    states,
                    stack,
                    ordered,
                )?;
            }
        }

        let popped = stack.pop();
        debug_assert_eq!(popped, Some(index));
        states[index] = STATE_VISITED;
        ordered.push(index);
        Ok(())
    }

    /// 将依赖选择器解析成唯一的定义索引。
    fn select_dependency(
        dependency: &Dependency,
        definitions: &[Arc<ComponentDefinition>],
        bindings: &[Arc<TraitBinding>],
        candidates: &HashMap<TypeId, Vec<usize>>,
        stack: &[usize],
    ) -> Result<Vec<usize>, GraphError> {
        if dependency.is_trait_binding() {
            return Self::select_trait_dependency(dependency, definitions, bindings, stack);
        }

        let matches: Vec<usize> = candidates
            .get(&dependency.type_id)
            .into_iter()
            .flatten()
            .copied()
            .filter(|index| {
                dependency.qualifier().is_none()
                    || definitions[*index].key().qualifier() == dependency.qualifier()
            })
            .collect();

        match matches.as_slice() {
            [index] => Ok(vec![*index]),
            [] if dependency.is_optional() => Ok(Vec::new()),
            [] => {
                let mut path = Self::display_stack(definitions, stack);
                path.push(dependency.to_string());
                Err(GraphError::MissingDependency { path })
            }
            _ => {
                let mut path = Self::display_stack(definitions, stack);
                path.push(dependency.to_string());
                let candidates = matches
                    .iter()
                    .map(|index| definitions[*index].key().to_string())
                    .collect();
                Err(GraphError::AmbiguousDependency { path, candidates })
            }
        }
    }

    /// 将 Trait 依赖转换为一个或多个具体组件定义索引。
    fn select_trait_dependency(
        dependency: &Dependency,
        definitions: &[Arc<ComponentDefinition>],
        bindings: &[Arc<TraitBinding>],
        stack: &[usize],
    ) -> Result<Vec<usize>, GraphError> {
        let matches: Vec<&Arc<TraitBinding>> = bindings
            .iter()
            .filter(|binding| binding.key().type_id == dependency.type_id)
            .filter(|binding| {
                dependency.qualifier().is_none()
                    || binding.key().qualifier() == dependency.qualifier()
            })
            .collect();

        // 集合注入没有实现时得到空集合；存在实现时每个目标都成为真实图边。
        if dependency.is_multiple() {
            return matches
                .into_iter()
                .map(|binding| Self::binding_target_index(binding, definitions))
                .collect();
        }

        let selected = match matches.as_slice() {
            [binding] => *binding,
            [] if dependency.is_optional() => return Ok(Vec::new()),
            [] => {
                let mut path = Self::display_stack(definitions, stack);
                path.push(dependency.to_string());
                return Err(GraphError::MissingDependency { path });
            }
            _ if dependency.qualifier().is_none() => {
                let primary: Vec<&Arc<TraitBinding>> = matches
                    .iter()
                    .copied()
                    .filter(|binding| binding.is_primary())
                    .collect();
                if let [binding] = primary.as_slice() {
                    *binding
                } else {
                    let mut path = Self::display_stack(definitions, stack);
                    path.push(dependency.to_string());
                    return Err(GraphError::AmbiguousDependency {
                        path,
                        candidates: matches.iter().map(ToString::to_string).collect(),
                    });
                }
            }
            _ => {
                let mut path = Self::display_stack(definitions, stack);
                path.push(dependency.to_string());
                return Err(GraphError::AmbiguousDependency {
                    path,
                    candidates: matches.iter().map(ToString::to_string).collect(),
                });
            }
        };

        Ok(vec![Self::binding_target_index(selected, definitions)?])
    }

    /// 校验每个 Trait Binding 都指向一个精确存在的具体组件定义。
    fn validate_binding_targets(
        definitions: &[Arc<ComponentDefinition>],
        bindings: &[Arc<TraitBinding>],
    ) -> Result<(), GraphError> {
        for binding in bindings {
            Self::binding_target_index(binding, definitions)?;
        }
        Ok(())
    }

    /// 返回绑定目标在组件定义表中的稳定索引。
    fn binding_target_index(
        binding: &TraitBinding,
        definitions: &[Arc<ComponentDefinition>],
    ) -> Result<usize, GraphError> {
        definitions
            .iter()
            .position(|definition| definition.key() == binding.target())
            .ok_or_else(|| GraphError::MissingTraitBindingTarget {
                binding: binding.to_string(),
            })
    }

    /// 从当前 DFS 栈构造闭合环诊断。
    fn cycle_error(
        repeated: usize,
        definitions: &[Arc<ComponentDefinition>],
        stack: &[usize],
    ) -> GraphError {
        let start = stack
            .iter()
            .position(|index| *index == repeated)
            .unwrap_or_default();
        let mut path: Vec<String> = stack[start..]
            .iter()
            .map(|index| definitions[*index].key().to_string())
            .collect();
        path.push(definitions[repeated].key().to_string());
        GraphError::Cycle { path }
    }

    /// 将内部索引栈转换成可读组件路径。
    fn display_stack(definitions: &[Arc<ComponentDefinition>], stack: &[usize]) -> Vec<String> {
        stack
            .iter()
            .map(|index| definitions[*index].key().to_string())
            .collect()
    }

    /// 计算每个组件在依赖图中的拓扑深度。
    ///
    /// 深度定义为从任意根节点到该节点的最长路径长度。
    /// 根节点（无依赖）的深度为 0，依赖深度 = max(所有依赖深度) + 1。
    fn compute_depths(definitions: &[Arc<ComponentDefinition>], ordered: &[usize]) -> Vec<usize> {
        let n = definitions.len();
        // 初始化所有节点深度为 0（根节点保持 0）
        let mut depths = vec![0usize; n];

        // 按拓扑序遍历（保证处理当前节点时，其所有依赖的深度已计算完毕）
        for &index in ordered {
            let mut max_dep_depth = 0usize;
            // 遍历当前节点的所有非延迟依赖
            for dep in definitions[index].dependencies() {
                // Provider 类型的依赖不参与深度计算（它们是延迟构造的）
                if dep.is_deferred() {
                    continue;
                }
                // 在拓扑序中查找匹配的依赖定义，取最大深度
                for &dep_idx in ordered {
                    if definitions[dep_idx].key().type_id == dep.type_id {
                        max_dep_depth = max_dep_depth.max(depths[dep_idx]);
                    }
                }
            }
            // 当前节点深度 = 最深依赖的深度 + 1
            // （根节点无依赖时 max_dep_depth 保持 0）
            depths[index] = max_dep_depth;
        }
        depths
    }
}
