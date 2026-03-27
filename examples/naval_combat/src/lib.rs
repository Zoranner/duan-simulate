//! 舰队对抗仿真示例
//!
//! 展示 duan 框架在多域协作、跨域服务调用、事件驱动动态实体管理上的完整用法。
//!
//! # 域依赖链
//!
//! faction → space → motion → detection → combat → collision

pub mod components;
pub mod domains;
pub mod events;
