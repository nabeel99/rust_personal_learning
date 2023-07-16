use validator::validate_email;
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        //delegate the heavy lifting of validated to validate_email.
        if !validate_email(&s) {
            Err(format!("Failed Email Validation, {s} is not a valid email"))
        } else {
            Ok(Self(s))
        }
    }
}
impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {

    use crate::domain::SubscriberEmail;
    use claims::assert_err;
    // We are importing the `SafeEmail` faker!
    // We also need the `Fake` trait to get access to the
    // `.fake` method on `SafeEmail`
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            //Rng trait from rand, which is automatically implemented
            //by all types implementing RngCore!
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let test_input = "".to_string();
        assert_err!(SubscriberEmail::parse(test_input));
    }
    #[test]
    fn empty_string_with_whitespace_is_rejected() {
        let test_input = " ".to_string();
        assert_err!(SubscriberEmail::parse(test_input));
    }
    #[test]
    fn missing_at_symbol_is_rejected() {
        let test_input = "ac3r_nabeellive.com".to_string();
        assert_err!(SubscriberEmail::parse(test_input));
    }
    #[test]
    fn missing_subject_is_rejected() {
        let test_input = "@live.com".to_string();
        assert_err!(SubscriberEmail::parse(test_input));
    }
    #[quickcheck_macros::quickcheck]
    fn valid_emails_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
