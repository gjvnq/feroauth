use crate::prelude::*;

#[derive(Debug,Queryable)]
#[table_name = "user"]
pub struct UserRaw {
    uuid: String,
    display_name: String,
    auth_info: String
}


#[derive(Debug)]
pub struct User {
	uuid: Uuid,
    display_name: String,
	login_handles: Vec<LoginHandle>,
    auth_info: String
}

#[derive(Debug)]
pub struct LoginHandle {
    handle: String,
    kind: String,
}