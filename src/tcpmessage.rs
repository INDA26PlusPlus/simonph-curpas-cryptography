use serde::{Serialize,Deserialize};
use futures_util::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use bytes::Bytes;
use futures_util::SinkExt;
use tokio::net::{TcpStream};
#[derive(Serialize,Deserialize,Debug)]
pub enum Status{
    Ok,
    Reject
}
#[derive(Serialize,Deserialize,Debug)]
pub enum MessageType{
    Retrieve,
    Send,
    Check,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct TcpMessage{
    pub status:Option<Status>,
    pub message_type:Option<MessageType>,
    pub file_id:Option<String>,
    pub data:Option<Vec<u8>>,
    pub hash_list:Option<Vec<[u8;32]>>,
    pub tree_side:Option<Vec<bool>>,
}

pub async fn send_message(data:&TcpMessage,stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<(),String>{
    let json = serde_json::to_vec(data).map_err(|e| e.to_string())?;
    stream.send(Bytes::from(json)).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn recieve_message(stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<TcpMessage,String>{
    let bytes = match stream.next().await{
        Some(Err(e)) => return Err(e.to_string()),
        Some(Ok(v)) => v,
        None => return Err("could not recieve message".to_string()),
    };
    let message:TcpMessage = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    return Ok(message);
}