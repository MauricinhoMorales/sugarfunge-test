use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestError {
    pub message: serde_json::Value,
    pub description: String,
}

pub fn endpoint(cmd: &'static str) -> String {
    format!("http://127.0.0.1:4000/{}", cmd)
}

pub async fn req<'a, I, O>(cmd: &'static str, args: I) -> Result<O, RequestError>
where
    I: Serialize,
    O: for<'de> Deserialize<'de>,
{
    let sf_res = reqwest::Client::new()
        .post(endpoint(cmd))
        .json(&args)
        .send()
        .await;

    match sf_res {
        Ok(res) => {
            if let Err(err) = res.error_for_status_ref() {
                match res.json::<RequestError>().await {
                    Ok(err) => Err(err),
                    Err(_) => Err(RequestError {
                        message: json!(format!("{:#?}", err)),
                        description: "Reqwest json error.".into(),
                    }),
                }
            } else {
                match res.json().await {
                    Ok(res) => Ok(res),
                    Err(err) => Err(RequestError {
                        message: json!(format!("{:#?}", err)),
                        description: "Reqwest json error.".into(),
                    }),
                }
            }
        }
        Err(err) => Err(RequestError {
            message: json!(format!("{:#?}", err)),
            description: "Reqwest error.".into(),
        }),
    }
}
