//! 测试模块汇总
//!
//! 项目所有单元测试的入口模块

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
