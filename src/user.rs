use crate::prelude::*;

pub const MAX_DISPLAY_NAME_LEN: usize = 30;

#[derive(Debug)]
pub struct User {
    uuid: Uuid,
    display_name: String,
    added: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
    login_handles: Vec<LoginHandle>,
}

#[derive(Debug)]
pub struct LoginHandle {
    handle: String,
    kind: String,
}

impl User {
    pub fn new(name: String) -> User {
        User {
            uuid: Uuid::new_v4(),
            display_name: name,
            added: Utc::now(),
            last_login: None,
            login_handles: vec![],
        }
    }

    pub fn set_display_name(new_name: String) -> Result<(), InvalidValue> {
        let new_name = new_name.trim();
        let len = new_name.chars().count();
        if !(MIN_NON_EMPTY_STR < len && len <= MAX_DISPLAY_NAME_LEN) {
            Err(InvalidValue::OutOfRange(
                "user.display_name".to_string(),
                MIN_NON_EMPTY_STR,
                MAX_DISPLAY_NAME_LEN,
            ))
        } else {
            Ok(())
        }
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
