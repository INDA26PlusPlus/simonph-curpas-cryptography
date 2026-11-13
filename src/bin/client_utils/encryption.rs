use rand::random;
use sha2::{Sha256, Digest};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm
};
use std::fs;
use crate::client_utils::client_file::ClientFile;
pub fn get_root_with_merge(hash_arr:&Vec<[u8;32]>,merge_side:&Vec<bool>,start_hash:[u8;32])->[u8;32]{
    let mut curhash=start_hash;
    for i in 0..hash_arr.len(){
        let cur = hash_arr[i];
        let (l,r) = match merge_side[i]{
            true => (curhash,cur),
            false => (cur,curhash)
        };
        let mut hasher = Sha256::new();
        hasher.update(l);
        hasher.update(r);
        curhash = hasher.finalize().into()
    }
    return curhash
}

pub fn make_user_get_key(salt:&[u8;12])->Result<[u8;32],String>{
    let mut password = String::new();
    println!("Provide password");
    std::io::stdin().read_line(&mut password).unwrap();
    let argon = argon2::Argon2::default();
    let mut key = [0u8;32];
    argon.hash_password_into(&password.into_bytes(), salt, &mut key).map_err(|e| e.to_string())?;
    return Ok(key);
}

pub fn encrypt(plaintext:&Vec<u8>,key:&[u8;32], nonce:&[u8;12])->Result<Vec<u8>,String>{
    let cipher = Aes256Gcm::new(key.into());
    let cipthertext = match cipher.encrypt(nonce.into(), plaintext.as_slice()){
        Ok(v) => v,
        Err(e) => return Err(e.to_string()), 
    };
    return Ok(cipthertext);
}

pub fn decrypt(ciphertext:&Vec<u8>, key:&[u8;32], nonce:&[u8;12])->Result<Vec<u8>,String>{
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = match cipher.decrypt(nonce.into(), ciphertext.as_slice()){
        Ok(v) => v,
        Err(e) => return Err(e.to_string())
    };
    return Ok(plaintext)
}

pub fn hash_file(data:&Vec<u8>)->Result<[u8;32], String>{
    let mut hasher = Sha256::new();
    hasher.update(data);
    return Ok(hasher.finalize().into());
}
pub fn create_encrytion(file_name:&str)->Result<ClientFile, String>{
    let mut nonce = [0u8;12];
    for x in nonce.iter_mut(){
        *x = random();
    }
    let mut salt = [0u8;12];
    for x in salt.iter_mut(){
        *x = random();
    }
    let key = make_user_get_key(&salt)?;
    let data = fs::read(file_name).map_err(|e| e.to_string())?;
    let cipher_text = encrypt(&data, &key, &nonce)?;
    return Ok(ClientFile{file_id:file_name.to_string(),nonce,salt,data:cipher_text});
}
pub fn decrypt_file(file:&ClientFile)->Result<Vec<u8>, String>{
    let key = make_user_get_key(&file.salt)?;
    let plaintext = decrypt(&file.data, &key, &file.nonce)?;
    return Ok(plaintext);
}


