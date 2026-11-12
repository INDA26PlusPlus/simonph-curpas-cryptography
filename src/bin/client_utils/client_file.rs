use serde::{Serialize,Deserialize};
#[derive(Serialize,Deserialize,Debug)]
pub struct ClientFile{
    pub file_id:String,
    pub nonce:[u8;12],
    pub salt:[u8;12],
    pub data:Vec<u8>
}