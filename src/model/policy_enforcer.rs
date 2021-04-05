use crate::model::prelude::*;
use oso::Oso;

#[derive(Debug)]
struct PolicyEnforcer {
    oso: Oso
}

impl PolicyEnforcer {
    pub async fn new(tx: &mut Transaction<'_>) -> FResult<PolicyEnforcer> {
        let oso = Oso::new();

        // Make classes
        
        // Add basic rules
        oso.load_str(r#"allow(actor: User, _, _) if actor.superuser;"#)?;
        oso.load_str(r#"allow(actor: User, action, resource) if user_allowed(actor, action, resource);"#)?;
        oso.load_str(r#"allow(actor: User, action, resource) if delegation_exists(other_actor, actor, action, resource) and user_allowed(other_actor, action, resource);"#)?;

        // Load DB rules
        let rules = PolicyRule::load_all(tx).await?;
        for rule in rules {
            if let Err(err) = oso.load_str(&rule.code) {
                error!("Failed to load rule {} - {}: {:?}", rule.get_uuid().to_string(), rule.title, err);
            }
        }

        // TODO: register classes

        Ok(oso)
    }
}