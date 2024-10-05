pub enum Permission {
    UserGet,
    UserList,
    UserCreate,
    UserUpdate,
    UserDelete,
}

impl Permission {
    pub fn value(&self) -> String {
        match *self {
            Permission::UserGet => "user:get".to_string(),
            Permission::UserList => "user:list".to_string(),
            Permission::UserCreate => "user:create".to_string(),
            Permission::UserUpdate => "user:update".to_string(),
            Permission::UserDelete => "user:delete".to_string(),
        }
    }
}
