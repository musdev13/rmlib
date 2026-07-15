use std::path::PathBuf;

use crate::core;

pub async fn install(version_id: String, versions_path_option: Option<PathBuf>, libs_path_option: Option<PathBuf>){
    let version_json_str = core::version::get_version_json(version_id).await;
    println!("{}", version_json_str);

    // let versions_path: PathBuf = directory.unwrap_or_else(|| {
    //     println!("using default path default path default... path..?");
    //     musutils::fs::tilda_desir(config::get_versions_path())
    // });

    // println!("{}", versions_path.display());
}
