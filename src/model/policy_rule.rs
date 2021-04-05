use crate::model::prelude::*;

pub const MAX_POLICY_RULE_TITLE_LEN: usize = 190;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct PolicyRule {
    uuid: Uuid,
    _revision: i32,
    /// Unique title given to the rule for the purpose  of admin organization
    pub title: String,
    /// Human readable description of the rule
    pub desc: String,
    /// Code in Polar for [Oso](https://osohq.com)
    pub code: String,
}

impl PolicyRule {
    #[inline]
    #[allow(unused)]
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    #[allow(unused)]
    pub async fn load_by_uuid(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<Self> {
        trace!("Loading PolicyRule {:?}", uuid);
        let row = sqlx::query!(
            "SELECT `uuid`, `_revision`, `title`, `desc`, `code` FROM `policy_rule` WHERE `uuid` = ?",
            uuid
        )
        .fetch_one(&mut *tx)
        .await?;
        let uuid = parse_uuid_vec(row.uuid)?;

        Ok(PolicyRule {
            uuid: uuid,
            _revision: row._revision,
            title: row.title,
            desc: row.desc,
            code: row.code,
        })
    }

    #[allow(unused)]
    pub async fn load_all(tx: &mut Transaction<'_>) -> FResult<Vec<PolicyRule>> {
        let rows = sqlx::query!(
            "SELECT `uuid`, `_revision`, `title`, `desc`, `code` FROM `policy_rule` ORDER BY `title` ASC"
        )
        .fetch_all(&mut *tx)
        .await?;

        let mut ans = Vec::new();
        for row in rows {
            ans.push(PolicyRule {
                uuid: parse_uuid_vec(row.uuid)?,
                _revision: row._revision,
                title: row.title,
                desc: row.desc,
                code: row.code,
            })
        }
        Ok(ans)
    }

    #[allow(unused)]
    pub async fn delete(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<()> {
        sqlx::query!("DELETE FROM `policy_rule` WHERE `uuid` = ?", uuid)
            .execute(&mut *tx)
            .await?;
        Ok(())
    }

    pub fn validate(&self) -> Vec<InvalidValue> {
        let len = self.title.chars().count();
        let mut ans = vec![];
        if !(MIN_NON_EMPTY_STR < len && len <= MAX_POLICY_RULE_TITLE_LEN) {
            ans.push(InvalidValue::OutOfRange(
                "policy_rule.title",
                MIN_NON_EMPTY_STR,
                MAX_POLICY_RULE_TITLE_LEN,
            ))
        }
        ans
    }

    pub fn validate_as_err(&self) -> FResult<()> {
        let errs = self.validate();
        if errs.len() != 0 {
            return Err(FError::new(ValidationError(errs)));
        }
        Ok(())
    }

    #[allow(unused)]
    pub async fn save(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        trace!("Saving PolicyRule {:?}", self.uuid);

        self.validate_as_err()?;

        match self._revision {
            0 => self.db_insert(tx).await,
            _ => self.db_update(tx).await,
        }
    }

    async fn db_insert(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        self._revision = 1;
        sqlx::query!(
            "INSERT INTO `policy_rule` (`uuid`, `_revision`, `title`, `desc`, `code`) VALUES (?, ?, ?, ?, ?)",
            self.uuid,
            self._revision,
            self.title,
            self.desc,
            self.code
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }

    async fn db_update(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        self._revision += 1;
        sqlx::query!(
            "UPDATE `policy_rule` SET `_revision` = ?, `title` = ?, `desc` = ?, `code` = ? WHERE `uuid` = ?",
            self._revision,
            self.title,
            self.desc,
            self.code,
            self.uuid
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }
}
