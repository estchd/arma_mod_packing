use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathJson
{
    pub paa_converter_path: String,
    pub rvmat_converter_path: String,
    pub config_converter_path: String,
    pub pbo_packer_path: String,
    pub pbo_signer_path: String
}

impl Default for PathJson
{
    fn default() -> Self
    {
        Self {
            paa_converter_path: "./external_tools/paa/ImageToPAA.exe".to_string(),
            rvmat_converter_path: "./external_tools/config/CfgConvert.exe".to_string(),
            config_converter_path: "./external_tools/config/CfgConvert.exe".to_string(),
            pbo_packer_path: "./external_tools/pbo/pboc.exe".to_string(),
            pbo_signer_path: "./external_tools/signing/dsSignFile.exe".to_string()
        }
    }
}