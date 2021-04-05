use crate::model::prelude::*;

pub const MAX_GROUP_NAME_LEN: usize = 190;

#[derive(Debug, Clone, Eq, PolarClass, Serialize, Deserialize)]
// Note that equality for [`MinGroup`] is determined solely by the UUID
pub struct MinGroup {
    #[polar(attribute)]
    uuid: Uuid,
    #[polar(attribute)]
    pub name: String,
}

impl MinGroup {
    pub fn new(uuid: Uuid, name: &str) -> Self {
        MinGroup {
            uuid,
            name: name.to_string(),
        }
    }

    #[inline]
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    // (direct groups, all groups)
    pub async fn load_for(
        uuid: Uuid,
        tx: &mut Transaction<'_>,
    ) -> FResult<(FSet<MinGroup>, FSet<MinGroup>)> {
        let rows = sqlx::query!(
            "SELECT `group_uuid`, `group_name` FROM `group_members_view` WHERE `member_uuid` = ?",
            uuid
        )
        .fetch_all(&mut *tx)
        .await?;
        let mut direct_groups = FSet::new();
        for row in rows {
            direct_groups.insert(MinGroup::new(
                parse_uuid_vec(row.group_uuid)?,
                &row.group_name,
            ));
        }

        let mut all_groups = direct_groups.clone();
        let mut old_count = 0;
        // Until all_groups size stabilizes...
        while old_count != all_groups.len() {
            old_count = all_groups.len();

            // ... construct a query to find the parents ...
            let mut sql_query = "SELECT `group_uuid`, `group_name` FROM `group_members_view` WHERE `member_uuid` IN (".to_string();
            let mut first_flag = true;
            for _ in 0..all_groups.len() {
                if first_flag {
                    sql_query += "?";
                    first_flag = false;
                } else {
                    sql_query += ", ?";
                }
            }
            sql_query += ")";

            let mut query = sqlx::query_as(&sql_query);
            for group in &all_groups {
                query = query.bind(group.get_uuid());
            }
            let rows: Vec<(Uuid, String)> = query.fetch_all(&mut *tx).await?;

            // ... and add each result to all_groups
            for row in rows {
                all_groups.insert(MinGroup::new(row.0, &row.1));
            }
        }

        Ok((direct_groups, all_groups))
    }
}

impl PartialEq for MinGroup {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl HashTrait for MinGroup {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.uuid.hash(state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    uuid: Uuid,
    _revision: i32,
    pub name: String,
    pub desc: String,
    direct_members: Option<Vec<MinObject>>,
}

impl Group {
    pub fn new(uuid: Uuid, name: &str, desc: &str) -> Self {
        Group {
            uuid,
            _revision: 0,
            name: name.to_string(),
            desc: desc.to_string(),
            direct_members: None,
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
            direct_members: None,
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
            0 => self.db_insert(tx).await,
            _ => self.db_update(tx).await,
        }
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
        self.db_save_members(tx).await?;
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
        self.db_save_members(tx).await?;
        Ok(())
    }

    async fn db_save_members(&self, tx: &mut Transaction<'_>) -> FResult<()> {
        let direct_members = match &self.direct_members {
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
