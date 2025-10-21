use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, Validate)]
pub struct DeliveryStaffDTO {
    #[validate(length(min = 2, max = 32))]
    pub name: String,

    #[validate(length(min = 11, max = 11))]
    pub phone: String,

    #[validate(length(min = 18, max = 18))]
    #[serde(rename = "idCard")]
    pub id_card: String,

    #[validate(length(min = 2, max = 32))]
    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[validate(range(min = 0, max = 1))]
    pub status: i8,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    #[validate(length(min = 0, max = 255))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, Validate)]
pub struct DeliveryStaffIdDTO {
    #[validate(length(min = 18, max = 18))]
    #[serde(rename = "idCard")]
    pub id_card: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn create_valid_staff() -> DeliveryStaffDTO {
        DeliveryStaffDTO {
            name: "张三".to_string(),
            phone: "13800138000".to_string(),
            id_card: "110101199001011234".to_string(),
            created_by: "管理员".to_string(),
            status: 1,
            created_at: Some(Utc::now()),
            remark: Some("配送员备注".to_string()),
        }
    }

    #[test]
    fn test_valid_delivery_staff_dto() {
        let staff = create_valid_staff();
        assert!(staff.validate().is_ok());
    }

    // 测试 name 字段验证
    #[test]
    fn test_name_validation() {
        // 名字太短
        let mut staff = create_valid_staff();
        staff.name = "张".to_string();
        assert!(staff.validate().is_err());

        // 名字太长
        staff.name = "张".repeat(33);
        assert!(staff.validate().is_err());

        // 名字为空
        staff.name = "".to_string();
        assert!(staff.validate().is_err());
    }

    // 测试 phone 字段验证
    #[test]
    fn test_phone_validation() {
        // 电话号码太短
        let mut staff = create_valid_staff();
        staff.phone = "1380013800".to_string();
        assert!(staff.validate().is_err());

        // 电话号码太长
        staff.phone = "138001380000".to_string();
        assert!(staff.validate().is_err());

        // 电话号码为空
        staff.phone = "".to_string();
        assert!(staff.validate().is_err());
    }

    // 测试 id_card 字段验证
    #[test]
    fn test_id_card_validation() {
        // 身份证号太短
        let mut staff = create_valid_staff();
        staff.id_card = "11010119900101123".to_string();
        assert!(staff.validate().is_err());

        // 身份证号太长
        staff.id_card = "1101011990010112345".to_string();
        assert!(staff.validate().is_err());

        // 身份证号为空
        staff.id_card = "".to_string();
        assert!(staff.validate().is_err());
    }

    // 测试 created_by 字段验证
    #[test]
    fn test_created_by_validation() {
        // 创建者名字太短
        let mut staff = create_valid_staff();
        staff.created_by = "管".to_string();
        assert!(staff.validate().is_err());

        // 创建者名字太长
        staff.created_by = "管".repeat(33);
        assert!(staff.validate().is_err());

        // 创建者名字为空
        staff.created_by = "".to_string();
        assert!(staff.validate().is_err());
    }

    // 测试 status 字段验证
    #[test]
    fn test_status_validation() {
        // 状态值超出范围（大于1）
        let mut staff = create_valid_staff();
        staff.status = 2;
        assert!(staff.validate().is_err());

        // 状态值超出范围（小于0）
        staff.status = -1;
        assert!(staff.validate().is_err());

        // 状态值为0（有效）
        staff.status = 0;
        assert!(staff.validate().is_ok());

        // 状态值为1（有效）
        staff.status = 1;
        assert!(staff.validate().is_ok());
    }

    // 测试 remark 字段验证
    #[test]
    fn test_remark_validation() {
        // 备注为空（有效）
        let mut staff = create_valid_staff();
        staff.remark = None;
        assert!(staff.validate().is_ok());

        // 备注太长
        staff.remark = Some("备".repeat(256));
        assert!(staff.validate().is_err());

        // 备注为空字符串（有效）
        staff.remark = Some("".to_string());
        assert!(staff.validate().is_ok());

        // 备注为最大长度（有效）
        staff.remark = Some("备".repeat(255));
        assert!(staff.validate().is_ok());
    }

    // 测试多个字段同时无效的情况
    #[test]
    fn test_multiple_invalid_fields() {
        let staff = DeliveryStaffDTO {
            name: "张".to_string(),                   // 名字太短
            phone: "1380013800".to_string(),          // 电话号码太短
            id_card: "11010119900101123".to_string(), // 身份证号太短
            created_by: "管".to_string(),             // 创建者名字太短
            status: 2,                                // 状态值无效
            remark: Some("备".repeat(256)),           // 备注太长
            created_at: Some(Utc::now()),
        };

        let validation_result = staff.validate();
        assert!(validation_result.is_err());

        // 验证错误消息包含所有无效字段
        let errors = validation_result.unwrap_err();
        assert!(errors.to_string().contains("name"));
        assert!(errors.to_string().contains("phone"));
        assert!(errors.to_string().contains("id_card"));
        assert!(errors.to_string().contains("created_by"));
        assert!(errors.to_string().contains("status"));
        assert!(errors.to_string().contains("remark"));
    }
}
