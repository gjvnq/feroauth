use crate::model::prelude::*;

pub const MAX_DISPLAY_NAME_LEN: usize = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    uuid: Uuid,
    pub display_name: String,
    added: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
    pub login_handles: Vec<LoginHandle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginHandle {
    handle: String,
    kind: String,
}

impl User {
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    #[allow(unused)]
    pub fn new(name: String) -> User {
        User {
            uuid: Uuid::new_v4(),
            display_name: name,
            added: Utc::now(),
            last_login: None,
            login_handles: vec![],
        }
    }

    #[allow(unused)]
    pub fn is_valid(&self) -> Vec<InvalidValue> {
        let len = self.display_name.chars().count();
        let mut ans = vec![];
        if !(MIN_NON_EMPTY_STR < len && len <= MAX_DISPLAY_NAME_LEN) {
            ans.push(InvalidValue::OutOfRange(
                "user.display_name".to_string(),
                MIN_NON_EMPTY_STR,
                MAX_DISPLAY_NAME_LEN,
            ))
        }
        ans
    }

    pub async fn load_by_uuid(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<User> {
        trace!("Loading user {:?}", uuid);
        let base_row = sqlx::query!(
            "SELECT `uuid`, `display_name`, `added`, `last_login` FROM `user` WHERE `uuid` = ?",
            uuid
        )
        .fetch_one(&mut *tx)
        .await?;
        println!("{:?}", base_row);

        let handle_row = sqlx::query!(
            "SELECT `login_handle`, `kind` FROM `login_handle` WHERE `user_uuid` = ?",
            uuid
        )
        .fetch_all(&mut *tx)
        .await?;
        println!("{:?}", handle_row);

        let mut handles = vec![];
        for handle in handle_row {
            handles.push(LoginHandle {
                handle: handle.login_handle,
                kind: handle.kind,
            })
        }

        Ok(User {
            uuid: parse_uuid_vec(base_row.uuid)?,
            display_name: base_row.display_name,
            added: Utc.from_utc_datetime(&base_row.added),
            last_login: base_row
                .last_login
                .as_ref()
                .map(|dt| Utc.from_utc_datetime(dt)),
            login_handles: handles,
        })
    }

    #[allow(unused)]
    pub async fn load_by_login_handle(
        login_handle: &str,
        tx: &mut Transaction<'_>,
    ) -> FResult<User> {
        // Remove trouble making whitespace
        let login_handle = login_handle.trim();
        trace!("Loading user {:?}", login_handle);

        let row: (Uuid,) = sqlx::query_as(
            "SELECT `user_uuid` as `uuid` FROM `login_handle` WHERE `login_handle` = ?",
        )
        .bind(login_handle)
        .fetch_one(&mut *tx)
        .await?;
        User::load_by_uuid(row.0, tx).await
    }
}
