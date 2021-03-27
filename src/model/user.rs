use crate::model::prelude::*;

pub const MAX_DISPLAY_NAME_LEN: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    uuid: Uuid,
    pub display_name: String,
    added: Option<DateTime<Utc>>,
    last_login: Option<DateTime<Utc>>,
    pub login_handles: Vec<LoginHandle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginHandle {
    handle: String,
    kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinUser {
    uuid: Uuid,
    pub display_name: String,
    pub handle: Option<String>,
    // TODO: add groups?
}

impl MinUser {
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub(crate) fn new(uuid: Uuid, display_name: &str) -> MinUser {
        MinUser {
            uuid: uuid,
            display_name: display_name.to_string(),
            handle: None,
        }
    }

    #[allow(unused)]
    pub async fn load_by_login_handle(
        login_handle: &str,
        tx: &mut Transaction<'_>,
    ) -> FResult<MinUser> {
        // Remove trouble making whitespace
        let login_handle = login_handle.trim();

        trace!("Loading user {:?}", login_handle);
        let base_row = sqlx::query!(
            "SELECT `uuid`, `display_name` FROM `user` INNER JOIN `login_handle` ON (`user_uuid` = `uuid`) WHERE `login_handle` = ?",
            login_handle
        )
        .fetch_one(&mut *tx)
        .await?;
        println!("{:?}", base_row);

        Ok(MinUser {
            uuid: parse_uuid_vec(base_row.uuid)?,
            display_name: base_row.display_name,
            handle: Some(login_handle.to_string()),
        })
    }
}

impl std::convert::From<User> for MinUser {
    fn from(val: User) -> Self {
        val.to_min_user()
    }
}

impl User {
    pub(crate) async fn mark_last_login(
        uuid: Uuid,
        time: DateTime<Utc>,
        tx: &mut Transaction<'_>,
    ) -> FResult<()> {
        sqlx::query!(
            "UPDATE `user` SET `last_login` = ? WHERE `uuid` = ? AND (`last_login` IS NULL OR `last_login` <= ?)",
            time, uuid, time
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }

    pub fn to_min_user(&self) -> MinUser {
        MinUser {
            uuid: self.uuid,
            display_name: self.display_name.clone(),
            handle: Some(self.uuid.to_string()),
        }
    }

    #[allow(unused)]
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn new() -> User {
        User {
            uuid: Uuid::new_v4(),
            display_name: "".to_string(),
            added: None,
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

    #[allow(unused)]
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
            added: Some(Utc.from_utc_datetime(&base_row.added)),
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

        if let Ok(uuid) = parse_uuid_str(login_handle) {
            return User::load_by_uuid(uuid, tx).await;
        }

        let base_row = sqlx::query!(
            "SELECT `user`.`uuid`, `user`.`display_name`, `user`.`added`, `user`.`last_login` FROM `user` JOIN `login_handle` ON (`user`.`uuid` = `login_handle`.`user_uuid`) WHERE `login_handle` =  ?",
            login_handle
        )
        .fetch_one(&mut *tx)
        .await?;
        println!("{:?}", base_row);

        let uuid = parse_uuid_vec(base_row.uuid)?;
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
            uuid: uuid,
            display_name: base_row.display_name,
            added: Some(Utc.from_utc_datetime(&base_row.added)),
            last_login: base_row
                .last_login
                .as_ref()
                .map(|dt| Utc.from_utc_datetime(dt)),
            login_handles: handles,
        })
    }

    pub async fn save(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        trace!("Saving user {:?}", self.uuid);
        match self.added {
            Some(_) => self.db_update(tx).await,
            None => self.db_insert(tx).await,
        }
    }

    async fn db_insert(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        self.last_login = None;
        self.added = Some(Utc::now());
        sqlx::query!(
            "INSERT INTO `user` (`uuid`, `display_name`, `added`, `last_login`) VALUES (?, ?, ?, ?)",
            self.uuid,
            self.display_name,
            self.added,
            self.last_login
        )
        .execute(&mut *tx)
        .await?;
        self.db_save_login_handles(tx).await
    }

    async fn db_update(&self, tx: &mut Transaction<'_>) -> FResult<()> {
        sqlx::query!(
            "UPDATE `user` SET `display_name` = ? WHERE `uuid` = ?",
            self.display_name,
            self.uuid
        )
        .execute(&mut *tx)
        .await?;
        self.db_save_login_handles(tx).await
    }

    async fn db_save_login_handles(&self, tx: &mut Transaction<'_>) -> FResult<()> {
        sqlx::query!(
            "DELETE FROM `login_handle` WHERE `user_uuid` = ?",
            self.uuid
        )
        .execute(&mut *tx)
        .await?;

        for handle in &self.login_handles {
            sqlx::query!(
                "INSERT INTO `login_handle` (`login_handle`, `kind`, `user_uuid`) VALUES (?, ?, ?)",
                handle.handle,
                handle.kind,
                self.uuid
            )
            .execute(&mut *tx)
            .await?;
        }

        Ok(())
    }

    pub fn apply_changes(&mut self, changes: &UserChange) {
        if let Some(display_name) = &changes.display_name {
            self.display_name = display_name.to_string()
        }
        if let Some(login_handles) = &changes.login_handles {
            self.login_handles = login_handles.to_vec()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserChange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_handles: Option<Vec<LoginHandle>>,
}
