use lettre::{message::{Attachment, Body, Mailbox, MultiPart, SinglePart}, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

fn _create_mailer() -> SmtpTransport {
  // Get the username and password from the env file
  let username = std::env::var("EMAIL_USERNAME").expect("EMAIL_USERNAME not set");
  let password = std::env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD not set");

  // Create the credentials
  let creds = Credentials::new(username, password);

  // Create a connection to our email provider
  // In this case, we are using Namecheap's Private Email
  // You can use any email provider you want
  SmtpTransport::relay("mail.privateemail.com")
      .unwrap()
      .credentials(creds)
      .build()
}

fn _send_email() {
  // Build the email
  let email = Message::builder()
      .from("info@revillagesociety.org".parse::<Mailbox>().unwrap())
      .to("jschappet@gmail.comls -a".parse::<Mailbox>().unwrap())
      .subject("Test Email")
      .body("Hello, this is a test email!".to_string())
      .unwrap();

  let mailer = create_mailer();

  // Send the email
  match mailer.send(&email) {
      Ok(_) => println!("Basic email sent!"),
      Err(error) => {
          println!("Basic email failed to send. {:?}", error);
      }
  }
}

pub fn _send_email_with_attachments() {
  let image = std::fs::read("src/logo.png").unwrap();
  let image_body = Body::new(image);

  let email = Message::builder()
      .from("contact@bocksdincoding.com".parse::<Mailbox>().unwrap())
      .to("questions@bocksdincoding.com".parse::<Mailbox>().unwrap())
      .subject("Test Email")
      // This takes the place of the .body(...)
      .multipart(
          MultiPart::related()
              // This is our HTML body
              .singlepart(SinglePart::html(
                  "<img src=\"cid:logo.png\" height=50 width=50 />
              <h1>Hello, this is a test email!</h1>
              <p>This is additional context.</p>
              <a href=\"https://bocksdincoding.com\">Check out my blog!</a>"
                      .to_string(),
              ))
              // This is our media to be referenced in the HTML body
              .singlepart(
                  Attachment::new_inline(String::from("logo.png"))
                      .body(image_body, "image/png".parse().unwrap()),
              ),
      )
      .unwrap();

  let mailer = create_mailer();

  match mailer.send(&email) {
      Ok(_) => println!("Email with attachments sent!"),
      Err(error) => {
          println!("Email with attachments failed to send. {:?}", error);
      }
  }
}