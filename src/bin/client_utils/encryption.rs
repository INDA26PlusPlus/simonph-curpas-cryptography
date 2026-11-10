use rand::random;
use sha2::{Sha256, Digest};
use argon2::Argon2;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use std::fs;
use crate::client_utils::client_file::ClientFile;

pub fn check_correctnes(hash_arr:Vec<[u8;32]>,merge_side:Vec<bool>,root_hash:[u8;32])->bool{
    let mut curhash = hash_arr_arr[0];
    for i in 1..hash_arr.len(){
        cur = hash_arr[i];
        let (l,r) = match merge_side[i-1]{
            true => (curhash,cur),
            false => (cur,curhash)
        };
        let hasher = Sha256::new();
        hasher.update(l);
        hasher.update(r);
        curhash = hasher.finalize().into()
    }
    return curhash == root_hash;
}

pub fn make_user_get_key()->Result<[u8;32],String>{
    let password = String::new();
    std::io::stdin().read_line(&mut password)?;
    let key = Argon2::hash_password(&self, password);
    return Ok(key);
}

pub fn encrypt(plaintext:&Vec<u8>,key:&[u8;32], nonce:&[u8;32])->Result<Vec<u8>,String>{
    let cipher = Aes256Gcm::new(key.into());
    let cipthertext = cipher.encrypt(nonce.into(), plaintext)?;
    return Ok(cipthertext);
}

pub fn decrypt(ciphertext:&Vec<u8>, key:&[u8;32], nonce:&[u8;32])->Result<Vec<u8>,String>{
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher.decrypt(nonce.into(), ciphertext)?;
    return Ok(plaintext)
}

pub fn hash_file(data:Vec<u8>)->Result<[u8;32], String>{
    let hasher = Sha256::new();
    hasher.update(data);
    return hasher.finalize().into();
}
pub fn create_encrytion(file_id:&str, file_name:&str)->Result<(ClientFile,Vec<u8>), String>{
    let mut nonce = [0u8;32];
    for &mut x in nonce.iter_mut(){
        x = random();
    }
    let key = make_user_get_key()?;
    let data = fs::read(file_name)?;
    let hash = hash_file(data)?;
    let cipher_text = encrypt_file(&data, &key, &nonce)?;
    return Ok((ClientFile{file_id:file_id.to_string(),file_hash:hash,nonce},cipher_text));
}
pub fn decrypt_file(file:&ClientFile,ciphertext:&Vec<u8>)->Result<Vec<u8>, String>{
    let key = make_user_get_key()?;
    let plaintext = decrypt(ciphertext, &key, &file.nonce)?;
    if file.file_hash != hash_file(plaintext)?{
        return Err("hash missmatch".to_string());
    }
    return Ok(plaintext);
}


