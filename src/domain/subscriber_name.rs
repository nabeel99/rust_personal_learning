use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);
impl SubscriberName {
    //if allocation is needed, leave it up to the caller
    //should not be empty
    //should not be > 256
    //should not contain forbidden letters
    pub fn parse(s: String) -> Result<Self, String> {
        //check if input is empty
        // `.trim()` returns a view over the input `s` without trailing
        // whitespace-like characters.
        // `.is_empty` checks if the view contains any character.
        let is_empty = s.trim().is_empty();

        //convert to graphenes and check length is not over 256
        let graphene_string = s.graphemes(true).count() > 256;
        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we
        //check for forbidden characters
        let forbidden_characters = ['/', '(', ')', '"', '\\', '{', '}', '/', '<', '>'];
        let contains_forbidden_chars = s.chars().any(|c| forbidden_characters.contains(&c));
        /*
        Burnt sushi on panics :
        […] If your Rust application panics in response to any user input, then the following should be true:
        your application has a bug, whether it be in a library or in the primary application code.
         */
        if contains_forbidden_chars || is_empty || graphene_string {
            Err(format!("{} is not a valid subscriber name", s))
        } else {
            Ok(Self(s))
        }
    }
    pub fn inner(self) -> String {
        // The caller gets the inner string,
        // but they do not have a SubscriberName anymore!
        // That's because `inner` takes `self` by value,
        // consuming it according to move semantics
        self.0
    }
    //     pub fn inner_ref(&self) -> &str {
    //         // The caller gets a shared reference to the inner string.
    // // This gives the caller **read-only** access,
    // // they have no way to compromise our invariants!
    //         &self.0
    //     }
}
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod test {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};
    #[test]
    fn test_a_256_graphenes_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }
    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }
    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Nabeel Naveed".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
