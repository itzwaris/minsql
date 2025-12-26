use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Select,
    Insert,
    Update,
    Delete,
    CreateTable,
    DropTable,
    CreateIndex,
    DropIndex,
    CreateUser,
    GrantPermission,
    RevokePermission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
    pub inherits_from: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub roles: Vec<String>,
}

pub struct RBACManager {
    roles: HashMap<String, Role>,
    users: HashMap<String, User>,
}

impl RBACManager {
    pub fn new() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
            users: HashMap::new(),
        };

        manager.create_default_roles();
        manager
    }

    fn create_default_roles(&mut self) {
        let admin_role = Role {
            name: "admin".to_string(),
            permissions: vec![
                Permission::Select,
                Permission::Insert,
                Permission::Update,
                Permission::Delete,
                Permission::CreateTable,
                Permission::DropTable,
                Permission::CreateIndex,
                Permission::DropIndex,
                Permission::CreateUser,
                Permission::GrantPermission,
                Permission::RevokePermission,
            ]
            .into_iter()
            .collect(),
            inherits_from: Vec::new(),
        };

        let readonly_role = Role {
            name: "readonly".to_string(),
            permissions: vec![Permission::Select].into_iter().collect(),
            inherits_from: Vec::new(),
        };

        let readwrite_role = Role {
            name: "readwrite".to_string(),
            permissions: vec![
                Permission::Select,
                Permission::Insert,
                Permission::Update,
                Permission::Delete,
            ]
            .into_iter()
            .collect(),
            inherits_from: Vec::new(),
        };

        self.roles.insert("admin".to_string(), admin_role);
        self.roles.insert("readonly".to_string(), readonly_role);
        self.roles.insert("readwrite".to_string(), readwrite_role);
    }

    pub fn create_role(&mut self, name: String, permissions: HashSet<Permission>) -> Result<()> {
        if self.roles.contains_key(&name) {
            anyhow::bail!("Role already exists: {}", name);
        }

        let role = Role {
            name: name.clone(),
            permissions,
            inherits_from: Vec::new(),
        };

        self.roles.insert(name, role);
        Ok(())
    }

    pub fn grant_permission(&mut self, role_name: &str, permission: Permission) -> Result<()> {
        let role = self
            .roles
            .get_mut(role_name)
            .ok_or_else(|| anyhow::anyhow!("Role not found: {}", role_name))?;

        role.permissions.insert(permission);
        Ok(())
    }

    pub fn revoke_permission(&mut self, role_name: &str, permission: &Permission) -> Result<()> {
        let role = self
            .roles
            .get_mut(role_name)
            .ok_or_else(|| anyhow::anyhow!("Role not found: {}", role_name))?;

        role.permissions.remove(permission);
        Ok(())
    }

    pub fn create_user(&mut self, username: String, roles: Vec<String>) -> Result<()> {
        if self.users.contains_key(&username) {
            anyhow::bail!("User already exists: {}", username);
        }

        for role in &roles {
            if !self.roles.contains_key(role) {
                anyhow::bail!("Role not found: {}", role);
            }
        }

        let user = User {
            username: username.clone(),
            roles,
        };
        self.users.insert(username, user);
        Ok(())
    }

    pub fn grant_role(&mut self, username: &str, role: String) -> Result<()> {
        let user = self
            .users
            .get_mut(username)
            .ok_or_else(|| anyhow::anyhow!("User not found: {}", username))?;

        if !self.roles.contains_key(&role) {
            anyhow::bail!("Role not found: {}", role);
        }

        if !user.roles.contains(&role) {
            user.roles.push(role);
        }

        Ok(())
    }

    pub fn revoke_role(&mut self, username: &str, role: &str) -> Result<()> {
        let user = self
            .users
            .get_mut(username)
            .ok_or_else(|| anyhow::anyhow!("User not found: {}", username))?;

        user.roles.retain(|r| r != role);
        Ok(())
    }

    pub fn check_permission(&self, username: &str, permission: &Permission) -> bool {
        let user = match self.users.get(username) {
            Some(u) => u,
            None => return false,
        };

        for role_name in &user.roles {
            if let Some(role) = self.roles.get(role_name) {
                if self.role_has_permission(role, permission) {
                    return true;
                }
            }
        }

        false
    }

    fn role_has_permission(&self, role: &Role, permission: &Permission) -> bool {
        if role.permissions.contains(permission) {
            return true;
        }

        for inherited_role_name in &role.inherits_from {
            if let Some(inherited_role) = self.roles.get(inherited_role_name) {
                if self.role_has_permission(inherited_role, permission) {
                    return true;
                }
            }
        }

        false
    }

    pub fn list_users(&self) -> Vec<String> {
        self.users.keys().cloned().collect()
    }

    pub fn list_roles(&self) -> Vec<String> {
        self.roles.keys().cloned().collect()
    }

    pub fn get_user_permissions(&self, username: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        if let Some(user) = self.users.get(username) {
            for role_name in &user.roles {
                if let Some(role) = self.roles.get(role_name) {
                    self.collect_role_permissions(role, &mut permissions);
                }
            }
        }

        permissions
    }

    fn collect_role_permissions(&self, role: &Role, permissions: &mut HashSet<Permission>) {
        permissions.extend(role.permissions.iter().cloned());

        for inherited_role_name in &role.inherits_from {
            if let Some(inherited_role) = self.roles.get(inherited_role_name) {
                self.collect_role_permissions(inherited_role, permissions);
            }
        }
    }
}
