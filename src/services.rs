// services.rs
use crate::models::{Account, CreateAccount, CreateTransaction, Transaction, TrialBalance, User, CreateUser, LoginUser, JwtClaims};
use crate::repositories::{AccountRepository, TransactionRepository, UserRepository};
use crate::errors::AppError;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;

pub struct AuthService<R: UserRepository> {
    repo: R,
    jwt_secret: String,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(repo: R, jwt_secret: String) -> Self { Self { repo, jwt_secret } }

    pub async fn register(&self, user: &CreateUser) -> Result<User, AppError> {
        if user.email.is_empty() || user.password.is_empty() {
            return Err(AppError::Validation("Email and password required".to_string()));
        }
        let password_hash = hash(&user.password, DEFAULT_COST).map_err(|_| AppError::Validation("Hash error".to_string()))?;
        self.repo.create(&user.email, &password_hash).await
    }

    pub async fn login(&self, user: &LoginUser) -> Result<String, AppError> {  // Returns JWT
        let db_user = self.repo.find_by_email(&user.email).await?
            .ok_or(AppError::Auth("User not found".to_string()))?;
        verify(&user.password, &db_user.password_hash)
            .map_err(|_| AppError::Auth("Invalid credentials".to_string()))?;
        let claims = JwtClaims {
            sub: db_user.id,
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(self.jwt_secret.as_ref()))
            .map_err(|_| AppError::Auth("Token generation failed".to_string()))?;
        Ok(token)
    }
}

pub struct AccountingService<R: AccountRepository, T: TransactionRepository> {
    account_repo: R,
    tx_repo: T,
}

impl<R: AccountRepository, T: TransactionRepository> AccountingService<R, T> {
    pub fn new(account_repo: R, tx_repo: T) -> Self { Self { account_repo, tx_repo } }

    pub async fn create_account(&self, user_id: Uuid, acc: &CreateAccount) -> Result<Account, AppError> {
        if !["asset", "liability", "equity", "revenue", "expense"].contains(&acc.type_.as_str()) {
            return Err(AppError::Validation("Invalid account type".to_string()));
        }
        self.account_repo.create(user_id, acc).await
    }

    pub async fn get_accounts(&self, user_id: Uuid) -> Result<Vec<Account>, AppError> {
        self.account_repo.find_by_user(user_id).await
    }

    pub async fn create_transaction(&self, user_id: Uuid, tx: &CreateTransaction) -> Result<Transaction, AppError> {
        // Business rule: Ensure accounts exist and belong to user
        let debit_acc = self.account_repo.find_by_id(tx.debit_account_id, user_id).await?
            .ok_or(AppError::NotFound)?;
        let credit_acc = self.account_repo.find_by_id(tx.credit_account_id, user_id).await?
            .ok_or(AppError::NotFound)?;
        if debit_acc.user_id != user_id || credit_acc.user_id != user_id {
            return Err(AppError::Validation("Accounts must belong to user".to_string()));
        }
        self.tx_repo.create(user_id, tx).await
    }

    pub async fn get_transactions(&self, user_id: Uuid) -> Result<Vec<Transaction>, AppError> {
        self.tx_repo.find_by_user(user_id).await
    }

    pub async fn get_trial_balance(&self, user_id: Uuid) -> Result<Vec<TrialBalance>, AppError> {
        let accounts = self.account_repo.find_by_user(user_id).await?;
        Ok(accounts.into_iter().map(|a| TrialBalance {
            account_id: a.id,
            name: a.name,
            balance: a.balance,
        }).collect())
    }
}
