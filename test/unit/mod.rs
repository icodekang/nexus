//! 单元测试模块
//!
//! 本模块包含所有单元测试

#[cfg(test)]
mod models {
    mod user_test;
    mod provider_test;
    mod model_test;
    mod subscription_test;
}

#[cfg(test)]
mod auth {
    mod password_test;
    mod jwt_test;
}

#[cfg(test)]
mod api {
    mod mod.rs;
}

#[cfg(test)]
mod adapters {
    mod mod.rs;
}
