use std::time;

use crate::domain::SubscriberEmail;
use secrecy::{ExposeSecret, Secret};
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: reqwest::Client,
    base_url: String,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub async fn send_email(
        &self,
        recipient_address: SubscriberEmail,
        subject_line: &str,
        html_email_content: &str,
        plain_text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient_address.as_ref(),
            subject: subject_line,
            html_body: html_email_content,
            text_body: plain_text_content,
        };
        let builder = self
            .http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body);
        //executing req
        builder.send().await?.error_for_status()?;
        Ok(())
    }
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            base_url,
            sender,
            //reqwest gives two options
            //instance wide timeout
            //our per request timeout
            //we chose instance wide tiemout.
            http_client: reqwest::Client::builder().timeout(timeout).build().unwrap(),
            authorization_token,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
// Lifetime parameters always start with an apostrophe, `'`
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}
#[cfg(test)]
mod test {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::{ExposeSecret, Secret};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::Request;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    //only need to test properties not specific data
    struct SendEmailBodyMatcher;
    //checks for the presence of properties in the json payload
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let res = serde_json::from_slice::<serde_json::Value>(&request.body);
            if let Ok(body) = res {
                dbg!(&body);
                // Check that all the mandatory fields are populated
                // without inspecting the field values
                //PascalCase
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }
    fn subject() -> String {
        Sentence(1..2).fake()
    }
    fn content() -> String {
        Paragraph(1..10).fake()
    }
    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        //Mock Server creation
        let mock_server = MockServer::start().await;
        //Fake Sender email
        let sender = email();
        //Instantiate our email client instance with mocker server url, fake sender email and fake token using faker
        // let fake_auth = Secret::new(Faker.fake());
        let email_client = email_client(mock_server.uri());
        //Configure mock server to accept any request and respond with 200
        //expect only one call to it
        //mount a mock server so when mock server gets dropped it checks all monuts
        //and verifies our expect invariant.
        // let exposed_secret_as_str = fake_auth.expose_secret();
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            // .and(header(
            //     "X-Postmark-Server-Token",
            //     fake_auth.expose_secret().as_str(),
            // ))
            .and(path("/email"))
            .and(method("POST"))
            //custom body matcher
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .named("Test in email_client")
            .expect(1)
            .mount(&mock_server)
            .await;
        //client side
        //test randomized subscriber email
        let subscriber_email = email();
        //test subjectl ine
        let subject: String = subject();
        //test content
        let content: String = content();

        //Act
        let output = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        assert_ok!(output);

        //Assert
        // Mock expectations are checked on drop
    }
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        //Mock Server creation
        let mock_server = MockServer::start().await;
        //Fake Sender email
        let sender = email();
        //Instantiate our email client instance with mocker server url, fake sender email and fake token using faker
        // let fake_auth = Secret::new(Faker.fake());
        let email_client = email_client(mock_server.uri());
        //Configure mock server to accept any request and respond with 200
        //expect only one call to it
        //mount a mock server so when mock server gets dropped it checks all monuts
        //and verifies our expect invariant.
        // let exposed_secret_as_str = fake_auth.expose_secret();

        let response = ResponseTemplate::new(200)
            //add artificial delay, by default mock server tries to fullfil request asap,
            //but we are adding delays
            //3 minutes
            .set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            //custom body matcher
            .respond_with(response)
            .named("Test in email_client")
            .expect(1)
            .mount(&mock_server)
            .await;
        //client side
        //test randomized subscriber email
        let subscriber_email = email();
        //test subjectl ine
        let subject: String = subject();
        //test content
        let content: String = content();

        //Act
        let output = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        assert_err!(output);

        //Assert
        // Mock expectations are checked on drop
    }
}
