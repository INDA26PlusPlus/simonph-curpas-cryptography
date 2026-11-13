use std::{path::Path, sync::Arc};

use sha2::{Digest, Sha256};
use simonph_curpas_cryptography::tcpmessage::{
    recieve_message, send_message, MessageType, Status, TcpMessage,
};
use tokio::{
    fs,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

const EMPTY: &str = "-";
const DATA_DIR: &str = "data";
const INDEX_PATH: &str = "data/index";
const INDEX_TMP: &str = "data/index.tmp";

type Hash = [u8; 32];
type Conn = Framed<TcpStream, LengthDelimitedCodec>;

fn hash_bytes(data: &[u8]) -> Hash {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().into()
}

fn hash_pair(l: &Hash, r: &Hash) -> Hash {
    let mut h = Sha256::new();
    h.update(l);
    h.update(r);
    h.finalize().into()
}

struct State {
    ids: Vec<String>,
    tree: Vec<Hash>,
}

impl State {
    async fn load() -> Result<State, String> {
        fs::create_dir_all(DATA_DIR).await.map_err(|e| e.to_string())?;

        let ids: Vec<String> = if Path::new(INDEX_PATH).exists() {
            let bytes = fs::read(INDEX_PATH).await.map_err(|e| e.to_string())?;
            let ids: Vec<String> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if ids.len() != 1024 {
                return Err(format!("index has {} entries, expected {}", ids.len(), 1024));
            }
            ids
        } else {
            let ids = vec![EMPTY.to_string(); 1024];
            let bytes = serde_json::to_vec(&ids).map_err(|e| e.to_string())?;
            fs::write(INDEX_PATH, bytes).await.map_err(|e| e.to_string())?;
            ids
        };

        let mut tree = vec![[0u8; 32]; 2048];
        for (i, id) in ids.iter().enumerate() {
            if id != EMPTY {
                let data = fs::read(format!("{}/{}", DATA_DIR, i))
                    .await
                    .map_err(|e| format!("slot {} ({}) unreadable: {}", i, id, e))?;
                tree[1024 + i] = hash_bytes(&data);
            }
        }
        for k in (1..1024).rev() {
            tree[k] = hash_pair(&tree[2 * k], &tree[2 * k + 1]);
        }

        Ok(State { ids, tree })
    }

    fn find(&self, file_id: &str) -> Option<usize> {
        self.ids.iter().position(|id| id == file_id)
    }

    fn first_free(&self) -> Option<usize> {
        self.ids.iter().position(|id| id == EMPTY)
    }

    fn set_leaf(&mut self, index: usize, leaf: Hash) {
        let mut pos = 1024 + index;
        self.tree[pos] = leaf;
        while pos > 1 {
            pos /= 2;
            self.tree[pos] = hash_pair(&self.tree[2 * pos], &self.tree[2 * pos + 1]);
        }
    }

    /// Currently has true = assembled hash on the left
    fn create_mask(&self, index: usize) -> (Vec<Hash>, Vec<bool>) {
        let mut mask = Vec::new();
        let mut sides = Vec::new();
        let mut pos = 1024 + index;
        while pos > 1 {
            mask.push(self.tree[pos ^ 1]);
            sides.push(pos % 2 == 0);
            pos /= 2;
        }
        (mask, sides)
    }

    async fn save_index(&self) -> Result<(), String> {
        let bytes = serde_json::to_vec(&self.ids).map_err(|e| e.to_string())?;
        fs::write(INDEX_TMP, bytes).await.map_err(|e| e.to_string())?;
        fs::rename(INDEX_TMP, INDEX_PATH).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

type Shared = Arc<Mutex<State>>;

fn ok_response() -> TcpMessage {
    TcpMessage {
        status: Some(Status::Ok),
        message_type: None,
        file_id: None,
        data: None,
        hash_list: None,
        tree_side: None,
    }
}

fn failure(msg: &str) -> TcpMessage {
    TcpMessage {
        status: Some(Status::Reject),
        message_type: None,
        file_id: None,
        data: Some(msg.as_bytes().to_vec()),
        hash_list: None,
        tree_side: None,
    }
}


// Handlers

async fn send(state: &Shared, file_id: Option<String>, data: Option<Vec<u8>>) -> Result<TcpMessage, String> {
    let file_id = file_id.ok_or("send is missing file_id")?;
    let data = data.ok_or("send is missing data")?;

    if file_id == EMPTY {
        return Err(format!("\"{}\" is not a valid file_id", EMPTY));
    }

    let mut st = state.lock().await;
    if st.find(&file_id).is_some() {
        return Err("File already exists".to_string());
    }
    let i = st.first_free().ok_or("Server is full")?;

    fs::write(format!("{}/{}", DATA_DIR, i), &data)
        .await
        .map_err(|e| e.to_string())?;

    st.ids[i] = file_id.clone();
    if let Err(e) = st.save_index().await {
        st.ids[i] = EMPTY.to_string();
        return Err(e);
    }
    st.set_leaf(i, hash_bytes(&data));

    let (mask, sides) = st.create_mask(i);
    let mut resp = ok_response();
    resp.hash_list = Some(mask);
    resp.tree_side = Some(sides);
    println!("Wrote file {}", file_id);
    Ok(resp)
}

async fn check(state: &Shared, file_id: Option<String>) -> Result<TcpMessage, String> {
    let file_id = file_id.ok_or("check is missing file_id")?;
    let st = state.lock().await;
    if st.find(&file_id).is_some() {
        Ok(ok_response())
    } else {
        Err("File not found".to_string())
    }
}

async fn get(state: &Shared, file_id: Option<String>) -> Result<TcpMessage, String> {
    let file_id = file_id.ok_or("retrieve is missing file_id")?;
    let st = state.lock().await;
    let i = st.find(&file_id).ok_or("File not found")?;
    println!("Accessing file {}", i.clone());
    let data = fs::read(format!("{}/{}", DATA_DIR, i))
        .await
        .map_err(|e| e.to_string())?;
    let (mask, sides) = st.create_mask(i);

    let mut resp = ok_response();
    resp.data = Some(data);
    resp.hash_list = Some(mask);
    resp.tree_side = Some(sides);
    println!("Sending ok");
    Ok(resp)
}


// TCP loop

async fn process(stream: TcpStream, state: Shared) {
    let mut framed: Conn = Framed::new(stream, LengthDelimitedCodec::new());

    loop {
        let msg = match recieve_message(&mut framed).await {
            Ok(m) => m,
            Err(e) => {
                println!("Connection closed: {}", e);
                return;
            }
        };

        let result = match msg.message_type {
            Some(MessageType::Send) => send(&state, msg.file_id, msg.data).await,
            Some(MessageType::Check) => check(&state, msg.file_id).await,
            Some(MessageType::Retrieve) => get(&state, msg.file_id).await,
            None => Err("Unrecognized type: message_type missing".to_string()),
        };

        let response = result.unwrap_or_else(|e| failure(&e));
        if let Err(e) = send_message(&response, &mut framed).await {
            println!("Failed to send response: {}", e);
            return;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state: Shared = Arc::new(Mutex::new(State::load().await?));
    let listener = TcpListener::bind("127.0.0.1:4489").await?;
    println!("Boot complete");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("Connection from {}", addr);
                tokio::spawn(process(stream, state.clone()));
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }
}