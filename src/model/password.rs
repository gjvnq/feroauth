#[derive(Debug)]
pub struct Password {
    uuid: Uuid,
    user_uuid: Uuid,
    algorithm: String,
    password: String,
    requires_2fa: bool,
    added: DateTime<Utc>
    last_used: Option<DateTime<Utc>>
}

impl Password {
    pub fn new(cleartext: String) -> Password {
        let mut hasher = argonautica::Hasher::default();
        hasher.opt_out_of_secret_key(true);
        let hash = hasher
            .with_password(cleartext)
            .hash()?;
        println!("{:?}", hash);
    }
}