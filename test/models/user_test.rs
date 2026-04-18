//! 用户模型单元测试
//!
//! 测试 User 结构的创建、验证和序列化

use nexus_models::user::{User, SubscriptionPlan};

#[test]
fn test_user_creation() {
    let user = User::new(
        "test@example.com".to_string(),
        Some("+1234567890".to_string()),
        "hashed_password".to_string(),
        SubscriptionPlan::Monthly,
    );

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.phone, Some("+1234567890".to_string()));
    assert_eq!(user.subscription_plan, SubscriptionPlan::Monthly);
    assert!(!user.is_admin);
}

#[test]
fn test_user_default_plan() {
    let user = User::new(
        "test@example.com".to_string(),
        None,
        "hashed_password".to_string(),
        SubscriptionPlan::None,
    );

    assert_eq!(user.subscription_plan, SubscriptionPlan::None);
}

#[test]
fn test_subscription_plan_as_str() {
    assert_eq!(SubscriptionPlan::None.as_str(), "none");
    assert_eq!(SubscriptionPlan::Monthly.as_str(), "monthly");
    assert_eq!(SubscriptionPlan::Yearly.as_str(), "yearly");
    assert_eq!(SubscriptionPlan::Team.as_str(), "team");
    assert_eq!(SubscriptionPlan::Enterprise.as_str(), "enterprise");
}

#[test]
fn test_subscription_plan_from_str() {
    assert_eq!(SubscriptionPlan::from_str("none"), SubscriptionPlan::None);
    assert_eq!(SubscriptionPlan::from_str("monthly"), SubscriptionPlan::Monthly);
    assert_eq!(SubscriptionPlan::from_str("yearly"), SubscriptionPlan::Yearly);
    assert_eq!(SubscriptionPlan::from_str("team"), SubscriptionPlan::Team);
    assert_eq!(SubscriptionPlan::from_str("enterprise"), SubscriptionPlan::Enterprise);
    assert_eq!(SubscriptionPlan::from_str("unknown"), SubscriptionPlan::None);
}

#[test]
fn test_user_admin_flag() {
    let mut user = User::new(
        "admin@example.com".to_string(),
        None,
        "hashed_password".to_string(),
        SubscriptionPlan::Enterprise,
    );
    assert!(!user.is_admin);

    user.is_admin = true;
    assert!(user.is_admin);
}

#[test]
fn test_user_serialization() {
    let user = User::new(
        "test@example.com".to_string(),
        Some("+1234567890".to_string()),
        "hashed_password".to_string(),
        SubscriptionPlan::Monthly,
    );

    let json = serde_json::to_string(&user).unwrap();
    assert!(json.contains("test@example.com"));
    assert!(json.contains("monthly"));

    let deserialized: User = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.email, user.email);
    assert_eq!(deserialized.subscription_plan, user.subscription_plan);
}
