#[derive(Debug)]
pub enum Authenticator {
	Password(PasswordHashAlg, String),
	Totp({
		period: u32,
		secret: Vec<u8>,
		digits: u8,
		alg: HMacAlg,
	}),
	Hotp({
		counter: u32,
		secret: Vec<u8>,
		digits: u8,
		alg: HMacAlg,
	}),
}

#[derive(Debug)]
pub enum PasswordHashAlg {
	Sha1,
	Sha2,
	Algo2,
}

#[derive(Debug)]
pub enum HMacAlg {
	Sha1,
	Sha2,
	Algo2,
}

pub struct Password {
	hash_alg: HashAlg,
	hash_val: String
}

impl Password {
	pub fn verify(cleartext: String) -> bool {
		false
	}

	pub fn set(cleartext: String) {
	}
}

#[derive(Debug)]
Totp struct Name {
	field: Type
}