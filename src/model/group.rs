use crate::model::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct MinGroup {
    uuid: Uuid,
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
    ) -> FResult<(HashSet<MinGroup>, HashSet<MinGroup>)> {
        let rows = sqlx::query!(
            "SELECT `group_uuid`, `group_name` FROM `group_members_view` WHERE `member_uuid` = ?",
            uuid
        )
        .fetch_all(&mut *tx)
        .await?;
        let mut direct_groups = HashSet::new();
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
