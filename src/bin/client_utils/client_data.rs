use crate::client_utils::client_file::ClientFile;
use std::collections::HashMap;
struct ClientData{
    pub root_hash:[u8;32],
    pub name_data:HashMap<String,ClientFile>
}