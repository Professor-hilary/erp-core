use crate::interface::api::errors::AppError;
use std::collections::HashSet;
use uuid::Uuid;

/// Coarse module gates (must match rbac.user_module_grants.module values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Module {
    Settings,
    Accounting,
    Reports,
    Workforce,
    Payroll,
    Inventory,
    Procurement,
    Sales,
    Manufacturing,
    Capital,
    FixedAssets,
    Investment,
}

impl Module {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::Accounting => "accounting",
            Self::Reports => "reports",
            Self::Workforce => "workforce",
            Self::Payroll => "payroll",
            Self::Inventory => "inventory",
            Self::Procurement => "procurement",
            Self::Sales => "sales",
            Self::Manufacturing => "manufacturing",
            Self::Capital => "capital",
            Self::FixedAssets => "fixed_assets",
            Self::Investment => "investment",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "settings" => Self::Settings,
            "accounting" => Self::Accounting,
            "reports" => Self::Reports,
            "workforce" => Self::Workforce,
            "payroll" => Self::Payroll,
            "inventory" => Self::Inventory,
            "procurement" => Self::Procurement,
            "sales" => Self::Sales,
            "manufacturing" => Self::Manufacturing,
            "capital" => Self::Capital,
            "fixed_assets" => Self::FixedAssets,
            "investment" => Self::Investment,
            _ => return None,
        })
    }
}

/// Resolved once per request (or from cache). Handlers only hash-lookup.
#[derive(Debug, Clone)]
pub struct Actor {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub is_tenant_admin: bool,
    pub department_uuid: Option<Uuid>,
    pub authz_version: i64,
    pub modules: HashSet<String>,
    pub permissions: HashSet<String>,
    pub roles: HashSet<String>,
}

impl Actor {
    pub fn has_module(&self, m: Module) -> bool {
        self.is_tenant_admin || self.modules.contains(m.as_str())
    }

    pub fn has_permission(&self, code: &str) -> bool {
        self.is_tenant_admin || self.permissions.contains(code)
    }

    pub fn require_module(&self, m: Module) -> Result<(), AppError> {
        if self.has_module(m) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Module '{}' not granted",
                m.as_str()
            )))
        }
    }

    pub fn require(&self, permission: &str) -> Result<(), AppError> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Permission '{permission}' denied"
            )))
        }
    }
}
