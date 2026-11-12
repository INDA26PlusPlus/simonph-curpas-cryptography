use serde::{Serialize,Deserialize};
#[derive(Serialize,Deserialize,Debug)]
pub struct ClientData{
    pub root_hash:[u8;32],
}

