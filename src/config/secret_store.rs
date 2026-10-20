use async_trait::async_trait;
use news_flash::feed_api::{SecretError, SecretKey, SecretKind, SecretStore};

use crate::prelude::*;

#[allow(dead_code)]
pub struct ConfiguredSecretStore {
    login_setup: Option<LoginConfiguration>,
}

#[allow(dead_code)]
impl ConfiguredSecretStore {
    pub fn new(login_setup: Option<LoginConfiguration>) -> Self {
        Self { login_setup }
    }
}

#[async_trait]
impl SecretStore for ConfiguredSecretStore {
    async fn get(&self, key: &SecretKey) -> Result<Option<String>, SecretError> {
        let Some(login_setup) = self.login_setup.as_ref() else {
            return Err(SecretError::Backend(
                "config option `login_setup` not set: no login configuration available".to_owned(),
            ));
        };

        if login_setup.provider != key.plugin.as_str() {
            return Err(SecretError::Backend(format!(
                "mismatch of plugin ids: `{}` (from news-flash) vs. `{}` (from config)",
                key.plugin, login_setup.provider
            )));
        }

        use SecretKind as K;
        Ok(<Option<&Secret>>::map(
            match key.kind {
                K::Password => login_setup.password.as_ref(),
                K::HttpPassword => login_setup.basic_auth_password.as_ref(),
                K::ApiKey => todo!(),
                K::RefreshToken => todo!(),
                K::AccessToken => todo!(),
                K::ClientSecret => todo!(),
                K::Session => todo!(),
            },
            |secret| {
                secret
                    .get_secret()
                    .map_err(|error| SecretError::Backend(error.to_string()))
            },
        )
        .transpose()?)
    }

    async fn set(&self, key: &SecretKey, _secret: &str) -> Result<(), SecretError> {
        log::trace!("set {key:?}");
        Ok(())
    }

    async fn delete(&self, key: &SecretKey) -> Result<(), SecretError> {
        log::trace!("delete {key:?}");
        Ok(())
    }
}
