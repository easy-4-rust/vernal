//! 确定性依赖图规划器对象。

use std::{any::TypeId, collections::HashMap, sync::Arc};

use crate::{ComponentDefinition, Dependency, GraphError};

const STATE_NEW: u8 = 0;
const STATE_VISITING: u8 = 1;
const STATE_VISITED: u8 = 2;

/// 对组件定义执行候选选择、环检测和稳定拓扑排序。
///
/// 规划器无运行时状态，只在注册表冻结阶段工作。使用数值状态而非额外公开对象，
/// 保持“一文件一核心对象”，同时将 DFS 的临时状态限制在单次调用中。
pub(crate) struct GraphPlanner;

impl GraphPlanner {
    /// 生成依赖优先的定义索引。
    pub(crate) fn plan(definitions: &[Arc<ComponentDefinition>]) -> Result<Vec<usize>, GraphError> {
        let mut candidates: HashMap<TypeId, Vec<usize>> = HashMap::new();
        for (index, definition) in definitions.iter().enumerate() {
            candidates
                .entry(definition.key().type_id)
                .or_default()
                .push(index);
        }

        let mut states = vec![STATE_NEW; definitions.len()];
        let mut stack = Vec::new();
        let mut ordered = Vec::with_capacity(definitions.len());

        for index in 0..definitions.len() {
            Self::visit(
                index,
                definitions,
                &candidates,
                &mut states,
                &mut stack,
                &mut ordered,
            )?;
        }

        Ok(ordered)
    }

    /// 深度优先访问一个定义并在退出节点时写入拓扑顺序。
    fn visit(
        index: usize,
        definitions: &[Arc<ComponentDefinition>],
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
            let dependency_index =
                Self::select_dependency(dependency, definitions, candidates, stack)?;
            if states[dependency_index] == STATE_VISITING {
                return Err(Self::cycle_error(dependency_index, definitions, stack));
            }
            Self::visit(
                dependency_index,
                definitions,
                candidates,
                states,
                stack,
                ordered,
            )?;
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
        candidates: &HashMap<TypeId, Vec<usize>>,
        stack: &[usize],
    ) -> Result<usize, GraphError> {
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
            [index] => Ok(*index),
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
}
