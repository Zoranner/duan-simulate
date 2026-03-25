//! 组件（Component）是实体的数据组成单元
//!
//! 组件只包含数据，不包含行为。组件的使用方式由域来决定。
//!
//! # 设计原则
//!
//! - 单一职责：每个组件只描述实体的一个方面
//! - 数据完整：组件应包含描述该方面所需的完整数据
//! - 避免冗余：组件之间不应该有数据冗余
//! - 独立性：组件之间没有依赖关系

use std::any::Any;

/// 组件 trait
///
/// 所有组件必须实现此 trait。组件是纯数据容器，不包含行为逻辑。
///
/// # 类型安全
///
/// 使用类型来区分组件，而不是字符串标识。这提供：
/// - 编译时类型检查
/// - 更好的性能（类型比较比字符串比较更快）
/// - 更好的 IDE 支持
///
/// # 示例
///
/// ```rust,ignore
/// use duan::Component;
///
/// /// 位置组件
/// pub struct Position {
///     pub x: f64,
///     pub y: f64,
///     pub z: f64,
/// }
///
/// impl Component for Position {
///     fn component_type(&self) -> &'static str {
///         "position"
///     }
/// }
/// ```
pub trait Component: Send + Sync + 'static {
    /// 组件类型名称
    ///
    /// 用于序列化、调试和日志记录。
    fn component_type(&self) -> &'static str;

    /// 类型转换（内部使用）
    ///
    /// 用于将 trait 对象转换回具体类型。
    fn as_any(&self) -> &dyn Any;

    /// 类型转换（内部使用，可变）
    ///
    /// 用于将 trait 对象转换回具体类型（可变引用）。
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 将 Box<Self> 转换为 Box<dyn Any>
    ///
    /// 用于移除组件时获取所有权。
    /// 此方法专门为 trait object 设计，不需要 Sized 约束。
    fn into_any_boxed(self: Box<Self>) -> Box<dyn Any>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        pub value: i32,
    }

    impl Component for TestComponent {
        fn component_type(&self) -> &'static str {
            "test"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any_boxed(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    #[test]
    fn test_component_type() {
        let comp = TestComponent { value: 42 };
        assert_eq!(comp.component_type(), "test");
        assert_eq!(comp.value, 42);
    }
}
