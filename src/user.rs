use crate::prelude::*;

#[derive(Debug)]
pub struct User {
    uuid: Uuid,
    display_name: String,
    login_handles: Vec<LoginHandle>,
    auth_info: String,
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
            login_handles: vec![],
            auth_info: "{}".to_string(),
        }
    }

    pub async fn load_by_uuid(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<User> {
        let uuid = uuid.to_string();
        User::load_by_uuid_str(&uuid, tx).await
    }

    pub async fn load_by_uuid_str(uuid: &str, tx: &mut Transaction<'_>) -> FResult<User> {
        unimplemented!()
        // println!("Loading user {:?}", uuid);
        // let base_row = sqlx::query!(
        //     "SELECT `uuid`, `display_name`, `auth_info` FROM `user` WHERE `uuid` = ?",
        //     uuid
        // )
        // .fetch_one(&mut *tx)
        // .await?;
        // println!("{:?}", base_row);

        // let handle_row = sqlx::query!(
        //     "SELECT `login_handle`, `kind` FROM `login_handle` WHERE `user_uuid` = ?",
        //     uuid
        // )
        // .fetch_all(&mut *tx)
        // .await?;
        // println!("{:?}", handle_row);

        // let mut handles = vec![];
        // for handle in handle_row {
        //     handles.push(LoginHandle {
        //         handle: handle.login_handle,
        //         kind: handle.kind,
        //     })
        // }

        // Ok(User {
        //     uuid: parse_uuid(&base_row.uuid)?,
        //     display_name: base_row.display_name,
        //     login_handles: handles,
        //     auth_info: base_row.auth_info,
        // })
    }

    pub async fn load_by_login_handle(
        login_handle: &str,
        tx: &mut Transaction<'_>,
    ) -> FResult<User> {
        // let login_handle = login_handle.trim();
        // let row = sqlx::query!(
        //     "SELECT `user_uuid` FROM `login_handle` WHERE `login_handle` = ?",
        //     login_handle
        // )
        // .fetch_one(&mut *tx)
        // .await?;
        // User::load_by_uuid_str(&row.user_uuid, tx).await
        unimplemented!()
    }
}
