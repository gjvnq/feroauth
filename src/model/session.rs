use crate::model::prelude::*;
use crate::model::user::{MinUser, User};
use actix_web::dev::Payload;
use actix_web::Error as AWError;
use actix_web::FromRequest;
use chrono::Duration;
use futures_util::future::Ready;
use sqlx::{MySql, Pool};
use std::sync::Arc;

const SESSION_LIFE_SHORT: i64 = 15 * 60; // 15 min
const SESSION_LIFE_LONG: i64 = 15 * 24 * 3600; // 15 days
const REFRESH_INTERVAL: i64 = 15; // 15 s

#[derive(Debug, sqlx::FromRow)]
struct FullSessionRaw {
    uuid: Uuid,
    user_uuid: Uuid,
    user_display_name: String,
    real_user_uuid: Uuid,
    real_user_display_name: String,
    login_time: DateTime<Utc>,
    last_used: DateTime<Utc>,
    remember_me: bool,
    ip_addr_real: String,
    ip_addr_peer: String,
    user_agent: String,
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSession {
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
pub struct MinSession {
    /// Session UUID
    pub sid: Uuid,
    pub iat: i64,
    pub exp: i64,
    pub user: MinUser,
    pub real_user: MinUser,
    pub auth_time: i64,
}

impl MinSession {
    pub async fn refresh(&self, db: &Pool<MySql>) -> FResult<()> {
        let now = Utc::now();
        let delta = now.timestamp() - self.iat;
        debug!("Delta: {:?}", delta);
        if REFRESH_INTERVAL < delta {
            let mut tx = db.begin().await?;
            FullSession::refresh_internal(self.sid, now, &mut tx).await?;
            tx.commit().await?;
        }
        Ok(())
    }
}

impl FullSession {
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

    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now <= self.valid_until()
    }

    #[allow(unused)]
    pub fn new(
        user: &MinUser,
        real_user: &MinUser,
        remember_me: bool,
        ip_addr_real: &str,
        ip_addr_peer: &str,
        user_agent: &str,
    ) -> FullSession {
        let now = Utc::now();
        FullSession {
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

    async fn refresh_internal(
        uuid: Uuid,
        time: DateTime<Utc>,
        tx: &mut Transaction<'_>,
    ) -> FResult<()> {
        debug!("Refreshing session {:?} with time {}", uuid, time);
        sqlx::query!(
            "UPDATE `session` SET `last_used` = ? WHERE `uuid` = ?",
            time,
            uuid
        )
        .execute(&mut *tx)
        .await?;

        Ok(())
    }

    pub async fn safe_load_by_uuid(
        uuid: Uuid,
        db_pool: Arc<sqlx::Pool<sqlx::MySql>>,
    ) -> FResult<FullSession> {
        let mut tx = db_pool.begin().await?;
        let mut ans = FullSession::unsafe_load_by_uuid(uuid, &mut tx).await?;

        if !ans.is_valid() {
            let _ = tx.rollback().await;
            warn!(
                "Attempted to use stale session {} last used {}",
                uuid, ans.last_used
            );
            return Err(FError::new(FErrorInner::StaleSession(uuid)));
        }

        ans.last_used = Utc::now();
        FullSession::refresh_internal(ans.uuid, ans.last_used, &mut tx).await?;
        tx.commit().await?;
        Ok(ans)
    }

    #[allow(unused)]
    pub async fn unsafe_load_by_uuid(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<FullSession> {
        trace!("Loading session {:?}", uuid);
        let row = sqlx::query_as_unchecked!(
            FullSessionRaw,
            "SELECT `uuid`, `user_uuid`, `user_display_name`, `real_user_uuid`, `real_user_display_name`, `login_time`, `last_used`, `remember_me`, `ip_addr_real`, `ip_addr_peer`, `user_agent`, `data` FROM `session_view` WHERE `uuid` = ?",
            uuid
        )
        .fetch_one(&mut *tx)
        .await?;

        Ok(FullSession {
            uuid: row.uuid,
            user: MinUser::new(row.user_uuid, &row.user_display_name),
            real_user: MinUser::new(row.real_user_uuid, &row.real_user_display_name),
            login_time: row.login_time,
            last_used: row.last_used,
            remember_me: row.remember_me,
            ip_addr_real: row.ip_addr_real,
            ip_addr_peer: row.ip_addr_peer,
            user_agent: row.user_agent,
        })
    }

    pub async fn save(&self, tx: &mut Transaction<'_>) -> FResult<()> {
        trace!("Saving session {:?}", self.get_uuid());
        sqlx::query!(
            "INSERT INTO `session` (`uuid`, `user_uuid`, `real_user_uuid`, `login_time`, `last_used`, `remember_me`, `ip_addr_real`, `ip_addr_peer`, `user_agent`, `data`) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, '{}')",
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

    pub fn to_claims(&self) -> MinSession {
        let now = Utc::now();
        MinSession {
            /// Session UUID
            sid: self.uuid,
            iat: now.timestamp(),
            exp: self.valid_until().timestamp(),
            user: self.user.clone(),
            real_user: self.real_user.clone(),
            auth_time: self.login_time.timestamp(),
        }
    }

    /// Save the session to the request so that it can be automagically converted to a cookie
    pub fn to_request(self, req: &mut HttpRequest) {
        req.head().extensions_mut().insert(self);
    }
}

impl FromRequest for FullSession {
    type Error = AWError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        use futures_util::future::{err, ok};
        if let Some(session) = req.head().extensions().get::<FullSession>() {
            return ok(session.clone());
        }
        err(actix_web::error::ErrorUnauthorized(""))
    }
}
