use crate::model::prelude::*;
use crate::model::user::{User,MinUser};
use chrono::Duration;

const SESSION_LIFE_SHORT: i64 = 15 * 60; // 15 min
const SESSION_LIFE_LONG: i64 = 15 * 24 * 3600; // 15 days

#[derive(Debug, sqlx::FromRow)]
// provisory pub
pub struct FSessionRaw {
    uuid: Uuid,
    user_uuid: Uuid,
    real_user_uuid: Uuid,
    login_time: DateTime<Utc>,
    last_used: DateTime<Utc>,
    remember_me: bool,
    ip_addr_real: Vec<u8>,
    ip_addr_peer: Vec<u8>,
    user_agent: String,
    data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FSessionInner {
    user: MinUser,
    real_user: MinUser,
}

#[derive(Debug, Serialize, Deserialize)]
// This is stored on the cookie itself
struct FSessionMin {
    uuid: Uuid,
    user: MinUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSession {
    uuid: Uuid,
    user: MinUser,
    real_user: MinUser,
    login_time: DateTime<Utc>,
    last_used: DateTime<Utc>,
    remember_me: bool,
    ip_addr_real: String,
    ip_addr_peer: String,
    user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    /// Session UUID
    pub sid: Uuid,
    pub iat: i64,
    pub exp: i64,
    pub user: MinUser,
    pub real_user: MinUser,
    pub auth_time: i64
}


impl FSession {
    #[allow(unused)]
    pub fn get_uuid(&self) -> Uuid {
        return self.uuid;
    }

    #[allow(unused)]
    pub fn get_user(&self) -> &MinUser {
        return &self.user;
    }

    #[allow(unused)]
    pub fn get_real_user(&self) -> &MinUser {
        return &self.real_user;
    }

    #[allow(unused)]
    pub fn valid_until(&self) -> DateTime<Utc> {
        let duration = match self.remember_me {
            true => Duration::seconds(SESSION_LIFE_LONG),
            false => Duration::seconds(SESSION_LIFE_SHORT),
        };
        return self.last_used.clone() + duration;
    }

    #[allow(unused)]
    pub fn new(
        user: &MinUser,
        real_user: &MinUser,
        remember_me: bool,
        ip_addr_real: &str,
        ip_addr_peer: &str,
        user_agent: &str,
    ) -> FSession {
        let now = Utc::now();
        FSession {
            uuid: Uuid::new_v4(),
            user: user.clone(),
            real_user: real_user.clone(),
            ip_addr_real: ip_addr_real.to_string(),
            ip_addr_peer: ip_addr_peer.to_string(),
            user_agent: user_agent.to_string(),
            login_time: now,
            last_used: now,
            remember_me: remember_me,
        }
    }

    /// If `refresh` is true (as it almost always should), the session's valid until time will be extended if needed.
    #[allow(unused)]
    pub async fn load_by_uuid(
        uuid: Uuid,
        _refresh: bool,
        tx: &mut Transaction<'_>,
    ) -> FResult<FSessionRaw> {
        trace!("Loading session {:?}", uuid);
        let row = sqlx::query_as_unchecked!(
            FSessionRaw,
            "SELECT `uuid`, `user_uuid`, `real_user_uuid`, `login_time`, `last_used`, `remember_me`, `ip_addr_real`, `ip_addr_peer`, `user_agent`, `data` FROM `sessions` WHERE `uuid` = ?",
            uuid
        )
        .fetch_one(&mut *tx)
        .await?;
        Ok(row)
    }

    #[allow(unused)]
    pub async fn save(&self, tx: &mut Transaction<'_>) -> FResult<()> {
        trace!("Saving session {:?}", self.get_uuid());
        sqlx::query!(
            "INSERT INTO `sessions` (`uuid`, `user_uuid`, `real_user_uuid`, `login_time`, `last_used`, `remember_me`, `ip_addr_real`, `ip_addr_peer`, `user_agent`, `data`) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, '{}')",
            self.uuid,
            self.user.get_uuid(),
            self.real_user.get_uuid(),
            self.login_time,
            self.last_used,
            self.remember_me,
            self.ip_addr_real,
            self.ip_addr_peer,
            self.user_agent
        )
        .execute(&mut *tx)
        .await?;
        User::mark_last_login(self.user.get_uuid(), self.login_time, tx).await?;

        Ok(())
    }

    pub fn to_claims(&self) -> SessionClaims {
        let now = Utc::now();
        SessionClaims {
            /// Session UUID
            sid: self.uuid,
            iat: now.timestamp(),
            exp: self.valid_until().timestamp(),
            user: self.user.clone(),
            real_user: self.real_user.clone(),
            auth_time: self.login_time.timestamp()
        }
    }
}
