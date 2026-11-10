Tcp protokoll:

#Om client vill skicka över data skickar den json object

{
    type:"send",
    file_id:str,
    file_length:int
}

Clienten skickar sedan filen som byte data

Server respondar

{
    Status:"Ok",
    file_hash:[u8;32],
}

om den fick emot filen

{
    Status:"Failure"
}


#Om client vill kolla om filen med file_id är lagrad

{
    type:"check",
    file_id:file_id
}

Server svarar

{
    hash_vec:list([u8;32])
    tree_side:list(bool)
}

#Om clienten vill få emot fil data

{
    type:"recieve",
    file_id:file_id
}

om servern accepterar svarar den

{
    status:"OK",
    data:Vec<u8>
}