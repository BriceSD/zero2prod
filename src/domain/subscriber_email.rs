#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<SubscriberEmail, String> {
        if validator::validate_email(&email) {
            Ok(Self(email))
        } else {
            Err(format!("{} is not a valid subscriber email", email))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use proptest::{proptest, strategy::Strategy};

    // Regex from the specs
    // https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
    // It will mark esoteric email addresses like quoted string as invalid
    // according to RFC5321 the max length of the local part is 64 characters
    // and the max length of the domain part is 255 characters
    const EMAIL_USER: &str = "[a-z0-9.!#$%&'*+/=?^_`{|}~-]{1,64}";
    const EMAIL_DOMAIN: &str =
        "[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)";
    // https://datatracker.ietf.org/doc/html/rfc5321#section-4.5.3.1.1
    // literal form, ipv4 or ipv6 address (SMTP 4.1.3)
    // const EMAIL_LITERAL: &str = "\\[([A-f0-9:\\.]+)\\]";

    proptest! {

        #[test]
        fn valid_emails_are_parsed_successfully(
            local in EMAIL_USER.prop_filter(
                "Local length must be in range: 1 <= local <= 64",
                |local| 1 <= local.len() && local.len() <= 64
            ),
            domain in EMAIL_DOMAIN.prop_filter(
                "Domain length must be in range: 1 <= domain <= 255",
                |domain| 1 <= domain.len() && domain.len() <= 255
            )) {
                let valid_email = format!("{}@{}", local, domain);
                dbg!(&valid_email);
                assert!(SubscriberEmail::parse(valid_email).is_ok());
        }

    //     #[test]
    //     fn valid_emails_with_literal_form_are_parsed_successfully(local in EMAIL_USER, domain in EMAIL_LITERAL) {
    //         let valid_email = format!("{}@{}", local, domain);
    //         dbg!(&valid_email);
    //         assert!(SubscriberEmail::parse(valid_email).is_ok());
    //     }

     }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
