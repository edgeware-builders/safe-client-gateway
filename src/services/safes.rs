use crate::models::service::safes::SafeInfoEx;
use crate::providers::info::{DefaultInfoProvider, InfoProvider};
use crate::utils::context::Context;
use crate::utils::errors::ApiResult;

pub fn get_safe_info_ex(context: &Context, safe_address: &String) -> ApiResult<SafeInfoEx> {
    let mut info_provider = DefaultInfoProvider::new(context);
    let safe_info = info_provider.safe_info(safe_address)?;

    Ok(safe_info.to_safe_info_ex(&mut info_provider))
}
