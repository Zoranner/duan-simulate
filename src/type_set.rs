//! 通用类型级列表底座
//!
//! 用递归零大小类型表达“类型集合”，避免 tuple 方案的元素数量上限。

use std::any::TypeId;
use std::marker::PhantomData;

/// 类型级集合的通用接口
pub trait TypeSet: 'static {
    /// 返回集合中所有类型的 TypeId，保持声明顺序。
    fn type_ids() -> Vec<TypeId>
    where
        Self: Sized;
}

/// 空集合
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeSetEnd;

impl TypeSet for TypeSetEnd {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }
}

/// 递归类型节点：Head + Tail
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeSetCons<Head: 'static, Tail: TypeSet>(PhantomData<fn() -> (Head, Tail)>);

impl<Head: 'static, Tail: TypeSet> TypeSet for TypeSetCons<Head, Tail> {
    fn type_ids() -> Vec<TypeId> {
        let mut ids = vec![TypeId::of::<Head>()];
        ids.extend(Tail::type_ids());
        ids
    }
}
