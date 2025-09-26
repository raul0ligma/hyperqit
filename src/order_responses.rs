use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RestingOrder {
    pub oid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilledOrder {
    pub total_sz: String,
    pub avg_px: String,
    pub oid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    Resting(RestingOrder),
    Error(String),
    Filled(FilledOrder),
    Success,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusResponse {
    pub statuses: Vec<OrderStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetGlobalResponse {
    pub data: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum ExchangeOrderResponse {
    Order(OrderStatusResponse),
    Default,
    Cancel(OrderStatusResponse),
    SetGlobal(SetGlobalResponse),
}

#[cfg(test)]
mod test {
    use crate::internal::ExchangeResponse;

    use super::*;

    #[test]
    fn test_all_exchange_responses() {
        let test_cases = [
            r#"{
               "status":"ok",
               "response":{
                  "type":"order",
                  "data":{
                     "statuses":[
                        {
                           "resting":{
                              "oid":77738308
                           }
                        }
                     ]
                  }
               }
            }"#,
            r#"{
               "status":"ok",
               "response":{
                  "type":"order",
                  "data":{
                     "statuses":[
                        {
                           "error":"Order must have minimum value of $10."
                        }
                     ]
                  }
               }
            }"#,
            r#"{
               "status":"ok",
               "response":{
                  "type":"order",
                  "data":{
                     "statuses":[
                        {
                           "filled":{
                              "totalSz":"0.02",
                              "avgPx":"1891.4",
                              "oid":77747314
                           }
                        }
                     ]
                  }
               }
            }"#,
            r#"{
               "status":"ok",
               "response":{
                  "type":"cancel",
                  "data":{
                     "statuses":[
                        "success"
                     ]
                  }
               }
            }"#,
            r#"{
               "status":"ok",
               "response":{
                  "type":"cancel",
                  "data":{
                     "statuses":[
                        {
                           "error":"Order was never placed, already canceled, or filled."
                        }
                     ]
                  }
               }
            }"#,
            r#"{"status": "ok", "response": {"type": "default"}}"#,
        ];

        for (i, json_str) in test_cases.iter().enumerate() {
            println!("test case {}", i + 1,);

            let exchange_response: Result<ExchangeResponse, _> = serde_json::from_str(json_str);
            match &exchange_response {
                Ok(resp) => println!("base parsed successfully: status={}", resp.status),
                Err(e) => println!(" base parse error: {}", e),
            }

            if let Ok(resp) = exchange_response {
                let order_response: Result<ExchangeOrderResponse, _> =
                    serde_json::from_value(resp.response);
                match order_response {
                    Ok(order_resp) => println!("inner parsed: {:?}", order_resp),
                    Err(e) => println!(" inner parse error: {}", e),
                }
            }
        }
    }
}
