//! A small model catalog in the shape `/v1/zeldoc/models` returns, shared by
//! the models tests.

use super::data_transfer_objects::model_dto::ModelDTO;
use super::data_transfer_objects::model_list_dto::ModelListDTO;

pub fn models() -> Vec<ModelDTO> {
    serde_json::from_str::<ModelListDTO>(
        r#"{"data":[
            {"id":"zdev","mode":"chat","limits":{"context":1000000,"output":131072},
             "pricing":{"input":"0","output":"0","cache_read":"0","cache_write":null},
             "capabilities":{"tool_calls":true,"reasoning":true,"vision":true,"pdf_input":false,"prompt_caching":false,"reasoning_efforts":["low","high","max"]}},
            {"id":"openai/text-embedding-3-large","mode":"embedding","limits":{"context":8191,"output":null},
             "pricing":{"input":"0.13","output":"0.13","cache_read":null,"cache_write":null},
             "capabilities":{"tool_calls":false,"reasoning":false,"vision":false,"pdf_input":false,"prompt_caching":false,"reasoning_efforts":[]}},
            {"id":"jev-latest","mode":null,"limits":{"context":null,"output":null},
             "pricing":{"input":null,"output":null,"cache_read":null,"cache_write":null},
             "capabilities":{"tool_calls":false,"reasoning":false,"vision":false,"pdf_input":false,"prompt_caching":false,"reasoning_efforts":[]}}
        ]}"#,
    )
    .unwrap()
    .data
}
