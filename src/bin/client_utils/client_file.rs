pub struct ClientFile{
    pub file_id:String,
    pub file_hash:[u8;32],
    pub nonce:[u8;12],
    pub salt:[u8;12]
}