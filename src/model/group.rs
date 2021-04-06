use crate::model::prelude::*;

pub const MAX_GROUP_NAME_LEN: usize = 190;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PolarClass)]
pub struct Group {
    #[polar(attribute)]
    uuid: Uuid,
    _revision: i32,
    pub name: String,
    pub desc: String,
    members: Option<Vec<MinObject>>,
    #[polar(attribute)]
    pub groups: GroupMembership,
}

impl Group {
    pub fn new(uuid: Uuid, name: &str, desc: &str) -> Self {
        Group {
            uuid,
            _revision: 1,
            name: name.to_string(),
            desc: desc.to_string(),
            members: None,
            groups: GroupMembership::new()
        }
    }

    #[inline]
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub async fn load_by_uuid(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<Self> {
        trace!("Loading group {:?}", uuid);
        let row = sqlx::query!(
            "SELECT `uuid`, `_revision`, `name`, `desc` FROM `group` WHERE `uuid` = ?",
            uuid
        )
        .fetch_one(&mut *tx)
        .await?;
        let uuid = parse_uuid_vec(row.uuid)?;

        Ok(Group {
            uuid,
            _revision: row._revision,
            name: row.name,
            desc: row.desc,
            members: None,
            groups: GroupMembership::load_for(uuid, tx).await?
        })
    }

    pub fn validate(&self) -> Vec<InvalidValue> {
        let len = self.name.chars().count();
        let mut ans = vec![];
        if !(MIN_NON_EMPTY_STR < len && len <= MAX_GROUP_NAME_LEN) {
            ans.push(InvalidValue::OutOfRange(
                "group.name",
                MIN_NON_EMPTY_STR,
                MAX_GROUP_NAME_LEN,
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

    pub async fn save(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        trace!("Saving group {:?}", self.uuid);

        self.validate_as_err()?;

        match self._revision {
            0 => self.db_insert(tx).await?,
            _ => self.db_update(tx).await?,
        };
        self.db_save_members(tx).await?;
        self.groups.save_for(self.uuid, tx).await?;
        Ok(())
    }

    pub async fn delete(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<()> {
        sqlx::query!("DELETE FROM `group` WHERE `uuid` = ?", uuid)
            .execute(&mut *tx)
            .await?;
        Ok(())
    }

    async fn db_insert(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        self._revision = 1;
        sqlx::query!(
            "INSERT INTO `group` (`uuid`, `_revision`, `name`, `desc`) VALUES (?, ?, ?, ?)",
            self.uuid,
            self._revision,
            self.name,
            self.desc
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }

    async fn db_update(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        self._revision += 1;
        sqlx::query!(
            "UPDATE `group` SET `_revision` = ?, `name` = ?, `desc` = ? WHERE `uuid` = ?",
            self._revision,
            self.name,
            self.desc,
            self.uuid
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }

    async fn db_save_members(&self, tx: &mut Transaction<'_>) -> FResult<()> {
        let direct_members = match &self.members {
            Some(v) => v,
            None => return Ok(()),
        };
        sqlx::query!(
            "DELETE FROM `group_members` WHERE `group_uuid` = ?",
            self.uuid
        )
        .execute(&mut *tx)
        .await?;

        for member in direct_members {
            sqlx::query!(
                "INSERT INTO `group_members` (`group_uuid`, `member_uuid`) VALUES (?, ?)",
                self.uuid,
                member.get_uuid()
            )
            .execute(&mut *tx)
            .await?;
        }

        Ok(())
    }
}
