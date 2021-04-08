use crate::model::prelude::*;
use oso::Oso;
use oso::ToPolar;
use std::sync::RwLock;
use std::sync::{RwLockReadGuard, RwLockWriteGuard, TryLockError, TryLockResult};

pub struct PolicyEnforcer {
    oso: RwLock<Oso>,
}

impl std::fmt::Debug for PolicyEnforcer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolicyEnforcer").finish()
    }
}

impl PolicyEnforcer {
    fn get_oso_r(&self) -> TryLockResult<RwLockReadGuard<'_, Oso>> {
        self.oso.try_read()
    }

    fn get_oso_rw(&self) -> TryLockResult<RwLockWriteGuard<'_, Oso>> {
        self.oso.try_write()
    }

    fn add_basic_rules(oso: &Oso) -> FResult<()> {
        oso.load_str(r#"allow(actor: User, _, _) if actor.superuser;"#)?;
        oso.load_str(
            r#"allow(actor: User, action, resource) if user_allowed(actor, action, resource);"#,
        )?;
        oso.load_str(r#"allow(actor: User, action, resource) if delegation_exists(other_actor, actor, action, resource) and user_allowed(other_actor, action, resource);"#)?;
        Ok(())
    }

    pub fn new() -> FResult<PolicyEnforcer> {
        let mut oso = Oso::new();

        // Make classes
        oso.register_class(User::get_polar_class_builder().build())?;
        oso.register_class(Group::get_polar_class_builder().build())?;
        oso.register_class(
            GroupMembership::get_polar_class_builder()
                .add_method("has_uuid", GroupMembership::polar_has_uuid)
                .add_method("has_intersection", GroupMembership::polar_has_intersection)
                .build(),
        )?;

        PolicyEnforcer::add_basic_rules(&oso);

        Ok(PolicyEnforcer {
            oso: RwLock::new(oso),
        })
    }

    pub async fn reload(&mut self, tx: &mut Transaction<'_>) -> FResult<()> {
        // Load DB rules
        let rules = PolicyRule::load_all(tx).await?;
        let oso = self.get_oso_rw()?;
        oso.clear_rules();
        PolicyEnforcer::add_basic_rules(&oso)?;
        for rule in rules {
            if let Err(err) = oso.load_str(&rule.code) {
                error!(
                    "Failed to load rule {} - {}: {:?}",
                    rule.get_uuid().to_string(),
                    rule.title,
                    err
                );
            }
        }
        Ok(())
    }

    #[track_caller]
    pub fn is_allowed<Actor, Action, Resource>(
        &self,
        actor: Actor,
        action: Action,
        resource: Resource,
    ) -> FResult<bool>
    where
        Actor: ToPolar,
        Action: ToPolar,
        Resource: ToPolar,
    {
        Ok(self.get_oso_r()?.is_allowed(actor, action, resource)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics_group() {
        let mut enforcer = PolicyEnforcer::new().unwrap();
        let oso = enforcer.get_oso_rw().unwrap();
        oso.load_str(r#"allow(actor: User, _, _) if actor.groups.has_uuid("e90b861a-5f59-4b2c-bd18-d18df76b27dd");"#)
            .unwrap();
        drop(oso);
        let mut user1 = User::new();
        let mut user2 = User::new();
        user2.groups.add(
            Uuid::parse_str("e90b861a-5f59-4b2c-bd18-d18df76b27dd").unwrap(),
            "Admins",
        );
        let mut superuser = User::new();
        superuser.superuser = true;

        assert_eq!(false, enforcer.is_allowed(user1, "foo", "bar").unwrap());
        assert_eq!(true, enforcer.is_allowed(user2, "foo", "bar").unwrap());
        assert_eq!(true, enforcer.is_allowed(superuser, "foo", "bar").unwrap());
    }
}
