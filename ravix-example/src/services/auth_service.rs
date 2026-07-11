use std::sync::Arc;

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use ravix::injectable;
use serde_json::json;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::user::{AuthResponse, Claims, CreateUserDto, LoginDto, UserResponse};
use crate::repositories::UserRepository;
use ravix_apm::Apm;

#[injectable]
pub struct AuthService {
    #[inject]
    repo: Arc<dyn UserRepository>,
    #[inject]
    config: Arc<AppConfig>,
}

impl AuthService {
    /// Register a new user. Returns a JWT + user response.
    pub async fn register(&self, dto: CreateUserDto) -> Result<AuthResponse, AppError> {
        Apm::wrap_span_future(
            "auth.register",
            "app",
            Some(
                [(
                    "email_domain".into(),
                    json!(dto.email.split('@').nth(1).unwrap_or("unknown")),
                )]
                .into(),
            ),
            self.register_inner(dto),
        )
        .await
    }

    async fn register_inner(&self, dto: CreateUserDto) -> Result<AuthResponse, AppError> {
        // Check uniqueness
        if self.repo.find_by_email(&dto.email).await?.is_some() {
            return Err(AppError::Conflict("Email already registered".into()));
        }
        if self.repo.find_by_username(&dto.username).await?.is_some() {
            return Err(AppError::Conflict("Username already taken".into()));
        }

        let password_hash = hash_password(&dto.password).await?;
        let user = self.repo.save(dto, password_hash).await?;

        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            exp: (Utc::now().timestamp() as usize) + self.config.jwt_expiry_seconds as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(format!("JWT encode error: {}", e)))?;

        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    /// Login an existing user. Returns a JWT + user response.
    pub async fn login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
        Apm::wrap_span_future(
            "auth.login",
            "app",
            Some(
                [(
                    "email_domain".into(),
                    json!(dto.email.split('@').nth(1).unwrap_or("unknown")),
                )]
                .into(),
            ),
            self.login_inner(dto),
        )
        .await
    }

    async fn login_inner(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
        let user = self
            .repo
            .find_by_email(&dto.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

        let valid = verify_password(&dto.password, &user.password_hash).await?;
        if !valid {
            return Err(AppError::Unauthorized("Invalid email or password".into()));
        }

        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            exp: (Utc::now().timestamp() as usize) + self.config.jwt_expiry_seconds as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(format!("JWT encode error: {}", e)))?;

        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    /// Verify a JWT and return the claims.
    pub async fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        Apm::wrap_span_future("auth.verify_token", "app", None, async {
            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;
            Ok(token_data.claims)
        })
        .await
    }
}

// ── Argon2 helpers (with APM spans) ──────────────────────────────────────────

async fn hash_password(password: &str) -> Result<String, AppError> {
    let span = Apm::start_span("argon2.hash", "app", None);
    let password = password.to_string();
    let result = tokio::task::spawn_blocking(move || {
        use argon2::{
            password_hash::{PasswordHasher, SaltString},
            Argon2,
        };
        use rand::rngs::OsRng;
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
    })
    .await
    .map_err(|_| AppError::InternalError("hash task panicked".into()))?;
    span.end(None);
    result.map_err(|e| AppError::InternalError(format!("argon2 hash error: {}", e)))
}

async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let password = password.to_string();
    let hash = hash.to_string();
    let span = Apm::start_span("argon2.verify", "app", None);
    let result = tokio::task::spawn_blocking(move || {
        use argon2::{
            password_hash::{PasswordHash, PasswordVerifier},
            Argon2,
        };
        let parsed = PasswordHash::new(&hash)
            .map_err(|e| AppError::InternalError(format!("invalid hash: {}", e)))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map(|_| true)
            .or_else(|_| Ok(false))
    })
    .await
    .map_err(|_| AppError::InternalError("verify task panicked".into()))?;
    span.end(None);
    result
}
