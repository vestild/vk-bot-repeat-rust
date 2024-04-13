pub fn mask(s: &str) -> String {
    let mut r = String::from(s);
    let len = s.len();
    if len < 3 {
        return String::from("**");
    } else if len <= 6 {
        r.replace_range(1.., &"*".repeat(len - 1))
    } else {
        r.replace_range(2..len - 2, &"*".repeat(len - 4))
    }
    r
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_log_secret() {
        assert_eq!(mask(""), "**");
        assert_eq!(mask("1"), "**");
        assert_eq!(mask("123"), "1**");
        assert_eq!(mask("123456"), "1*****");
        assert_eq!(mask("1234567"), "12***67");
        assert_eq!(mask("123456789"), "12*****89");
    }
}
