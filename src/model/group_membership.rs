use crate::model::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PolarClass)]
// the i32 indicates the level of indirection where 0 means direct
pub struct GroupMembership(HashMap<Uuid, (String, i32)>);

impl GroupMembership {
    pub fn new() -> Self {
        GroupMembership(HashMap::new())
    }

    pub fn add(&mut self, uuid: Uuid, name: &str) {
        self.0.insert(uuid, (name.to_string(), 0));
    }

    pub fn has(&self, uuid: Uuid) -> bool {
        self.0.get(&uuid).is_some()
    }

    pub fn to_keys_set(&self) -> HashSet<Uuid> {
        self.0.keys().cloned().collect()
    }

    pub fn has_intersection(&self, other: &GroupMembership) -> bool {
        let set1 = self.to_keys_set();
        let set2 = other.to_keys_set();
        let mut intersection = set1.intersection(&set2).next();
        intersection.is_some()
    }

    pub fn polar_has_intersection(&self, other: GroupMembership) -> bool {
        self.has_intersection(&other)
    }

    #[track_caller]
    pub fn polar_has_uuid(&self, val: String) -> bool {
        self.has_str(&val)
    }

    #[track_caller]
    pub fn has_str(&self, val: &str) -> bool {
        match parse_uuid_str(&val) {
            Ok(uuid) => self.has(uuid),
            Err(err) => {
                error!("{:?}", err);
                false
            }
        }
    }

    pub async fn save_for(&self, uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<()> {
        sqlx::query!("DELETE FROM `group_members` WHERE `member_uuid` = ?", uuid)
            .execute(&mut *tx)
            .await?;

        for (group, (_, level)) in &self.0 {
            // Save only direct group memberships
            if *level == 0 {
                sqlx::query!(
                    "INSERT INTO `group_members` (`group_uuid`, `member_uuid`) VALUES (?, ?)",
                    group,
                    uuid
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn load_for(uuid: Uuid, tx: &mut Transaction<'_>) -> FResult<GroupMembership> {
        let rows = sqlx::query!(
            "SELECT `group_uuid`, `group_name` FROM `group_members_view` WHERE `member_uuid` = ?",
            uuid
        )
        .fetch_all(&mut *tx)
        .await?;
        let mut groups = HashMap::<Uuid, (String, i32)>::new();
        for row in rows {
            let group_uuid = parse_uuid_vec(row.group_uuid)?;
            groups.insert(group_uuid, (row.group_name, 0));
        }

        let mut old_count = 0;
        let mut level = 1;
        // Until all_groups size stabilizes...
        while old_count != groups.len() {
            old_count = groups.len();

            // ... construct a query to find the parents ...
            let mut sql_query = "SELECT `group_uuid`, `group_name` FROM `group_members_view` WHERE `member_uuid` IN (".to_string();
            let mut first_flag = true;
            for _ in 0..groups.len() {
                if first_flag {
                    sql_query += "?";
                    first_flag = false;
                } else {
                    sql_query += ", ?";
                }
            }
            sql_query += ")";
            let mut query = sqlx::query_as(&sql_query);
            for group in &groups {
                query = query.bind(group.0);
            }

            // ... and add each result to groups
            let rows: Vec<(Uuid, String)> = query.fetch_all(&mut *tx).await?;
            for row in rows {
                if groups.get(&row.0).is_none() {
                    groups.insert(row.0, (row.1, level));
                }
            }
            // Go the next level of indirection
            level += 1;
        }

        Ok(GroupMembership(groups))
    }
}
