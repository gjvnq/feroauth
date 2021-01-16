use crate::prelude::*;

#[derive(Debug)]
pub struct User {
	uuid: Uuid,
    display_name: String,
	login_handles: Vec<String>,
    auth_info: String
}