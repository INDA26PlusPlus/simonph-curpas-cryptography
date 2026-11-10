use std::io::{self};
use tokio::net::{TcpListener,TcpStream};
use simonph_curpas_cryptography::file_structs;
async fn send_file(file_name:&str,stream:&TcpStream){

}

async fn recieve_file(file_id:&str,stream:&TcpStream){

}


#[tokio::main]
async fn main(){
    println!("write server ip");
    let mut ip = String::new();
    io::stdin().read_line(&mut ip).expect("not a valid string");
    let stream = TcpStream::connect(ip).await.unwrap();
    loop{
        println!("to exit write 0");
        println!("to send file to server write 1 {{file_name}}");
        println!("to recieve file from server write 2 {{file_id}}");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("read error");
        let parts: Vec<&str> = input.split(" ").collect();
        match parts.len(){
            1 => {if parts[0] == "0" {return}}
            2 => {
                match parts[0]{
                    "1" => send_file(parts[1],&stream).await,
                    "2" => recieve_file(parts[1],&stream).await,
                    _ => println!("{} is not a valid command for length 2",parts[0])
                }
            },
            _ =>{
                println!("too many arguments")
            }
        }
    }

}