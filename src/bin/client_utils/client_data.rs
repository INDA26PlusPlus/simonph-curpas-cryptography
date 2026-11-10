use crate::client_utils::client_file::ClientFile;
use std::collections::HashMap;
use serde::{Serialize,Deserialize};
#[derive(Serialize,Deserialize,Debug)]
pub struct ClientData{
    pub root_hash:[u8;32],
    pub name_data:HashMap<String,ClientFile>
}

