use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct Password {
    uuid: Uuid,
    user_uuid: Uuid,
    algorithm: String,
    password: String,
    requires_2fa: bool,
    added: DateTime<Utc>,
    used: Option<DateTime<Utc>>
}

impl Password {
    // pub fn new(cleartext: String) -> Password {
    //     let mut hasher = argonautica::Hasher::default();
    //     hasher.opt_out_of_secret_key(true);
    //     let hash = hasher
    //         .with_password(cleartext)
    //         .hash()?;
    //     println!("{:?}", hash);
    // }

    fn new_from_sql(row: i32) -> Password {
        unimplemented!()
    }

    pub async fn load_by_user_uuid(user_uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<Vec<Password>> {
        trace!("Loading password for user {:?}", user_uuid);
        let rows = sqlx::query_as_unchecked!(
            Password,
            "SELECT  `uuid`, `user_uuid`, `algorithm`, `password`, `requires_2fa`, `added`, `used` FROM `password` WHERE `user_uuid` = ?",
            user_uuid
        )
        .fetch_all(&mut *tx)
        .await?;
        println!("{:?}", rows);
        Err(FError::NotImplemented)
        // let mut ans = vec![];
        // for row in rows {
        //     ans.push(Password::new_from_sql(row));
        // }
        // Ok(ans)
    }
}