use std::io::{self};
use tokio::net::{TcpStream};
use std::fs::{self, File};
use simonph_curpas_cryptography::tcpmessage::{self, TcpMessage, recieve_message,send_message};
mod client_utils;
use client_utils::client_data::ClientData;
use client_utils::CLIENTDATALOC;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use client_utils::encryption::{hash_file,decrypt_file,get_root_with_merge};
use client_utils::client_file::ClientFile;

fn update_root(hash_list:&Vec<[u8;32]>, tree_side:&Vec<bool>,file_hash:[u8;32])->Result<(),String>{
    let data_file = File::open(CLIENTDATALOC).map_err(|e| e.to_string())?;
    let mut data_base:ClientData = serde_json::from_reader(data_file).map_err(|e| e.to_string())?;
    if get_root_with_merge(hash_list, tree_side,[0u8;32]) != data_base.root_hash{
        return Err("root hash is not correct".to_string());
    }
    data_base.root_hash = get_root_with_merge(hash_list, tree_side, file_hash);
    serde_json::to_writer(
        File::create(CLIENTDATALOC).map_err(|e| e.to_string())?,
        &data_base
    ).map_err(|e| e.to_string())?;
    return Ok(());
}

async fn send_file(file_name:&str,stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<(),String>{
    let file_data = client_utils::encryption::create_encrytion(file_name)?;
    let data = serde_json::ser::to_vec(&file_data).map_err(|e| e.to_string())?;
    let message = TcpMessage{
        status:None,
        message_type:Some(tcpmessage::MessageType::Send),
        file_id:Some(file_data.file_id.clone()),
        data:Some(data.clone()),
        hash_list:None,
        tree_side:None
    };
    send_message(&message, stream).await?;
    let response = recieve_message(stream).await?;
    let hash_list = response.hash_list.ok_or("Server did not send hash")?;
    let tree_side = response.tree_side.ok_or("server did not have tree side")?;
    update_root(&hash_list,&tree_side,hash_file(&data)?)?;
    if let Some(tcpmessage::Status::Ok) = response.status {} else {return Err("server did not recieve data".to_string())}
    return Ok(());
}

async fn recieve_file(file_name:&str,stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<(),String>{
    let message = tcpmessage::TcpMessage{
        status:None,
        message_type:Some(tcpmessage::MessageType::Retrieve),
        file_id:Some(file_name.to_string()),
        data:None,
        hash_list:None,
        tree_side:None,
    };
    send_message(&message, stream).await?;
    let response = recieve_message(stream).await?;
    if let Some(tcpmessage::Status::Reject) = response.status{
        return Err("server rejected".to_string());
    }
    let data = response.data.ok_or("data not sent")?;
    let file_data:ClientFile = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    let hash_list = response.hash_list.ok_or("Server did not send hash")?;
    let tree_side = response.tree_side.ok_or("server did not have tree side")?;
    update_root(&hash_list,&tree_side,hash_file(&data)?)?;
    let plaintext = decrypt_file(&file_data)?;
    fs::write(file_name, plaintext).map_err(|e| e.to_string())?;
    println!("file loaded to directory");
    return Ok(())
}

async fn check(file_name:&str,stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<(),String>{
    let message = TcpMessage{
        status:None,
        message_type:Some(tcpmessage::MessageType::Check),
        file_id:Some(file_name.to_string()),
        data:None,
        hash_list:None,
        tree_side:None,
    };
    send_message(&message, stream).await?;
    let response = recieve_message(stream).await?;
    if let Some(tcpmessage::Status::Ok) = response.status {} else {return Err("server rejected check response".to_string())}
    println!("all went well :)");
    return Ok(());
}

#[tokio::main]
async fn main(){
    println!("write server ip");
    let mut ip = String::new();
    io::stdin().read_line(&mut ip).expect("not a valid string");
    let stream = TcpStream::connect(ip.trim()).await.unwrap();
    let mut framed   = Framed::new(stream, LengthDelimitedCodec::new());
    loop{
        println!("to exit write 0");
        println!("to send file to server write 1 {{file_name}}");
        println!("to recieve file from server write 2 {{file_name}}");
        println!("to check if a file is stored write 3 {{file_name}}");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("read error");
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.len(){
            1 => {if parts[0] == "0" {return}}
            2 => {
                match parts[0]{
                    "1" => send_file(parts[1],&mut framed).await.unwrap(),
                    "2" => recieve_file(parts[1],&mut framed).await.unwrap(),
                    "3" => check(parts[1], &mut framed).await.unwrap(),
                    _ => println!("{} is not a valid command for length 2",parts[0])
                }
            },
            _ =>{
                println!("too many arguments")
            }
        }
    }
}