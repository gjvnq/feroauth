#![allow(unused)]

pub mod db;
pub mod fset;
pub mod group;
pub mod password;
pub mod policy_delegation;
pub mod policy_rule;
pub mod prelude;
pub mod session;
pub mod user;

// SET = ADD/NEW + EDIT + DEL

pub const POLVERB_USER_NEW: &'static str = "feroauth/user.new";
pub const POLVERB_USER_GET: &'static str = "feroauth/user.get";
pub const POLVERB_USER_DEL: &'static str = "feroauth/user.del";
pub const POLVERB_USER_SET_SUPER: &'static str = "feroauth/user.set_super";
pub const POLVERB_USER_NAME_SET: &'static str = "feroauth/user.name.set";
pub const POLVERB_USER_PASSWORD_GET: &'static str = "feroauth/user.password.get";
pub const POLVERB_USER_PASSWORD_SET: &'static str = "feroauth/user.password.set";
pub const POLVERB_USER_2FA_SET: &'static str = "feroauth/user.2fa.set";
pub const POLVERB_USER_2FA_GET: &'static str = "feroauth/user.2fa.get";
pub const POLVERB_USER_GROUP_ADD: &'static str = "feroauth/user.groups.add";
pub const POLVERB_USER_GROUP_DEL: &'static str = "feroauth/user.groups.del";
pub const POLVERB_USER_LOGIN_ADD: &'static str = "feroauth/user.login.add";
pub const POLVERB_USER_LOGIN_DEL: &'static str = "feroauth/user.login.del";

pub const POLVERB_GROUP_NEW: &'static str = "feroauth/group.new";
pub const POLVERB_GROUP_GET: &'static str = "feroauth/group.get";
pub const POLVERB_GROUP_NAME_SET: &'static str = "feroauth/group.name.set";
pub const POLVERB_GROUP_DESC_SET: &'static str = "feroauth/group.desc.set";
pub const POLVERB_GROUP_DEL: &'static str = "feroauth/group.del";

// Permissions about groups that work for any kind of element
pub const POLVERB_GROUP_MEMBER_ADD_ANYKIND: &'static str = "feroauth/group.add-anykind";
pub const POLVERB_GROUP_MEMBER_DEL_ANYKIND: &'static str = "feroauth/group.del-anykind";

pub const POLVERB_POLICY_RULE_SET: &'static str = "feroauth/policy_rule.set";

pub use fset::FSet;
pub use group::MinGroup;
pub use password::Password;
pub use session::FullSession;
pub use user::{MinUser, User, UserChange};
