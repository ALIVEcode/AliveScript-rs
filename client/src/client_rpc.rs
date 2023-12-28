use std::io::{Read, Write};

use derive_new::new;
use serde::Deserialize;
use serde_json::{json, Value};

use alivescript_rust::{
    data::{Data, Response},
    io::InterpretorIO,
};

#[derive(Deserialize)]
struct RpcError {
    message: String,
    code: i64,
    id: Option<usize>,
    data: Option<Value>,
}

#[derive(Deserialize)]
struct RpcResult {
    result: Value,
    id: Option<usize>,
}

enum RpcResponse {
    Result(RpcResult),
    Error(RpcError),
}

#[derive(Debug)]
struct InvalidResponseError(String);

fn single_response_from_message(
    message: &serde_json::Map<String, Value>,
) -> Result<RpcResponse, InvalidResponseError> {
    if !matches!(message.get("jsonrpc".into()), Some(Value::String(x)) if x == "2.0") {
        return Err(InvalidResponseError(
            "\"jsonrpc\" field not set to 2.0".into(),
        ));
    }

    let Some(Value::Number(id)) = message.get("id".into()) else {
        return Err(InvalidResponseError("No \"id\" field".into()));
    };
    let id = id.as_i64().unwrap() as usize;

    if let Some(result) = message.get("result".into()) {
        return Ok(RpcResponse::Result(RpcResult {
            result: result.clone(),
            id: Some(id),
        }));
    };
    if let Some(error) = message.get("error".into()) {
        return Ok(RpcResponse::Error(RpcError {
            message: error["message"].as_str().unwrap().into(),
            code: error["code"].as_i64().unwrap(),
            id: Some(id),
            data: Some(error["data"].clone()),
        }));
    }
    unreachable!()
}

#[derive(new)]
pub struct ClientRPC<'a> {
    reader: &'a mut dyn Read,
    writer: &'a mut dyn Write,
    #[new(value = "0")]
    next_request_id: usize,
}

impl<'a> ClientRPC<'a> {}

impl InterpretorIO for ClientRPC<'_> {
    fn send(&mut self, data: Data) {
        let (method, params) = match data {
            Data::Afficher(msg) => ("afficher", json!(vec![msg])),
            Data::Erreur { texte, ligne } => todo!(),
            Data::Demander { prompt } => todo!(),
            Data::NotifInfo { msg } => todo!(),
            Data::GetFichier(..) => todo!(),
            Data::NotifErr { msg } => todo!(),
        };

        let result = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        write!(self.writer, "{}", result).expect("To write into the buffer");
        self.writer.flush().expect("To flush");
    }

    fn request(&mut self, data: Data) -> Option<Response> {
        let req_id = self.next_request_id;
        self.next_request_id += 1;

        let (method, params) = match data {
            Data::Afficher(msg) => ("afficher", json!(vec![msg])),
            Data::Erreur { texte, ligne } => todo!(),
            Data::Demander { prompt } => ("demander", json!([prompt])),
            Data::NotifInfo { msg } => todo!(),
            Data::GetFichier(..) => None?,
            Data::NotifErr { msg } => todo!(),
        };

        let result = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": req_id
        });

        write!(self.writer, "{}", result).expect("To write into the buffer");
        self.writer.flush().expect("To flush");

        let responses: Value = serde_json::from_reader(&mut self.reader).ok()?;
        let response = match responses {
            Value::Array(res) => res,
            res @ Value::Object(..) => vec![res],
            _ => panic!("Received a bad json value"),
        };
        for r in response.iter().map(|r| single_response_from_message(r.as_object().unwrap())) {
            match r {
                Ok(RpcResponse::Error(RpcError { message, code, id, data })) => todo!(),
                Ok(RpcResponse::Result(RpcResult { result, id })) => {
                    if id == Some(req_id) {
                        return Some(Response::Text(result.as_str().unwrap().into()));
                    } else {
                        return None;
                    }
                }
                Err(err) => panic!("{:?}", err)
            }
        }
        panic!()
    }
}
