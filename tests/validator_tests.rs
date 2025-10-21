use market_saas_backend::utils::validator::{check_password, validate_username};

#[test]
fn test_validate_username() {
    // 有效用户名测试
    assert!(validate_username("JohnDoe123"));
    assert!(validate_username("user1234"));
    assert!(validate_username("TESTUSER"));

    // 长度边界测试
    assert!(validate_username("user12")); // 正好6个字符
    assert!(validate_username("abcdefghijklmnopqrstuvwxyz123456")); // 正好32个字符

    // 无效用户名测试
    assert!(!validate_username("short")); // 太短
    assert!(!validate_username("abcdefghijklmnopqrstuvwxyz1234567")); // 太长
    assert!(!validate_username("User@123")); // 包含特殊字符
    assert!(!validate_username("User Name")); // 包含空格
    assert!(!validate_username("")); // 空字符串
}

#[test]
fn test_validate_password() {
    // 有效密码测试
    assert!(check_password("Password123!"));
    assert!(check_password("Abcd1234@"));
    assert!(check_password("Complex#Pass1"));
    assert!(check_password("@#-23Ps!"));

    // 无效密码测试
    assert!(!check_password("password")); // 缺少数字和特殊字符
    assert!(!check_password("password123")); // 缺少特殊字符
    assert!(!check_password("pass4!")); // 太短
    assert!(!check_password("PASSWORD123!")); // 没有小写
    assert!(!check_password("password123!")); // 没有大写
    assert!(!check_password("12345678!")); // 缺少字母
    assert!(!check_password("PasswordABC!")); // 缺少数字
    assert!(!check_password("Password123")); // 缺少特殊字符
    assert!(!check_password("Password12 3@")); // 含有空格
    assert!(!check_password("%!@#-_++!")); // 只有特殊字符
    assert!(!check_password("")); // 空字符串
}
