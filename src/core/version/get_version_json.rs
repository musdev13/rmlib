use crate::core;

pub async fn get_version_json(version_id: String) -> String{
   let url = core::version::get_version_json_url(version_id).await;
   musutils::http::get_json_str(url).await
}
