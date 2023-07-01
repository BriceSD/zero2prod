use anyhow::anyhow;
use secrecy::Secret;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct AdminPassword(Secret<String>);

impl AdminPassword {
    pub fn parse(password: String) -> Result<AdminPassword, anyhow::Error> {
        let is_empty_or_whitespace = password.trim().is_empty();
        let is_too_long = password.graphemes(true).count() > 128;
        let is_too_short = password.graphemes(true).count() < 12;
        if is_too_long {
            Err(anyhow!("Invalid password, too long"))
        } else if is_too_short {
            Err(anyhow!("Invalid password, too short"))
        } else if is_empty_or_whitespace {
            Err(anyhow!("Invalid password, can't be empty or only whitespaces"))
        } else {
            Ok(Self(Secret::new(password)))
        }
    }
}

impl AsRef<Secret<String>> for AdminPassword {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::AdminPassword;
    use claims::{assert_err, assert_ok};
    use proptest::{proptest, strategy::Strategy};

    //TODO: Add space  as a valid character
    const ADMIN_PASSWORD: &str = "[a-z0-9.!#$%&'*+/=?^_`{|}~-]{12,128}";

    proptest! {

        #[allow(clippy::len_zero)]
        #[test]
        fn valid_passwords_are_parsed_successfully(
            password in ADMIN_PASSWORD.prop_filter(
                "Password length must be in range: 12 <= password <= 128",
                |password| 12 <= password.len() && password.len() <= 128
            ),) {
                dbg!(&password);
                assert!(AdminPassword::parse(password).is_ok());
        }

    //     #[test]
    //     fn valid_emails_with_literal_form_are_parsed_successfully(local in EMAIL_USER, domain in EMAIL_LITERAL) {
    //         let valid_email = format!("{}@{}", local, domain);
    //         dbg!(&valid_email);
    //         assert!(AdminPassword::parse(valid_email).is_ok());
    //     }

     }

    #[test]
    fn empty_password_is_rejected() {
        let password = "".to_string();
        assert_err!(AdminPassword::parse(password));
    }

    #[test]
    fn space_only_password_is_rejected() {
        let password = " ".repeat(20);
        assert_err!(AdminPassword::parse(password));
    }

    #[test]
    fn too_short_password_is_rejected() {
        let password = "a".repeat(11);
        assert_err!(AdminPassword::parse(password));
    }

    #[test]
    fn a_128_grapheme_long_password_is_valid() {
        let password = "a".repeat(128);
        assert_ok!(AdminPassword::parse(password));
    }

    #[test]
    fn a_password_longer_than_128_grapheme_is_rejected() {
        let password = "a".repeat(129);
        assert_err!(AdminPassword::parse(password));
    }

    #[test]
    fn whitespace_only_passwords_are_rejected() {
        let password = " ".to_string();
        assert_err!(AdminPassword::parse(password));
    }
}
