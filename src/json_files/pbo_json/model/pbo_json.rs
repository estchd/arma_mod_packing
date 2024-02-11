use serde::{Deserialize, Serialize};
use crate::json_files::pbo_json::model::pbo_compress::PBOCompress;
use crate::json_files::pbo_json::model::pbo_header::PBOHeader;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PBOJson
{
    #[serde(default)]
    pub headers: Vec<PBOHeader>,

    #[serde(default)]
    pub compress: Option<PBOCompress>
}