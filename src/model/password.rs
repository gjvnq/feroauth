use crate::model::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub enum PasswordCheck {
    WrongPassword,
    RightNeeds2FA,
    RightNo2FA,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Password {
    uuid: Uuid,
    user_uuid: Uuid,
    algorithm: String,
    hash: String,
    requires_2fa: bool,
    added: DateTime<Utc>,
    last_used: Option<DateTime<Utc>>,
}

impl Password {
    #[allow(unused)]
    pub fn new(user_uuid: Uuid, cleartext: String, requires_2fa: bool) -> FResult<Password> {
        let cleartext = cleartext.trim();
        let mut hasher = argonautica::Hasher::default();
        hasher.opt_out_of_secret_key(true);
        let hash = hasher.with_password(cleartext).hash()?;
        Ok(Password {
            uuid: Uuid::new_v4(),
            user_uuid: user_uuid,
            algorithm: "ARGON2".to_string(),
            hash: hash,
            requires_2fa: requires_2fa,
            added: Utc::now(),
            last_used: None,
        })
    }

    pub fn verify(&self, cleartext: &str) -> FResult<PasswordCheck> {
        let cleartext = cleartext.trim();
        let mut verifier = argonautica::Verifier::default();
        let ok = verifier
            .with_hash(&self.hash)
            .with_password(cleartext)
            .verify()?;
        match ok {
            false => Ok(PasswordCheck::WrongPassword),
            true => match self.requires_2fa {
                false => Ok(PasswordCheck::RightNo2FA),
                true => Ok(PasswordCheck::RightNeeds2FA),
            },
        }
    }

    pub async fn load_by_user_uuid(
        user_uuid: Uuid,
        tx: &mut Transaction<'_>,
    ) -> FResult<Vec<Password>> {
        trace!("Loading password for user {:?}", user_uuid);
        let rows = sqlx::query_as_unchecked!(
            Password,
            "SELECT  `uuid`, `user_uuid`, `algorithm`, `password` AS `hash`, `requires_2fa`, `added`, `last_used` FROM `password` WHERE `user_uuid` = ?",
            user_uuid
        )
        .fetch_all(&mut *tx)
        .await?;
        Ok(rows)
    }

    /// Tries to use a clear text password to authenticate a user. This function considers all Password objects for the user and tries to find one that does not require 2FA.
    pub async fn verify_for_user(
        user_uuid: Uuid,
        cleartext: &str,
        tx: &mut Transaction<'_>,
    ) -> FResult<PasswordCheck> {
        let passes = Password::load_by_user_uuid(user_uuid, tx).await?;
        let mut best_answer = PasswordCheck::WrongPassword;
        for pass in passes {
            match pass.verify(cleartext) {
                Ok(ans) => {
                    if ans == PasswordCheck::RightNo2FA {
                        return Ok(ans);
                    }
                    if ans == PasswordCheck::RightNeeds2FA {
                        best_answer = ans;
                    }
                }
                Err(err) => {
                    error!("Failed to verify password: {:?}", err);
                }
            }
        }
        Ok(best_answer)
    }
}
