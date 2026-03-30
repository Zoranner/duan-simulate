//! 调度器
//!
//! 在 `World::build()` 时静态分析所有域的依赖关系，构建有序执行计划。
//!
//! # 分析内容
//!
//! 1. **写入冲突检测**：两个域声明写入同一 State 类型 → `build()` 时 panic
//! 2. **循环依赖检测**：`After` 形成环 → `build()` 时 panic
//! 3. **拓扑排序**：按 `After` 依赖关系生成执行顺序（Kahn 算法）
//!
//! # 执行计划
//!
//! 返回 `Vec<usize>`，每个元素是域在注册列表中的索引，按计算顺序排列。
//! 同层（无依赖关系）的域可并行执行（后续扩展），当前为顺序执行。

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// 域调度信息（框架内部使用）
pub(crate) struct DomainInfo {
    pub type_id: TypeId,
    pub writes: Vec<TypeId>,
    pub after: Vec<TypeId>,
}

/// 调度器
///
/// 持有域执行顺序（索引列表），在 `World::build()` 时构建一次，运行期不变。
pub struct Scheduler {
    /// 按执行顺序排列的域索引（对应 World 的 domains 列表）
    pub(crate) execution_order: Vec<usize>,
}

impl Scheduler {
    /// 构建调度器
    ///
    /// - `infos`：各域的调度信息（顺序与 World.domains 一致）
    ///
    /// # Panics
    ///
    /// - 两个域声明写入同一 State 类型
    /// - `After` 依赖形成循环
    pub(crate) fn build(infos: &[DomainInfo]) -> Self {
        // 1. 检测写入冲突
        let mut write_owners: HashMap<TypeId, usize> = HashMap::new();
        for (idx, info) in infos.iter().enumerate() {
            for &type_id in &info.writes {
                if let Some(&prev_idx) = write_owners.get(&type_id) {
                    panic!(
                        "写入冲突：域 #{idx} 和域 #{prev_idx} 同时声明写入组件 {type_id:?}。\
                         每个 State 类型只能由唯一的域写入。"
                    );
                }
                write_owners.insert(type_id, idx);
            }
        }

        // 2. 建立 TypeId → index 映射
        let type_to_idx: HashMap<TypeId, usize> = infos
            .iter()
            .enumerate()
            .map(|(idx, info)| (info.type_id, idx))
            .collect();

        // 3. 构建邻接表（after[i] = {j} 表示 j 必须在 i 之前执行）
        let n = infos.len();
        let mut in_degree = vec![0usize; n];
        let mut deps: Vec<HashSet<usize>> = vec![HashSet::new(); n]; // deps[i] = i 依赖的集合

        for (idx, info) in infos.iter().enumerate() {
            for &dep_tid in &info.after {
                if let Some(&dep_idx) = type_to_idx.get(&dep_tid) {
                    if deps[idx].insert(dep_idx) {
                        in_degree[idx] += 1;
                    }
                }
                // 若依赖的域未注册，忽略（允许可选依赖）
            }
        }

        // 4. Kahn 拓扑排序
        let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        // 固定顺序保证可复现性
        queue.sort_unstable();

        // 构建反向图：before[j] = j 完成后需降低 in_degree 的节点集合
        let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (idx, dep_set) in deps.iter().enumerate() {
            for &dep in dep_set {
                successors[dep].push(idx);
            }
        }

        let mut order = Vec::with_capacity(n);
        while !queue.is_empty() {
            // 取 in_degree 为 0 的最小 index（保证同层稳定排序）
            queue.sort_unstable();
            let cur = queue.remove(0);
            order.push(cur);

            for &succ in &successors[cur] {
                in_degree[succ] -= 1;
                if in_degree[succ] == 0 {
                    queue.push(succ);
                }
            }
        }

        if order.len() != n {
            panic!(
                "域依赖存在循环：共 {n} 个域，但只能排序 {} 个。\
                 请检查 Domain::After 中是否存在环形依赖。",
                order.len()
            );
        }

        Self {
            execution_order: order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info(type_id: TypeId, writes: Vec<TypeId>, after: Vec<TypeId>) -> DomainInfo {
        DomainInfo {
            type_id,
            writes,
            after,
        }
    }

    struct DomainA;
    struct DomainB;

    #[test]
    fn test_scheduler_no_deps() {
        let infos = vec![
            make_info(TypeId::of::<DomainA>(), vec![], vec![]),
            make_info(TypeId::of::<DomainB>(), vec![], vec![]),
        ];
        let sched = Scheduler::build(&infos);
        assert_eq!(sched.execution_order.len(), 2);
    }

    #[test]
    fn test_scheduler_ordered() {
        // B must run after A
        let infos = vec![
            make_info(TypeId::of::<DomainA>(), vec![], vec![]),
            make_info(
                TypeId::of::<DomainB>(),
                vec![],
                vec![TypeId::of::<DomainA>()],
            ),
        ];
        let sched = Scheduler::build(&infos);
        let a_pos = sched.execution_order.iter().position(|&i| i == 0).unwrap();
        let b_pos = sched.execution_order.iter().position(|&i| i == 1).unwrap();
        assert!(a_pos < b_pos);
    }

    #[test]
    #[should_panic(expected = "循环")]
    fn test_scheduler_cycle() {
        // A after B, B after A → cycle
        let infos = vec![
            make_info(
                TypeId::of::<DomainA>(),
                vec![],
                vec![TypeId::of::<DomainB>()],
            ),
            make_info(
                TypeId::of::<DomainB>(),
                vec![],
                vec![TypeId::of::<DomainA>()],
            ),
        ];
        Scheduler::build(&infos);
    }

    #[test]
    #[should_panic(expected = "写入冲突")]
    fn test_scheduler_write_conflict() {
        struct StateX;
        let tid = TypeId::of::<StateX>();
        let infos = vec![
            make_info(TypeId::of::<DomainA>(), vec![tid], vec![]),
            make_info(TypeId::of::<DomainB>(), vec![tid], vec![]),
        ];
        Scheduler::build(&infos);
    }
}
