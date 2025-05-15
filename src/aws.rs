use aws_runtime::env_config;
use aws_types::{SdkConfig, os_shim_internal};
use color_eyre::Result;
use env_config::file;

pub mod s3_client;

#[derive(Debug)]
pub struct AWS {
    pub config: SdkConfig,
}

impl AWS {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        Self { config }
    }

    async fn get_profile_set() -> Result<Vec<String>> {
        let fs = os_shim_internal::Fs::real();
        let env = os_shim_internal::Env::real();
        let profile_files = file::EnvConfigFiles::default();
        let profiles_set = aws_config::profile::load(&fs, &env, &profile_files, None).await?;
        let section_names: Vec<String> = profiles_set.profiles().map(|s| s.to_string()).collect();
        Ok(section_names)
    }
}
