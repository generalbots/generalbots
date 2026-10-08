use super::*;

pub(crate) fn jwt_sign_inner(message: &str, secret: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    mac.update(message.as_bytes());
    base64_url_encode(&mac.finalize().into_bytes())
}

#[derive(Debug, Deserialize)]
pub struct CheckoutBody {
    pub payload: String,
    pub email: String,
    pub organization_name: Option<String>,
    pub return_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupBody {
    pub email: String,
    pub name: String,
    pub bot_name: Option<String>,
    pub password: Option<String>,
    pub plan: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgBody {
    pub name: String,
    pub plan: Option<String>,
    pub period: Option<String>,
    pub storage_gb: Option<f64>,
    pub ai_addons: Option<Vec<String>>,
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgBody {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub plan: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ProfileUpdateBody {
    pub name: Option<String>,
    pub organization: Option<String>,
}

