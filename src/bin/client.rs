use std::io::{self};
use tokio::net::{TcpStream};
mod client_utils;
use client_utils::client_data::ClientData;
use client_utils::CLIENTDATALOC;
use std::fs::{self, File};
use simonph_curpas_cryptography::tcpmessage::{self, TcpMessage, recieve_message};
use simonph_curpas_cryptography::tcpmessage::send_message;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::client_utils::encryption::decrypt_file;



async fn send_file(file_name:&str,stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<(),String>{
    let data_file = File::open(CLIENTDATALOC).map_err(|e| e.to_string())?;
    let mut data_base:ClientData = serde_json::from_reader(data_file).map_err(|e| e.to_string())?;
    let (file_data, data) = client_utils::encryption::create_encrytion(file_name)?;
    let message = TcpMessage{
        status:None,
        message_type:Some(tcpmessage::MessageType::Send),
        file_id:Some(file_data.file_id.clone()),
        data:Some(data),
        hash_list:None,
        tree_side:None
    };
    send_message(&message, stream).await?;
    let response = recieve_message(stream).await?;
    if let Some(tcpmessage::Status::Ok) = response.status {} else {return Err("server did not recieve data".to_string())}

    data_base.name_data.insert(file_name.to_string(), file_data);
    serde_json::to_writer(
        File::create(CLIENTDATALOC).map_err(|e| e.to_string())?,
        &data_base
    ).map_err(|e| e.to_string())?;
    return Ok(());
}

async fn recieve_file(file_name:&str,stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<(),String>{
    let data_file = File::open(CLIENTDATALOC).map_err(|e| e.to_string())?;
    let data:ClientData = serde_json::from_reader(data_file).map_err(|e| e.to_string())?;
    let file_data = match data.name_data.get(file_name){
        Some(v) => v,
        None => return Err(format!("cant find the file {}",file_name))
    };
    let message = tcpmessage::TcpMessage{
        status:None,
        message_type:Some(tcpmessage::MessageType::Retrieve),
        file_id:Some(file_data.file_id.clone()),
        data:None,
        hash_list:None,
        tree_side:None,
    };
    send_message(&message, stream).await?;
    let response = recieve_message(stream).await?;
    let data_opt = match response.status{Some(tcpmessage::Status::Ok) => response.data, _ => return Err("no data sent".to_string())};
    let ciphertext = match data_opt{
        Some(v) => v,
        None => return Err("no data field sent from server".to_string())
    };
    let plaintext = decrypt_file(file_data, &ciphertext)?;
    fs::write(file_name, plaintext).map_err(|e| e.to_string())?;
    println!("file loaded to directory");
    return Ok(())
}

async fn check_correctnes(file_name:&str,stream:&mut Framed<TcpStream,LengthDelimitedCodec>)->Result<(),String>{
    let data_file = File::open(CLIENTDATALOC).map_err(|e| e.to_string())?;
    let data:ClientData = serde_json::from_reader(data_file).map_err(|e| e.to_string())?;
    let file_data = match data.name_data.get(file_name){
        Some(v) => v,
        None => return Err(format!("cant find the file {}",file_name))
    };
    let message = TcpMessage{
        status:None,
        message_type:Some(tcpmessage::MessageType::Check),
        file_id:Some(file_data.file_id.clone()),
        data:None,
        hash_list:None,
        tree_side:None,
    };
    send_message(&message, stream).await?;
    let response = recieve_message(stream).await?;
    if let Some(tcpmessage::Status::Ok) = response.status {} else {return Err("server rejected check response".to_string())}
    let hash_arr = match response.hash_list{
        Some(v) => v,
        None => return Err("no hashlist provided by server".to_string())
    };
    let tree_side = match response.tree_side{
        Some(v) => v,
        None => return Err("no tree side list provided by server".to_string())
    };
    if client_utils::encryption::check_correctnes(hash_arr, tree_side, data.root_hash) == false{
        println!("server data is incorrect");
    }
    else{
        println!("server data is correct");
    }
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
        let parts: Vec<&str> = input.split(" ").collect();
        match parts.len(){
            1 => {if parts[0] == "0" {return}}
            2 => {
                match parts[0]{
                    "1" => send_file(parts[1],&mut framed).await.unwrap(),
                    "2" => recieve_file(parts[1],&mut framed).await.unwrap(),
                    "3" => check_correctnes(parts[1], &mut framed).await.unwrap(),
                    _ => println!("{} is not a valid command for length 2",parts[0])
                }
            },
            _ =>{
                println!("too many arguments")
            }
        }
    }

}